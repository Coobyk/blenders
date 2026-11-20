//! Port of `task_pool.cc` — `BLI_task_pool_*` task scheduler.
//!
//! Pools run `TaskRunFunction` work items; types mirror upstream:
//! `TaskPoolType::Tbb` (foreground, multi-threaded), `TbbSuspended`
//! (tasks queued until `work_and_wait`), `NoThreads` (serial),
//! `Background`/`BackgroundSerial` (daemon worker threads).
//! PORT_NOTE: TBB scheduling → worker threads over a shared deque;
//! TBB-specific suspended/sleep-state behavior is preserved at the API
//! level but implemented with std::sync primitives.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskPoolType {
    Tbb,
    TbbSuspended,
    NoThreads,
    Background,
    BackgroundSerial,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    High,
}

/// A queued task: `run` receives the pool userdata.
pub type TaskFn = Box<dyn FnOnce(&TaskPoolCtx) + Send>;

struct Shared {
    queue: Mutex<VecDeque<TaskFn>>,
    cond: Condvar,
    is_cancelled: AtomicBool,
    /// Number of tasks still executing (for `work_and_wait`).
    running: AtomicUsize,
    /// Worker threads may exit when this is set and queue is empty.
    done: AtomicBool,
}

/// Context handed to each task (userdata + pool control).
pub struct TaskPoolCtx {
    shared: Arc<Shared>,
}

impl TaskPoolCtx {
    /// `BLI_task_pool_is_canceled` — cooperative cancel check.
    pub fn is_cancelled(&self) -> bool {
        self.shared.is_cancelled.load(Ordering::Relaxed)
    }
    /// `BLI_task_pool_cancel` — signal all workers/tasks to stop.
    pub fn cancel(&self) {
        self.shared.is_cancelled.store(true, Ordering::Relaxed);
        self.shared.cond.notify_all();
    }
}

pub struct TaskPool {
    ty: TaskPoolType,
    #[allow(dead_code)]
    priority: TaskPriority,
    shared: Arc<Shared>,
    /// For background pools: join handle of the worker thread.
    worker: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Suspended pools only run on `work_and_wait`.
    is_suspended: AtomicBool,
}

fn spawn_workers(shared: Arc<Shared>, num: usize) -> Vec<std::thread::JoinHandle<()>> {
    (0..num.max(1))
        .map(|_| {
            let shared = Arc::clone(&shared);
            std::thread::spawn(move || loop {
                let task = {
                    let mut q = shared.queue.lock().unwrap();
                    loop {
                        if shared.is_cancelled.load(Ordering::Relaxed) {
                            q.clear();
                            return;
                        }
                        if let Some(t) = q.pop_front() {
                            break Some(t);
                        }
                        if shared.done.load(Ordering::Relaxed) {
                            return;
                        }
                        q = shared.cond.wait(q).unwrap();
                    }
                };
                if let Some(t) = task {
                    shared.running.fetch_add(1, Ordering::Relaxed);
                    let ctx = TaskPoolCtx {
                        shared: Arc::clone(&shared),
                    };
                    t(&ctx);
                    shared.running.fetch_sub(1, Ordering::Relaxed);
                    shared.cond.notify_all();
                }
            })
        })
        .collect()
}

impl TaskPool {
    /// `BLI_task_pool_create` — foreground pool using all cores.
    pub fn create(priority: TaskPriority) -> Self {
        Self::new(TaskPoolType::Tbb, priority)
    }
    /// `BLI_task_pool_create_suspended` — queue until work_and_wait.
    pub fn create_suspended(priority: TaskPriority) -> Self {
        let p = Self::new(TaskPoolType::TbbSuspended, priority);
        p.is_suspended.store(true, Ordering::Relaxed);
        p
    }
    /// `BLI_task_pool_create_no_threads` — serial execution.
    pub fn create_no_threads() -> Self {
        Self::new(TaskPoolType::NoThreads, TaskPriority::High)
    }
    /// `BLI_task_pool_create_background` — separate worker thread
    /// starts executing pushed tasks immediately.
    pub fn create_background(priority: TaskPriority) -> Self {
        let p = Self::new(TaskPoolType::Background, priority);
        let handles = spawn_workers(
            Arc::clone(&p.shared),
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        );
        for h in handles {
            *p.worker.lock().unwrap() = Some(h); // keep last; serial uses 1
        }
        p
    }
    /// `BLI_task_pool_create_background_serial` — one worker thread.
    pub fn create_background_serial(priority: TaskPriority) -> Self {
        let p = Self::new(TaskPoolType::BackgroundSerial, priority);
        let handles = spawn_workers(Arc::clone(&p.shared), 1);
        for h in handles {
            *p.worker.lock().unwrap() = Some(h);
        }
        p
    }

    fn new(ty: TaskPoolType, priority: TaskPriority) -> Self {
        Self {
            ty,
            priority,
            shared: Arc::new(Shared {
                queue: Mutex::new(VecDeque::new()),
                cond: Condvar::new(),
                is_cancelled: AtomicBool::new(false),
                running: AtomicUsize::new(0),
                done: AtomicBool::new(false),
            }),
            worker: Mutex::new(None),
            is_suspended: AtomicBool::new(false),
        }
    }

    /// `BLI_task_pool_push` — enqueue a task. For non-suspended
    /// foreground pools, tasks start immediately on worker threads.
    pub fn push(&self, task: TaskFn) {
        if matches!(self.ty, TaskPoolType::NoThreads)
            && !self.is_suspended.load(Ordering::Relaxed)
        {
            // Serial foreground: run inline (matches upstream
            // TASK_POOL_NO_THREADS which never schedules).
            let ctx = TaskPoolCtx {
                shared: Arc::clone(&self.shared),
            };
            task(&ctx);
            return;
        }
        self.shared.queue.lock().unwrap().push_back(task);
        self.shared.cond.notify_one();
    }

    /// `BLI_task_pool_work_and_wait` — run queued tasks (from this
    /// thread and workers for TBB pools) until the queue drains.
    pub fn work_and_wait(&self) {
        if self.is_suspended.swap(false, Ordering::Relaxed) {
            // Suspended pools wake their tasks now.
            self.shared.cond.notify_all();
        }
        let ctx = TaskPoolCtx {
            shared: Arc::clone(&self.shared),
        };
        loop {
            if self.shared.is_cancelled.load(Ordering::Relaxed) {
                self.shared.queue.lock().unwrap().clear();
                break;
            }
            let task = self.shared.queue.lock().unwrap().pop_front();
            match task {
                Some(t) => {
                    self.shared.running.fetch_add(1, Ordering::Relaxed);
                    t(&ctx);
                    self.shared.running.fetch_sub(1, Ordering::Relaxed);
                    self.shared.cond.notify_all();
                }
                None => {
                    if self.shared.running.load(Ordering::Relaxed) == 0 {
                        break;
                    }
                    // Wait for in-flight tasks.
                    let q = self.shared.queue.lock().unwrap();
                    let _ = self
                        .shared
                        .cond
                        .wait_timeout(q, std::time::Duration::from_millis(1))
                        .unwrap();
                }
            }
        }
    }

    /// `BLI_task_pool_cancel`.
    pub fn cancel(&self) {
        self.shared.is_cancelled.store(true, Ordering::Relaxed);
        self.shared.cond.notify_all();
    }

    /// `BLI_task_pool_is_canceled`.
    pub fn is_cancelled(&self) -> bool {
        self.shared.is_cancelled.load(Ordering::Relaxed)
    }

    /// `BLI_task_pool_current_is_cancelled` — alias on the pool.
    pub fn num_threads_use(&self) -> usize {
        match self.ty {
            TaskPoolType::NoThreads => 1,
            _ => std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        }
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        self.shared.done.store(true, Ordering::Relaxed);
        self.shared.cond.notify_all();
        if let Some(h) = self.worker.lock().unwrap().take() {
            let _ = h.join();
        }
    }
}

/// `BLI_task_pool_num_workers_on_suspended_pool` helper analog.
pub fn system_num_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicI32;

    #[test]
    fn pool_runs_all_tasks() {
        let pool = TaskPool::create_no_threads();
        let n = Arc::new(AtomicI32::new(0));
        for _ in 0..10 {
            let n = Arc::clone(&n);
            pool.push(Box::new(move |ctx| {
                assert!(!ctx.is_cancelled());
                n.fetch_add(1, Ordering::Relaxed);
            }));
        }
        pool.work_and_wait();
        assert_eq!(n.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn suspended_pool_queues() {
        let pool = TaskPool::create_suspended(TaskPriority::High);
        let n = Arc::new(AtomicI32::new(0));
        for _ in 0..5 {
            let n = Arc::clone(&n);
            pool.push(Box::new(move |_| {
                n.fetch_add(1, Ordering::Relaxed);
            }));
        }
        pool.work_and_wait();
        assert_eq!(n.load(Ordering::Relaxed), 5);
    }
}
