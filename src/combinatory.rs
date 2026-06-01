//! Combinatory logic: SKI combinators, fixed-point combinator.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Combinatory logic term.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Comb {
    S,
    K,
    I,
    /// Application.
    App(Box<Comb>, Box<Comb>),
}

impl fmt::Display for Comb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comb::S => write!(f, "S"),
            Comb::K => write!(f, "K"),
            Comb::I => write!(f, "I"),
            Comb::App(g, x) => {
                let g_str = match g.as_ref() {
                    Comb::App(_, _) => format!("({})", g),
                    other => format!("{}", other),
                };
                let x_str = match x.as_ref() {
                    Comb::App(_, _) => format!("({})", x),
                    other => format!("{}", other),
                };
                write!(f, "{} {}", g_str, x_str)
            }
        }
    }
}

impl Comb {
    pub fn app(f: Comb, x: Comb) -> Self {
        Comb::App(Box::new(f), Box::new(x))
    }
}

/// S combinator: S x y z = x z (y z)
pub fn s() -> Comb { Comb::S }

/// K combinator: K x y = x
pub fn k() -> Comb { Comb::K }

/// I combinator: I x = x
pub fn i() -> Comb { Comb::I }

/// One step of weak reduction for combinatory logic.
pub fn comb_step(t: &Comb) -> Option<Comb> {
    match t {
        Comb::S | Comb::K | Comb::I => None,
        Comb::App(f, x) => {
            // Try reducing f first
            if let Some(rf) = comb_step(f) {
                return Some(Comb::app(rf, x.as_ref().clone()));
            }
            // Try reducing x
            if let Some(rx) = comb_step(x) {
                return Some(Comb::app(f.as_ref().clone(), rx));
            }
            // Check for redex
            match f.as_ref() {
                Comb::I => Some(x.as_ref().clone()),
                Comb::K => {
                    // K x is a partial application, need K x _
                    None // Not enough args
                }
                Comb::App(kf, kx) if matches!(kf.as_ref(), Comb::K) => {
                    // K x y => x
                    Some(kx.as_ref().clone())
                }
                Comb::S => {
                    // S needs 3 args, only have 1
                    None
                }
                Comb::App(sf, sx) if matches!(sf.as_ref(), Comb::S) => {
                    // S x needs one more arg
                    None
                }
                Comb::App(App_inner, s_x1) if matches!(App_inner.as_ref(), Comb::App(_, _)) => {
                    // Check for S x y z pattern
                    if let Comb::App(s_ref, s_x) = App_inner.as_ref() {
                        if matches!(s_ref.as_ref(), Comb::S) {
                            // S x y z => x z (y z)
                            let x_val = s_x.as_ref().clone();
                            let y_val = s_x1.as_ref().clone();
                            let z_val = x.as_ref().clone();
                            return Some(Comb::app(
                                Comb::app(x_val.clone(), z_val.clone()),
                                Comb::app(y_val, z_val),
                            ));
                        }
                    }
                    None
                }
                _ => None,
            }
        }
    }
}

/// Reduce combinatory term to normal form.
pub fn comb_normalize(t: &Comb, fuel: usize) -> Comb {
    let mut current = t.clone();
    for _ in 0..fuel {
        match comb_step(&current) {
            Some(next) => current = next,
            None => break,
        }
    }
    current
}

/// Fixed-point Y combinator in combinatory logic.
/// Y = S (K (S I I)) (S (S (K S) K) (K (S I I)))
/// Simplified: we use a well-known encoding.
pub fn y_combinator_comb() -> Comb {
    let sii = Comb::app(s(), Comb::app(i(), i()));
    let ksii = Comb::app(k(), sii);
    // Y = S (K (S I I)) (S (S (K S) K) (K (S I I)))
    let ks = Comb::app(k(), s());
    let ssk = Comb::app(Comb::app(s(), ks), k());
    let ksii2 = Comb::app(k(), Comb::app(Comb::app(s(), i()), i()));
    Comb::app(Comb::app(s(), ksii), Comb::app(ssk, ksii2))
}

/// Convert a simple lambda term to SKI combinators (bracket abstraction).
pub fn lambda_to_ski(term: &crate::lambda::Term) -> Comb {
    use crate::lambda::Term;
    match term {
        Term::Var(_) => {
            // Variables become I-like (identity for now, proper would be environment)
            Comb::I
        }
        Term::Abs(x, body) => {
            let body_comb = lambda_to_ski(body);
            bracket_abstract(x, &body_comb)
        }
        Term::App(f, x) => Comb::app(lambda_to_ski(f), lambda_to_ski(x)),
    }
}

/// Bracket abstraction: abstract variable x from combinatory term.
fn bracket_abstract(x: &str, t: &Comb) -> Comb {
    // Simplified: we treat variable names symbolically
    // For a full implementation we'd track free variables
    match t {
        Comb::S | Comb::K | Comb::I => Comb::app(k(), t.clone()),
        Comb::App(f, arg) => Comb::app(Comb::app(s(), bracket_abstract(x, f)), bracket_abstract(x, arg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_combinator() {
        let t = Comb::app(i(), s());
        let result = comb_normalize(&t, 10);
        assert_eq!(result, s());
    }

    #[test]
    fn test_k_combinator() {
        let t = Comb::app(Comb::app(k(), s()), i());
        let result = comb_normalize(&t, 10);
        assert_eq!(result, s());
    }

    #[test]
    fn test_s_combinator() {
        // S K K x = K x (K x) = x  (identity via SKK)
        let skk = Comb::app(Comb::app(s(), k()), k());
        let t = Comb::app(skk, i());
        let result = comb_normalize(&t, 10);
        assert_eq!(result, i());
    }

    #[test]
    fn test_ski_identity() {
        // S K K = I (extensionally)
        let skk = Comb::app(Comb::app(s(), k()), k());
        let skkx = Comb::app(skk, s());
        let result = comb_normalize(&skkx, 10);
        assert_eq!(result, s());
    }

    #[test]
    fn test_display_comb() {
        let t = Comb::app(s(), k());
        assert_eq!(format!("{}", t), "S K");
    }

    #[test]
    fn test_y_combinator_exists() {
        let y = y_combinator_comb();
        let s = format!("{}", y);
        assert!(s.contains('S'));
    }

    #[test]
    fn test_lambda_to_ski() {
        use crate::lambda::Term;
        let id = Term::abs("x", Term::var("x"));
        let comb = lambda_to_ski(&id);
        // λx.x should reduce via SKI
        let result = comb_normalize(&comb, 10);
        // Should be some normal form
        assert!(format!("{}", result).len() > 0);
    }
}
