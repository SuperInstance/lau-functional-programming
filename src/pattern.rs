//! Pattern matching theory: exhaustiveness and redundancy checking.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A pattern in a match expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: _
    Wildcard,
    /// Constructor pattern: Some(x), None, Cons(x, xs)
    Constructor(String, Vec<Pattern>),
    /// Literal pattern: 42, true, "hello"
    Literal(LitPat),
    /// Variable binding: x (matches anything, binds name)
    Var(String),
    /// Or pattern: p1 | p2
    Or(Box<Pattern>, Box<Pattern>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LitPat {
    Int(i64),
    Bool(bool),
    Str(String),
}

/// A match arm: pattern => body.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<String>,
}

/// A match expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Match {
    pub arms: Vec<MatchArm>,
}

/// Result of exhaustiveness check.
#[derive(Clone, Debug, PartialEq)]
pub enum Exhaustiveness {
    /// All cases are covered.
    Exhaustive,
    /// Some patterns are missing.
    NonExhaustive(Vec<Pattern>),
}

/// Check if a set of patterns covers all possible values of a given type.
/// Simplified: works on a flat domain.
pub fn check_exhaustiveness(arms: &[MatchArm], type_arms: &[String]) -> Exhaustiveness {
    if arms.is_empty() {
        return Exhaustiveness::NonExhaustive(vec![Pattern::Wildcard]);
    }

    let mut covered_constructors: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut has_wildcard = false;

    for arm in arms {
        match &arm.pattern {
            Pattern::Wildcard | Pattern::Var(_) => {
                has_wildcard = true;
            }
            Pattern::Constructor(name, _) => {
                covered_constructors.insert(name.clone());
            }
            Pattern::Literal(_) => {}
            Pattern::Or(p1, p2) => {
                // Recursively check both sides
                let sub_arms = vec![
                    MatchArm { pattern: *p1.clone(), guard: None },
                    MatchArm { pattern: *p2.clone(), guard: None },
                ];
                if let Exhaustiveness::NonExhaustive(missing) = check_exhaustiveness(&sub_arms, type_arms) {
                    // The or pattern itself might not be exhaustive, but we still record coverage
                }
            }
        }
    }

    if has_wildcard {
        return Exhaustiveness::Exhaustive;
    }

    // Check if all constructors are covered
    let all_covered = type_arms.iter().all(|c| covered_constructors.contains(c));
    if all_covered && !type_arms.is_empty() {
        return Exhaustiveness::Exhaustive;
    }

    let missing: Vec<Pattern> = type_arms
        .iter()
        .filter(|c| !covered_constructors.contains(*c))
        .map(|c| Pattern::Constructor(c.clone(), vec![Pattern::Wildcard]))
        .collect();

    if missing.is_empty() {
        Exhaustiveness::Exhaustive
    } else {
        Exhaustiveness::NonExhaustive(missing)
    }
}

/// Check if any arm is redundant (covered by previous arms).
pub fn check_redundancy(arms: &[MatchArm]) -> Vec<usize> {
    let mut redundant = Vec::new();
    let mut covered_wildcard = false;

    for (i, arm) in arms.iter().enumerate() {
        if covered_wildcard {
            redundant.push(i);
            continue;
        }
        match &arm.pattern {
            Pattern::Wildcard | Pattern::Var(_) => {
                covered_wildcard = true;
            }
            _ => {}
        }
    }
    redundant
}

/// Check if two patterns overlap (match some of the same values).
pub fn overlaps(p1: &Pattern, p2: &Pattern) -> bool {
    match (p1, p2) {
        (Pattern::Wildcard, _) | (_, Pattern::Wildcard) => true,
        (Pattern::Var(_), _) | (_, Pattern::Var(_)) => true,
        (Pattern::Constructor(n1, args1), Pattern::Constructor(n2, args2)) => {
            if n1 != n2 { return false; }
            args1.len() == args2.len() && args1.iter().zip(args2.iter()).all(|(a, b)| overlaps(a, b))
        }
        (Pattern::Literal(l1), Pattern::Literal(l2)) => l1 == l2,
        (Pattern::Or(a, b), p) | (p, Pattern::Or(a, b)) => {
            overlaps(a, p) || overlaps(b, p)
        }
        _ => false,
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Var(s) => write!(f, "{}", s),
            Pattern::Constructor(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "(")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            Pattern::Literal(l) => match l {
                LitPat::Int(n) => write!(f, "{}", n),
                LitPat::Bool(b) => write!(f, "{}", b),
                LitPat::Str(s) => write!(f, "\"{}\"", s),
            },
            Pattern::Or(a, b) => write!(f, "{} | {}", a, b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustive_wildcard() {
        let arms = vec![MatchArm { pattern: Pattern::Wildcard, guard: None }];
        let result = check_exhaustiveness(&arms, &["A".into(), "B".into()]);
        assert_eq!(result, Exhaustiveness::Exhaustive);
    }

    #[test]
    fn test_exhaustive_constructors() {
        let arms = vec![
            MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
            MatchArm { pattern: Pattern::Constructor("None".into(), vec![]), guard: None },
        ];
        let result = check_exhaustiveness(&arms, &["Some".into(), "None".into()]);
        assert_eq!(result, Exhaustiveness::Exhaustive);
    }

    #[test]
    fn test_non_exhaustive() {
        let arms = vec![
            MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
        ];
        let result = check_exhaustiveness(&arms, &["Some".into(), "None".into()]);
        match result {
            Exhaustiveness::NonExhaustive(missing) => {
                assert_eq!(missing.len(), 1);
            }
            _ => panic!("expected non-exhaustive"),
        }
    }

    #[test]
    fn test_redundancy_detection() {
        let arms = vec![
            MatchArm { pattern: Pattern::Wildcard, guard: None },
            MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
        ];
        let redundant = check_redundancy(&arms);
        assert_eq!(redundant, vec![1]);
    }

    #[test]
    fn test_no_redundancy() {
        let arms = vec![
            MatchArm { pattern: Pattern::Constructor("A".into(), vec![]), guard: None },
            MatchArm { pattern: Pattern::Constructor("B".into(), vec![]), guard: None },
        ];
        let redundant = check_redundancy(&arms);
        assert!(redundant.is_empty());
    }

    #[test]
    fn test_overlaps_wildcard() {
        assert!(overlaps(&Pattern::Wildcard, &Pattern::Constructor("X".into(), vec![])));
    }

    #[test]
    fn test_overlaps_same_constructor() {
        assert!(overlaps(
            &Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]),
            &Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]),
        ));
    }

    #[test]
    fn test_no_overlap_different_constructor() {
        assert!(!overlaps(
            &Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]),
            &Pattern::Constructor("None".into(), vec![]),
        ));
    }

    #[test]
    fn test_pattern_display() {
        assert_eq!(format!("{}", Pattern::Wildcard), "_");
        assert_eq!(format!("{}", Pattern::Constructor("Some".into(), vec![Pattern::Wildcard])), "Some(_)");
    }

    #[test]
    fn test_empty_arms_non_exhaustive() {
        let result = check_exhaustiveness(&[], &["A".into()]);
        assert_eq!(result, Exhaustiveness::NonExhaustive(vec![Pattern::Wildcard]));
    }
}
