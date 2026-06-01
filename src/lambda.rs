//! Lambda calculus: terms, alpha/beta/eta reduction, normal forms, Church encodings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Lambda calculus term.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Term {
    /// Variable reference by name.
    Var(String),
    /// Lambda abstraction: λx.body
    Abs(String, Box<Term>),
    /// Application: (f x)
    App(Box<Term>, Box<Term>),
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(s) => write!(f, "{}", s),
            Term::Abs(x, body) => write!(f, "λ{}.{}", x, body),
            Term::App(g, x) => {
                let g_str = match g.as_ref() {
                    Term::Abs(_, _) => format!("({})", g),
                    _ => format!("{}", g),
                };
                let x_str = match x.as_ref() {
                    Term::App(_, _) | Term::Abs(_, _) => format!("({})", x),
                    _ => format!("{}", x),
                };
                write!(f, "{} {}", g_str, x_str)
            }
        }
    }
}

impl Term {
    pub fn var<S: Into<String>>(s: S) -> Self {
        Term::Var(s.into())
    }

    pub fn abs<S: Into<String>>(x: S, body: Term) -> Self {
        Term::Abs(x.into(), Box::new(body))
    }

    pub fn app(f: Term, x: Term) -> Self {
        Term::App(Box::new(f), Box::new(x))
    }
}

/// Free variables in a term.
pub fn free_vars(t: &Term) -> Vec<String> {
    let mut vars = Vec::new();
    free_vars_acc(t, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn free_vars_acc(t: &Term, acc: &mut Vec<String>) {
    match t {
        Term::Var(x) => {
            if !acc.contains(x) {
                acc.push(x.clone());
            }
        }
        Term::Abs(x, body) => {
            let mut inner = Vec::new();
            free_vars_acc(body, &mut inner);
            for v in inner {
                if v != *x && !acc.contains(&v) {
                    acc.push(v);
                }
            }
        }
        Term::App(f, x) => {
            free_vars_acc(f, acc);
            free_vars_acc(x, acc);
        }
    }
}

/// Substitute `sub` for variable `name` in term `t`.
pub fn substitute(t: &Term, name: &str, sub: &Term) -> Term {
    match t {
        Term::Var(x) if x == name => sub.clone(),
        Term::Var(_) => t.clone(),
        Term::Abs(x, body) if x == name => t.clone(),
        Term::Abs(x, body) => {
            let fv_sub = free_vars(sub);
            if fv_sub.contains(x) {
                // Capture-avoiding: rename bound var
                let fresh = fresh_var(x, &fv_sub, &free_vars(body));
                let new_body = substitute(body, x, &Term::var(&fresh));
                Term::abs(fresh, substitute(&new_body, name, sub))
            } else {
                Term::abs(x.clone(), substitute(body, name, sub))
            }
        }
        Term::App(f, x) => Term::app(substitute(f, name, sub), substitute(x, name, sub)),
    }
}

fn fresh_var(base: &str, avoid1: &[String], avoid2: &[String]) -> String {
    let mut i = 0;
    loop {
        let candidate = if i == 0 {
            format!("{}'", base)
        } else {
            format!("{}{}", base, i)
        };
        if !avoid1.contains(&candidate) && !avoid2.contains(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// Alpha-renaming: systematically rename bound variables.
pub fn alpha_rename(t: &Term, mapping: &HashMap<String, String>) -> Term {
    match t {
        Term::Var(x) => Term::var(mapping.get(x).cloned().unwrap_or_else(|| x.clone())),
        Term::Abs(x, body) => {
            let new_x = mapping.get(x).cloned().unwrap_or_else(|| x.clone());
            let mut new_mapping = mapping.clone();
            new_mapping.insert(x.clone(), new_x.clone());
            Term::abs(new_x, alpha_rename(body, &new_mapping))
        }
        Term::App(f, arg) => Term::app(alpha_rename(f, mapping), alpha_rename(arg, mapping)),
    }
}

/// One step of beta reduction (call-by-name, leftmost-outermost).
/// Returns None if no reduction is possible.
pub fn beta_step(t: &Term) -> Option<Term> {
    match t {
        Term::Var(_) => None,
        Term::Abs(x, body) => {
            let reduced = beta_step(body)?;
            Some(Term::abs(x.clone(), reduced))
        }
        Term::App(f, x) => match f.as_ref() {
            Term::Abs(param, body) => Some(substitute(body, param, x)),
            _ => {
                if let Some(rf) = beta_step(f) {
                    Some(Term::app(rf, x.as_ref().clone()))
                } else if let Some(rx) = beta_step(x) {
                    Some(Term::app(f.as_ref().clone(), rx))
                } else {
                    None
                }
            }
        },
    }
}

/// Beta-reduce to normal form with a fuel limit.
pub fn beta_normalize(t: &Term, fuel: usize) -> Term {
    let mut current = t.clone();
    for _ in 0..fuel {
        match beta_step(&current) {
            Some(next) => current = next,
            None => break,
        }
    }
    current
}

/// Eta-reduction: λx.f x => f when x not free in f.
pub fn eta_step(t: &Term) -> Option<Term> {
    match t {
        Term::Abs(x, body) => {
            if let Term::App(f, arg) = body.as_ref() {
                if let Term::Var(v) = arg.as_ref() {
                    if v == x && !free_vars(f).contains(x) {
                        return Some(f.as_ref().clone());
                    }
                }
            }
            let reduced = eta_step(body)?;
            Some(Term::abs(x.clone(), reduced))
        }
        Term::App(f, x) => {
            if let Some(rf) = eta_step(f) {
                Some(Term::app(rf, x.as_ref().clone()))
            } else {
                let rx = eta_step(x)?;
                Some(Term::app(f.as_ref().clone(), rx))
            }
        }
        _ => None,
    }
}

/// Check if a term is in beta normal form.
pub fn is_beta_normal(t: &Term) -> bool {
    beta_step(t).is_none()
}

// --- Church Encodings ---

/// Church numeral: λf.λx.f^n(x)
pub fn church_numeral(n: usize) -> Term {
    let mut body = Term::var("x");
    for _ in 0..n {
        body = Term::app(Term::var("f"), body);
    }
    Term::abs("f", Term::abs("x", body))
}

/// Church boolean true: λt.λf.t
pub fn church_true() -> Term {
    Term::abs("t", Term::abs("f", Term::var("t")))
}

/// Church boolean false: λt.λf.f
pub fn church_false() -> Term {
    Term::abs("t", Term::abs("f", Term::var("f")))
}

/// Church if: λc.λt.λe.c t e
pub fn church_if() -> Term {
    Term::abs("c", Term::abs("t", Term::abs("e", Term::app(Term::app(Term::var("c"), Term::var("t")), Term::var("e")))))
}

/// Church pair: λx.λy.λf.f x y
pub fn church_pair(x: Term, y: Term) -> Term {
    Term::abs(
        "f",
        Term::app(Term::app(Term::var("f"), x.clone()), y),
    )
}

/// Church fst: λp.p (λx.λy.x)
pub fn church_fst() -> Term {
    Term::abs("p", Term::app(Term::var("p"), Term::abs("x", Term::abs("y", Term::var("x")))))
}

/// Church snd: λp.p (λx.λy.y)
pub fn church_snd() -> Term {
    Term::abs("p", Term::app(Term::var("p"), Term::abs("x", Term::abs("y", Term::var("y")))))
}

/// Church successor: λn.λf.λx.f(n f x)
pub fn church_succ() -> Term {
    Term::abs("n", Term::abs("f", Term::abs("x", Term::app(Term::var("f"), Term::app(Term::app(Term::var("n"), Term::var("f")), Term::var("x"))))))
}

/// Church plus: λm.λn.λf.λx.m f (n f x)
pub fn church_plus() -> Term {
    Term::abs("m", Term::abs("n", Term::abs("f", Term::abs("x",
        Term::app(Term::app(Term::var("m"), Term::var("f")), Term::app(Term::app(Term::var("n"), Term::var("f")), Term::var("x")))
    ))))
}

/// Church multiply: λm.λn.λf.m (n f)
pub fn church_mult() -> Term {
    Term::abs("m", Term::abs("n", Term::abs("f", Term::app(Term::var("m"), Term::app(Term::var("n"), Term::var("f"))))))
}

/// Church Y combinator: λf.(λx.f(x x))(λx.f(x x))
pub fn y_combinator() -> Term {
    let xx = Term::app(Term::var("x"), Term::var("x"));
    Term::abs("f", Term::app(Term::abs("x", Term::app(Term::var("f"), xx.clone())), Term::abs("x", Term::app(Term::var("f"), xx))))
}

/// Decode a Church numeral to a natural number (by counting applications).
pub fn decode_church_numeral(t: &Term) -> Option<usize> {
    // Should be λf.λx.(body) where body has n applications of f
    match t {
        Term::Abs(_, body1) => match body1.as_ref() {
            Term::Abs(_, body2) => {
                let mut count = 0;
                let mut current = body2.as_ref();
                loop {
                    match current {
                        Term::App(f, arg) => {
                            if let Term::Var(name) = f.as_ref() {
                                if name == "f" {
                                    count += 1;
                                    current = arg.as_ref();
                                    continue;
                                }
                            }
                            return None;
                        }
                        Term::Var(name) if name == "x" => return Some(count),
                        _ => return None,
                    }
                }
            }
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_vars_var() {
        let t = Term::var("x");
        assert_eq!(free_vars(&t), vec!["x"]);
    }

    #[test]
    fn test_free_vars_abs_binds() {
        let t = Term::abs("x", Term::var("x"));
        assert!(free_vars(&t).is_empty());
    }

    #[test]
    fn test_free_vars_app() {
        let t = Term::app(Term::var("x"), Term::var("y"));
        let mut fv = free_vars(&t);
        fv.sort();
        assert_eq!(fv, vec!["x", "y"]);
    }

    #[test]
    fn test_substitute_simple() {
        let t = Term::abs("x", Term::app(Term::var("x"), Term::var("y")));
        let sub = Term::var("z");
        let result = substitute(&t, "y", &sub);
        match result {
            Term::Abs(_, body) => {
                let expected = Term::app(Term::var("x"), Term::var("z"));
                assert_eq!(*body, expected);
            }
            _ => panic!("expected abstraction"),
        }
    }

    #[test]
    fn test_substitute_avoids_capture() {
        let t = Term::abs("z", Term::var("y"));
        let sub = Term::var("z");
        let result = substitute(&t, "y", &sub);
        // Should rename bound var to avoid capturing z
        match result {
            Term::Abs(x, body) => {
                assert_ne!(x, "z");
                assert_eq!(*body, Term::var("z"));
            }
            _ => panic!("expected abstraction"),
        }
    }

    #[test]
    fn test_beta_step_identity() {
        // (λx.x) y => y
        let t = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        let result = beta_step(&t).unwrap();
        assert_eq!(result, Term::var("y"));
    }

    #[test]
    fn test_beta_normalize() {
        let t = Term::app(
            Term::abs("x", Term::var("x")),
            Term::app(
                Term::abs("y", Term::var("y")),
                Term::var("z"),
            ),
        );
        let result = beta_normalize(&t, 100);
        assert_eq!(result, Term::var("z"));
    }

    #[test]
    fn test_is_beta_normal() {
        assert!(is_beta_normal(&Term::var("x")));
        assert!(!is_beta_normal(&Term::app(Term::abs("x", Term::var("x")), Term::var("y"))));
    }

    #[test]
    fn test_eta_reduction() {
        // λx.f x => f
        let t = Term::abs("x", Term::app(Term::var("f"), Term::var("x")));
        let result = eta_step(&t).unwrap();
        assert_eq!(result, Term::var("f"));
    }

    #[test]
    fn test_eta_no_reduce_when_captured() {
        // λx.x x should NOT eta-reduce
        let t = Term::abs("x", Term::app(Term::var("x"), Term::var("x")));
        assert!(eta_step(&t).is_none());
    }

    #[test]
    fn test_church_zero() {
        let zero = church_numeral(0);
        assert_eq!(decode_church_numeral(&zero), Some(0));
    }

    #[test]
    fn test_church_three() {
        let three = church_numeral(3);
        assert_eq!(decode_church_numeral(&three), Some(3));
    }

    #[test]
    fn test_church_true_selects_first() {
        let t = Term::app(Term::app(church_true(), Term::var("a")), Term::var("b"));
        let result = beta_normalize(&t, 10);
        assert_eq!(result, Term::var("a"));
    }

    #[test]
    fn test_church_false_selects_second() {
        let t = Term::app(Term::app(church_false(), Term::var("a")), Term::var("b"));
        let result = beta_normalize(&t, 10);
        assert_eq!(result, Term::var("b"));
    }

    #[test]
    fn test_church_pair_fst() {
        let pair = church_pair(Term::var("a"), Term::var("b"));
        let t = Term::app(church_fst(), pair);
        let result = beta_normalize(&t, 100);
        assert_eq!(result, Term::var("a"));
    }

    #[test]
    fn test_church_pair_snd() {
        let pair = church_pair(Term::var("a"), Term::var("b"));
        let t = Term::app(church_snd(), pair);
        let result = beta_normalize(&t, 100);
        assert_eq!(result, Term::var("b"));
    }

    #[test]
    fn test_church_succ() {
        let two = church_numeral(2);
        let t = Term::app(church_succ(), two);
        let result = beta_normalize(&t, 100);
        assert_eq!(decode_church_numeral(&result), Some(3));
    }

    #[test]
    fn test_church_plus() {
        let two = church_numeral(2);
        let three = church_numeral(3);
        let t = Term::app(Term::app(church_plus(), two), three);
        let result = beta_normalize(&t, 200);
        assert_eq!(decode_church_numeral(&result), Some(5));
    }

    #[test]
    fn test_church_mult() {
        let two = church_numeral(2);
        let three = church_numeral(3);
        let t = Term::app(Term::app(church_mult(), two), three);
        let result = beta_normalize(&t, 200);
        assert_eq!(decode_church_numeral(&result), Some(6));
    }

    #[test]
    fn test_alpha_rename() {
        let t = Term::abs("x", Term::app(Term::var("x"), Term::var("y")));
        let mut m = HashMap::new();
        m.insert("x".into(), "z".into());
        let result = alpha_rename(&t, &m);
        assert_eq!(result, Term::abs("z", Term::app(Term::var("z"), Term::var("y"))));
    }

    #[test]
    fn test_display() {
        let t = Term::abs("x", Term::app(Term::var("x"), Term::var("y")));
        let s = format!("{}", t);
        assert!(s.contains("λ"));
    }
}
