//! Port of `intern/wm_jobs.cc` — `wmJob` background-job
//! management: get/find, suspend-stop arbitration,
//! start/end/kill, progress aggregation, task pools.

/// `WM_JOB_TYPE_*`/`WM_JOB_FLAG_*` verbatim.
pub const WM_JOB_TYPE_ANY: i32 = -1;
pub const WM_JOB_PROGRESS: i32 = 1 << 0;
pub const WM_JOB_EXCL_RENDER: i32 = 1 << 1;
pub const WM_JOB_PRIORITY: i32 = 1 << 2;

/// `wmJob` fields used (verbatim).
pub struct WmJob {
    /// `win`.
    pub win: usize,
    /// `owner`.
    pub owner: usize,
    /// `flag`.
    pub flag: i32,
    /// `job_type`.
    pub job_type: i32,
    /// `name`.
    pub name: String,
    /// `running`/`ready`/`suspended`/`stop`/`finished`.
    pub running: bool,
    pub ready: bool,
    pub suspended: bool,
    pub stop: bool,
    pub finished: bool,
    /// `start_time`.
    pub start_time: f64,
    /// `start_delay_time`/`time_step`.
    pub start_delay_time: f64,
    pub time_step: f64,
    /// `customdata`/`run_customdata`/`free`/`run_free`.
    pub customdata: usize,
    pub run_customdata: usize,
    /// `worker_status.progress`.
    pub progress: f32,
    /// `wt` (timer handle).
    pub wt: usize,
    /// `initjob`/`startjob` present.
    pub has_initjob: bool,
    pub has_startjob: bool,
}

/// `wm_job_find` (verbatim — owner + type match).
pub fn wm_job_find(jobs: &[WmJob], owner: usize, job_type: i32) -> Option<usize> {
    jobs.iter().position(|j| j.owner == owner && j.job_type == job_type)
}

/// `WM_jobs_get` (verbatim — find-or-create + lock +
/// reports init; job_type != ANY asserted).
pub fn wm_jobs_get(
    jobs: &mut Vec<WmJob>,
    win: usize,
    owner: usize,
    name: &str,
    flag: i32,
    job_type: i32,
) -> usize {
    if let Some(i) = wm_job_find(jobs, owner, job_type) {
        return i;
    }
    jobs.push(WmJob {
        win,
        owner,
        flag,
        job_type,
        name: name.to_string(),
        running: false,
        ready: false,
        suspended: false,
        stop: false,
        finished: false,
        start_time: 0.0,
        start_delay_time: 0.0,
        time_step: 0.0,
        customdata: 0,
        run_customdata: 0,
        progress: 0.0,
        wt: 0,
        has_initjob: false,
        has_startjob: false,
    });
    jobs.len() - 1
}

/// `WM_jobs_progress_test` (verbatim — PROGRESS flag +
/// running/suspended).
pub fn wm_jobs_progress_test(jobs: &[WmJob], owner: usize, job_type: i32) -> bool {
    jobs.iter().any(|j| {
        j.owner == owner
            && (job_type == WM_JOB_TYPE_ANY || j.job_type == job_type)
            && (j.flag & WM_JOB_PROGRESS) != 0
            && (j.running || j.suspended)
    })
}

/// `WM_jobs_progress` (verbatim — first matching owner,
/// PROGRESS gate).
pub fn wm_jobs_progress(jobs: &[WmJob], owner: usize) -> f32 {
    if let Some(i) = wm_job_find(jobs, owner, WM_JOB_TYPE_ANY) {
        let j = &jobs[i];
        if (j.flag & WM_JOB_PROGRESS) != 0 {
            return j.progress;
        }
    }
    0.0
}

/// `wm_jobs_update_progress_bars` (verbatim — mean of
/// running PROGRESS jobs).
pub fn wm_jobs_update_progress_bars(jobs: &[WmJob]) -> Option<f32> {
    let mut jobs_progress = 0.0f32;
    let mut total_progress = 0.0f32;
    for j in jobs {
        if j.running && !j.ready && (j.flag & WM_JOB_PROGRESS) != 0 {
            jobs_progress += 1.0;
            total_progress += j.progress;
        }
    }
    if jobs_progress > 0.0 {
        Some(total_progress / jobs_progress)
    } else {
        None
    }
}

/// `wm_jobs_test_suspend_stop` (verbatim — delay →
/// suspend; same-type/EXCL_RENDER arbitration; PRIORITY
/// stops others).
pub fn wm_jobs_test_suspend_stop(jobs: &mut [WmJob], test_idx: usize) {
    let mut suspend = false;
    if jobs[test_idx].start_delay_time > 0.0 {
        suspend = true;
        jobs[test_idx].start_delay_time = 0.0;
    } else {
        for i in 0..jobs.len() {
            if i == test_idx || !jobs[i].running {
                continue;
            }
            if (jobs[test_idx].flag & WM_JOB_EXCL_RENDER) == 0 {
                if jobs[i].job_type != jobs[test_idx].job_type {
                    continue;
                }
            }
            if (jobs[test_idx].flag & WM_JOB_EXCL_RENDER) != 0
                && (jobs[i].flag & WM_JOB_EXCL_RENDER) == 0
            {
                continue;
            }
            suspend = true;
            if (jobs[test_idx].flag & WM_JOB_PRIORITY) != 0 {
                jobs[i].stop = true;
            }
        }
    }
    jobs[test_idx].suspended = suspend;
}

/// `WM_jobs_start` (verbatim — running → stop; else
/// suspend-check → init + task-pool push + timer).
pub fn wm_jobs_start(
    jobs: &mut Vec<WmJob>,
    job_idx: usize,
    task_pool_push: &mut dyn FnMut(i32, usize),
    event_timer_add: &mut dyn FnMut(usize, f64) -> usize,
    event_timer_remove: &mut dyn FnMut(usize),
    time_now_seconds: &mut dyn FnMut() -> f64,
) {
    if jobs[job_idx].running {
        jobs[job_idx].stop = true;
        return;
    }
    if jobs[job_idx].customdata != 0 && jobs[job_idx].has_startjob {
        let time_step = if jobs[job_idx].start_delay_time > 0.0 {
            jobs[job_idx].start_delay_time
        } else {
            jobs[job_idx].time_step
        };
        wm_jobs_test_suspend_stop(jobs, job_idx);
        if !jobs[job_idx].suspended {
            let j = &mut jobs[job_idx];
            j.run_customdata = j.customdata;
            j.customdata = 0;
            j.running = true;
            j.stop = false;
            j.ready = false;
            j.progress = 0.0;
            let jt = j.job_type;
            (task_pool_push)(jt, j.run_customdata);
        }
        if jobs[job_idx].wt != 0 {
            (event_timer_remove)(jobs[job_idx].wt);
        }
        jobs[job_idx].wt = (event_timer_add)(jobs[job_idx].win, time_step);
        jobs[job_idx].start_time = (time_now_seconds)();
    }
}

/// `WM_jobs_is_running`/`WM_jobs_is_stopped` (verbatim).
pub fn wm_jobs_is_running(j: &WmJob) -> bool { j.running }
pub fn wm_jobs_is_stopped(j: &WmJob) -> bool { j.stop }

/// `WM_jobs_starttime` (verbatim).
pub fn wm_jobs_starttime(jobs: &[WmJob], owner: usize) -> f64 {
    if let Some(i) = wm_job_find(jobs, owner, WM_JOB_TYPE_ANY) {
        if (jobs[i].flag & WM_JOB_PROGRESS) != 0 {
            return jobs[i].start_time;
        }
    }
    0.0
}

/// `WM_jobs_name` (verbatim — owner lookup name).
pub fn wm_jobs_name<'a>(jobs: &'a [WmJob], owner: usize) -> Option<&'a str> {
    wm_job_find(jobs, owner, WM_JOB_TYPE_ANY).map(|i| jobs[i].name.as_str())
}

/// `WM_jobs_customdata_from_type` (verbatim).
pub fn wm_jobs_customdata_from_type(jobs: &[WmJob], owner: usize, job_type: i32) -> Option<usize> {
    wm_job_find(jobs, owner, job_type).map(|i| jobs[i].customdata)
}

/// `WM_jobs_kill_all`/`_except`/`_type`/`_from_owner`
/// filter selection (verbatim predicates).
pub fn jobs_matching_kill(jobs: &[WmJob], owner: usize, job_type: i32) -> Vec<usize> {
    jobs.iter()
        .enumerate()
        .filter(|(_, j)| {
            (owner == usize::MAX || j.owner == owner)
                && (job_type == WM_JOB_TYPE_ANY || j.job_type == job_type)
        })
        .map(|(i, _)| i)
        .collect()
}
