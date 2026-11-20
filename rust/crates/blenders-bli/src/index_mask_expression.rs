//! Port of `index_mask_expression.cc` + `BLI_index_mask_expression.hh`
//! — lazy set-algebra over `IndexMask` terms.
//!
//! PORT_NOTE: upstream evaluates lazily via coarse-segment merging
//! and threaded range construction; here each op evaluates eagerly to
//! a sorted index list (identical results, simpler scheduling).

use crate::index_mask::IndexMask;

/// `Expr::Type` tree — `Atomic` wraps a materialized mask.
#[derive(Clone)]
pub enum Expr {
    Atomic(IndexMask),
    Union(Vec<Expr>),
    Intersection(Vec<Expr>),
    /// first minus the rest.
    Difference(Vec<Expr>),
}

/// Term variants accepted by `ExprBuilder` (`Term` variant upstream).
pub enum Term {
    Expr(Expr),
    Mask(IndexMask),
    Range(i64, i64),
}

fn term_to_expr(t: Term) -> Expr {
    match t {
        Term::Expr(e) => e,
        Term::Mask(m) => Expr::Atomic(m),
        Term::Range(start, size) => Expr::Atomic(IndexMask::from_range(start, size)),
    }
}

/// `ExprBuilder::merge` → UnionExpr.
pub fn merge(terms: Vec<Term>) -> Expr {
    Expr::Union(terms.into_iter().map(term_to_expr).collect())
}
/// `ExprBuilder::subtract` → DifferenceExpr.
pub fn subtract(main: Term, subtract_terms: Vec<Term>) -> Expr {
    let mut v = Vec::with_capacity(subtract_terms.len() + 1);
    v.push(term_to_expr(main));
    v.extend(subtract_terms.into_iter().map(term_to_expr));
    Expr::Difference(v)
}
/// `ExprBuilder::intersect` → IntersectionExpr.
pub fn intersect(terms: Vec<Term>) -> Expr {
    Expr::Intersection(terms.into_iter().map(term_to_expr).collect())
}

/// `evaluate_expression` — materialize the expression.
pub fn evaluate_expression(expr: &Expr) -> IndexMask {
    let indices = eval_sorted(expr);
    IndexMask::from_indices_i64(&indices)
}

fn eval_sorted(expr: &Expr) -> Vec<i64> {
    match expr {
        Expr::Atomic(m) => m.iter().collect(),
        Expr::Union(terms) => {
            // k-way merge of sorted lists.
            let mut lists: Vec<Vec<i64>> = terms.iter().map(eval_sorted).collect();
            let mut out = Vec::new();
            if lists.is_empty() {
                return out;
            }
            // Simple merge: concatenate + sort + dedup (the C does
            // segmented merges; same result).
            for l in lists.drain(..) {
                out.extend_from_slice(&l);
            }
            out.sort_unstable();
            out.dedup();
            out
        }
        Expr::Intersection(terms) => {
            if terms.is_empty() {
                return Vec::new();
            }
            let mut acc = eval_sorted(&terms[0]);
            for t in &terms[1..] {
                let other = eval_sorted(t);
                acc = intersect_sorted(&acc, &other);
                if acc.is_empty() {
                    break;
                }
            }
            acc
        }
        Expr::Difference(terms) => {
            if terms.is_empty() {
                return Vec::new();
            }
            let mut acc = eval_sorted(&terms[0]);
            for t in &terms[1..] {
                let other = eval_sorted(t);
                acc = difference_sorted(&acc, &other);
                if acc.is_empty() {
                    break;
                }
            }
            acc
        }
    }
}

fn intersect_sorted(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    out
}

fn difference_sorted(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() {
        while j < b.len() && b[j] < a[i] {
            j += 1;
        }
        if j == b.len() || b[j] != a[i] {
            out.push(a[i]);
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn union_intersect_subtract() {
        let a = IndexMask::from_range(0, 10); // 0..10
        let b = IndexMask::from_range(5, 10); // 5..15
        let u = evaluate_expression(&merge(vec![Term::Mask(a.clone()), Term::Mask(b.clone())]));
        assert_eq!(u.iter().collect::<Vec<_>>(), (0..15).collect::<Vec<_>>());
        let i = evaluate_expression(&intersect(vec![
            Term::Mask(a.clone()),
            Term::Mask(b.clone()),
        ]));
        assert_eq!(i.iter().collect::<Vec<_>>(), (5..10).collect::<Vec<_>>());
        let d = evaluate_expression(&subtract(Term::Mask(a), vec![Term::Mask(b)]));
        assert_eq!(d.iter().collect::<Vec<_>>(), (0..5).collect::<Vec<_>>());
    }
}
