//! Port of `string_search.cc` — `blender::string_search`: fuzzy string
//! search with word-group awareness (`damerau_levenshtein_distance`,
//! `get_fuzzy_match_errors`, `extract_normalized_words`,
//! `StringSearchBase`).

const UNUSED_WORD: i32 = -1;
/// `UI_MENU_ARROW_SEP_UNICODE` — group separator between menu sections.
const UNICODE_RIGHT_TRIANGLE: u32 = 0x25b8;

fn count_utf8_code_points(s: &str) -> usize {
    s.chars().count()
}

/// `damerau_levenshtein_distance` — Damerau–Levenshtein over UTF-8
/// code points, all costs 1.
pub fn damerau_levenshtein_distance(a: &str, b: &str) -> i32 {
    const DELETION_COST: i32 = 1;
    const INSERTION_COST: i32 = 1;
    const SUBSTITUTION_COST: i32 = 1;
    const TRANSPOSITION_COST: i32 = 1;

    let size_a = count_utf8_code_points(a);
    let size_b = count_utf8_code_points(b);
    let row_length = size_b + 1;
    let mut rows = vec![0i32; row_length * 3];
    {
        for i in 0..row_length {
            rows[row_length + i] = (i as i32) * INSERTION_COST;
        }
    }
    let (mut v0, mut v1, mut v2) = (0usize, row_length, 2 * row_length);
    let mut prev_unicode_a = 0u32;
    for (i, unicode_a) in a.chars().enumerate() {
        let unicode_a = unicode_a as u32;
        rows[v2] = (i as i32 + 1) * DELETION_COST;
        let mut prev_unicode_b = 0u32;
        for (j, unicode_b) in b.chars().enumerate() {
            let unicode_b = unicode_b as u32;
            let mut new_cost = (rows[v1 + j + 1] + DELETION_COST)
                .min(rows[v2 + j] + INSERTION_COST)
                .min(rows[v1 + j] + (unicode_a != unicode_b) as i32 * SUBSTITUTION_COST);
            if i > 0 && j > 0 && unicode_a == prev_unicode_b && prev_unicode_a == unicode_b {
                new_cost = new_cost.min(rows[v0 + j - 1] + TRANSPOSITION_COST);
            }
            rows[v2 + j + 1] = new_cost;
            prev_unicode_b = unicode_b;
        }
        std::mem::swap(&mut v0, &mut v1);
        std::mem::swap(&mut v1, &mut v2);
        prev_unicode_a = unicode_a;
    }
    rows[v1 + size_b]
}

/// `get_fuzzy_match_errors` — minimum edit distance of `query` against
/// the best sliding window of `full`, or -1.
pub fn get_fuzzy_match_errors(query: &str, full: &str) -> i32 {
    if full.contains(query) {
        return 0;
    }
    let query_size = count_utf8_code_points(query);
    let full_size = count_utf8_code_points(full);
    if query_size == 1 {
        return -1;
    }
    let max_errors = if query_size <= 1 { 0 } else { query_size / 8 + 1 };
    if query_size as i64 - full_size as i64 > max_errors as i64 {
        return -1;
    }
    let mut qit = query.chars();
    let query_first_unicode = qit.next().unwrap() as u32;
    let query_second_unicode = qit.next().unwrap() as u32;

    let full_chars: Vec<char> = full.chars().collect();
    let window_size = (query_size + max_errors).min(full_size);
    let extra_chars = (window_size as i32 - query_size as i32).max(0);
    let max_acceptable_distance = (max_errors + extra_chars as usize) as i32;

    // Byte-offset windows of `window_size` code points.
    let byte_off = |ci: usize| -> usize { full.char_indices().nth(ci).map(|(o, _)| o).unwrap_or(full.len()) };
    let mut wb_ci = 0usize; // window begin char index
    loop {
        let we_ci = wb_ci + window_size;
        if we_ci > full_size {
            return -1;
        }
        let window = &full[byte_off(wb_ci)..byte_off(we_ci)];
        let window_begin_unicode = window.chars().next().unwrap() as u32;
        let mut distance = 0;
        if window_begin_unicode == query_first_unicode
            || window_begin_unicode == query_second_unicode
        {
            distance = damerau_levenshtein_distance(query, window);
            if distance <= max_acceptable_distance {
                return distance;
            }
        }
        if we_ci == full_size {
            return -1;
        }
        let window_offset = (distance / 2).max(1) as usize;
        wb_ci += window_offset.min(full_size - we_ci);
        if wb_ci + window_size > full_size {
            return -1;
        }
    }
}

/// `SearchItem` — normalized words + group ids for one added string.
pub struct SearchItem<T> {
    pub user_data: T,
    normalized_words: Vec<String>,
    word_group_ids: Vec<i32>,
    main_group_id: i32,
    main_group_length: usize,
    total_length: usize,
    weight: f32,
    recent_time: i64,
    is_deprecated: bool,
}

struct InitialsMatch {
    matched_word_indices: Vec<usize>,
}

impl InitialsMatch {
    fn count_main_group_matches<T>(&self, item: &SearchItem<T>) -> usize {
        self.matched_word_indices
            .iter()
            .filter(|&&i| item.word_group_ids[i] == item.main_group_id)
            .count()
    }
    fn better_than<T>(&self, other: &InitialsMatch, item: &SearchItem<T>) -> bool {
        self.count_main_group_matches(item) > other.count_main_group_matches(item)
    }
}

fn match_word_initials<T>(
    query: &str,
    item: &SearchItem<T>,
    word_match_map: &[i32],
    start: usize,
) -> Option<InitialsMatch> {
    let words = &item.normalized_words;
    if start >= words.len() {
        return None;
    }
    let mut matched: Vec<usize> = Vec::new();
    let mut first_found_word_index: Option<usize> = None;
    let mut word_index = start;
    let mut char_iter = words[start].chars().peekable();
    let mut char_consumed = true;
    for qc in query.chars() {
        loop {
            if word_index >= words.len() {
                if let Some(f) = first_found_word_index {
                    return match_word_initials(query, item, word_match_map, f + 1);
                }
                return None;
            }
            if word_match_map[word_index] != UNUSED_WORD {
                word_index += 1;
                if word_index < words.len() {
                    char_iter = words[word_index].chars().peekable();
                }
                char_consumed = true;
                continue;
            }
            if !char_consumed {
                if let Some(&cc) = char_iter.peek() {
                    char_iter.next();
                    if cc == qc {
                        matched.push(word_index);
                        if first_found_word_index.is_none() {
                            first_found_word_index = Some(word_index);
                        }
                        break;
                    }
                }
                char_consumed = true;
            } else {
                char_iter = words[word_index].chars().peekable();
                char_consumed = false;
                continue;
            }
            word_index += 1;
        }
    }
    if let Some(f) = first_found_word_index {
        if let Some(sub) = match_word_initials(query, item, word_match_map, f + 1) {
            let m = InitialsMatch {
                matched_word_indices: matched,
            };
            if sub.better_than(&m, item) {
                return Some(sub);
            }
            return Some(m);
        }
    }
    Some(InitialsMatch {
        matched_word_indices: matched,
    })
}

fn get_best_word_index_that_startswith<T>(
    query: &str,
    item: &SearchItem<T>,
    word_match_map: &[i32],
    remaining_query_words: &[String],
) -> Option<usize> {
    let use_shortest_match = remaining_query_words
        .iter()
        .any(|w| w.starts_with(query));
    let mut best_word_size = usize::MAX;
    let mut best_word_index = None;
    let mut best_word_in_main_group = false;
    for (i, word) in item.normalized_words.iter().enumerate() {
        if word_match_map[i] != UNUSED_WORD {
            continue;
        }
        let word_in_main_group = item.word_group_ids[i] == item.main_group_id;
        if word.starts_with(query) {
            let found_new_best = if use_shortest_match {
                word.len() < best_word_size
            } else {
                !best_word_in_main_group
            };
            if found_new_best {
                best_word_index = Some(i);
                best_word_size = word.len();
                best_word_in_main_group = word_in_main_group;
            }
        }
    }
    best_word_index
}

fn get_word_index_that_fuzzy_matches(
    query: &str,
    words: &[String],
    word_match_map: &[i32],
) -> Option<usize> {
    for (i, word) in words.iter().enumerate() {
        if word_match_map[i] != UNUSED_WORD {
            continue;
        }
        if get_fuzzy_match_errors(query, word) >= 0 {
            return Some(i);
        }
    }
    None
}

fn score_query_against_words<T>(query_words: &[String], item: &SearchItem<T>) -> Option<f32> {
    let mut word_match_map = vec![UNUSED_WORD; item.normalized_words.len()];
    let mut total_match_score = if item.is_deprecated { 500.0 } else { 1000.0 };
    for (qi, query_word) in query_words.iter().enumerate() {
        if let Some(word_index) = get_best_word_index_that_startswith(
            query_word,
            item,
            &word_match_map,
            &query_words[qi + 1..],
        ) {
            let is_main_group = item.word_group_ids[word_index] == item.main_group_id;
            total_match_score += if is_main_group { 10.0 } else { 9.0 };
            word_match_map[word_index] = qi as i32;
            continue;
        }
        if let Some(m) = match_word_initials(query_word, item, &word_match_map, 0) {
            let all_main = m.count_main_group_matches(item) == m.matched_word_indices.len();
            total_match_score += if all_main { 4.0 } else { 3.0 };
            for i in m.matched_word_indices {
                word_match_map[i] = qi as i32;
            }
            continue;
        }
        if let Some(word_index) =
            get_word_index_that_fuzzy_matches(query_word, &item.normalized_words, &word_match_map)
        {
            let error_count = get_fuzzy_match_errors(query_word, &item.normalized_words[word_index]);
            total_match_score += 3.0 - error_count as f32;
            word_match_map[word_index] = qi as i32;
            continue;
        }
        return None;
    }
    let match_indices: Vec<i32> = word_match_map
        .iter()
        .copied()
        .filter(|&i| i != UNUSED_WORD)
        .collect();
    if !match_indices.is_empty() {
        for i in 0..match_indices.len() - 1 {
            if match_indices[i] > match_indices[i + 1] {
                total_match_score -= 1.0;
            }
        }
    }
    Some(total_match_score)
}

/// `extract_normalized_words` — split `str` into lowercase words,
/// tracking `▸` section group ids.
pub fn extract_normalized_words(s: &str) -> (Vec<String>, Vec<i32>) {
    let is_separator = |u: u32| -> bool {
        matches!(u, 0x20 | 0x2d | 0x5f | 0x2f) || u == UNICODE_RIGHT_TRIANGLE
    };
    let lowered = s.to_lowercase();
    let mut words = Vec::new();
    let mut group_ids = Vec::new();
    let mut group_id = 0i32;
    let mut word = String::new();
    for c in s.chars() {
        let lc = c.to_lowercase().next().unwrap_or(c);
        let _ = &lowered;
        if is_separator(c as u32) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
                group_ids.push(group_id);
            }
        } else {
            word.push(lc);
        }
        if c as u32 == UNICODE_RIGHT_TRIANGLE {
            group_id += 1;
        }
    }
    if !word.is_empty() {
        words.push(word);
        group_ids.push(group_id);
    }
    (words, group_ids)
}

/// `MainWordsHeuristic` — which group counts as "main".
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MainWordsHeuristic {
    FirstGroup,
    LastGroup,
    All,
}

/// `StringSearchBase` — generic over the stored `user_data`.
pub struct StringSearchBase<T> {
    items: Vec<SearchItem<T>>,
    main_words_heuristic: MainWordsHeuristic,
}

impl<T: Copy> StringSearchBase<T> {
    /// `StringSearchBase()` — `MainWordsHeuristic::FirstGroup` default.
    pub fn new() -> Self {
        StringSearchBase {
            items: Vec::new(),
            main_words_heuristic: MainWordsHeuristic::FirstGroup,
        }
    }
    /// `set_main_words_heuristic`.
    pub fn set_main_words_heuristic(&mut self, h: MainWordsHeuristic) {
        self.main_words_heuristic = h;
    }
    /// `add_impl(str, user_data, weight)`.
    pub fn add(&mut self, s: &str, user_data: T, weight: f32) {
        let (words, mut word_group_ids) = extract_normalized_words(s);
        let recent_time = -1i64;
        let mut main_group_id = 0;
        if !word_group_ids.is_empty() {
            match self.main_words_heuristic {
                MainWordsHeuristic::FirstGroup => main_group_id = 0,
                MainWordsHeuristic::LastGroup => main_group_id = *word_group_ids.last().unwrap(),
                MainWordsHeuristic::All => {
                    main_group_id = 0;
                    word_group_ids.fill(0);
                }
            }
        }
        let main_group_length: usize = words
            .iter()
            .zip(&word_group_ids)
            .filter(|(_, g)| **g == main_group_id)
            .map(|(w, _)| w.len())
            .sum();
        let is_deprecated = s.contains("eprecated");
        self.items.push(SearchItem {
            user_data,
            normalized_words: words,
            word_group_ids,
            main_group_id,
            main_group_length,
            total_length: s.len(),
            weight,
            recent_time,
            is_deprecated,
        });
    }
    /// `query_impl(query)` — matching `user_data` sorted by score.
    pub fn query(&self, query: &str) -> Vec<T> {
        let (query_words, _) = extract_normalized_words(query);
        let mut scored: Vec<(f32, usize)> = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            if let Some(score) = score_query_against_words(&query_words, item) {
                scored.push((score, i));
            }
        }
        // group by score descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let mut out = Vec::new();
        let mut pos = 0;
        while pos < scored.len() {
            let score = scored[pos].0;
            let mut end = pos;
            while end < scored.len() && scored[end].0 == score {
                end += 1;
            }
            let mut group: Vec<usize> = scored[pos..end].iter().map(|&(_, i)| i).collect();
            if pos == 0 && !query.is_empty() {
                group.sort_by(|&a, &b| {
                    let ia = &self.items[a];
                    let ib = &self.items[b];
                    ia.main_group_length
                        .cmp(&ib.main_group_length)
                        .then(ia.total_length.cmp(&ib.total_length))
                });
                let mut stable = group.clone();
                stable.sort_by(|&a, &b| {
                    self.items[b]
                        .weight
                        .partial_cmp(&self.items[a].weight)
                        .unwrap()
                });
                group = stable;
            }
            if pos == 0 && query.chars().count() <= 1 {
                let mut stable = group.clone();
                stable.sort_by(|&a, &b| self.items[b].recent_time.cmp(&self.items[a].recent_time));
                group = stable;
            }
            out.extend(group.iter().map(|&i| self.items[i].user_data));
            pos = end;
        }
        out
    }
    /// `items_.size()`.
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: Copy> Default for StringSearchBase<T> {
    fn default() -> Self {
        Self::new()
    }
}
