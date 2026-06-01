//! Recursion schemes: catamorphism, anamorphism, hylomorphism, paramorphism
//! on fixed-point types.

use serde::{Deserialize, Serialize};

/// Fixed-point type: μf. f
/// This ties the recursive knot for any functor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fix<F> {
    pub unfix: Box<F>,
}

impl<F> Fix<F> {
    pub fn new(f: F) -> Self {
        Fix { unfix: Box::new(f) }
    }
}

/// A simple expression functor for demonstration.
/// ExprF a = Add a a | Mul a a | Val i64
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExprF<A> {
    Val(i64),
    Add(A, A),
    Mul(A, A),
}

impl<A: Clone> ExprF<A> {
    /// Map over the recursive positions.
    pub fn fmap<B>(&self, f: impl Fn(&A) -> B) -> ExprF<B> {
        match self {
            ExprF::Val(n) => ExprF::Val(*n),
            ExprF::Add(a, b) => ExprF::Add(f(a), f(b)),
            ExprF::Mul(a, b) => ExprF::Mul(f(a), f(b)),
        }
    }
}

/// A list functor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ListF<A, B> {
    Nil,
    Cons(A, B),
}

impl<A: Clone, B: Clone> ListF<A, B> {
    pub fn fmap<C>(&self, f: impl Fn(&B) -> C) -> ListF<A, C> {
        match self {
            ListF::Nil => ListF::Nil,
            ListF::Cons(a, b) => ListF::Cons(a.clone(), f(b)),
        }
    }
}

/// Simple recursive expression type (the fixed point unrolled).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Val(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn val(n: i64) -> Self { Expr::Val(n) }
    pub fn add(a: Expr, b: Expr) -> Self { Expr::Add(Box::new(a), Box::new(b)) }
    pub fn mul(a: Expr, b: Expr) -> Self { Expr::Mul(Box::new(a), Box::new(b)) }
}

/// Evaluate an expression tree.
pub fn eval_expr(expr: &Expr) -> i64 {
    match expr {
        Expr::Val(n) => *n,
        Expr::Add(a, b) => eval_expr(a) + eval_expr(b),
        Expr::Mul(a, b) => eval_expr(a) * eval_expr(b),
    }
}

/// Pretty-print an expression.
pub fn show_expr(expr: &Expr) -> String {
    match expr {
        Expr::Val(n) => format!("{}", n),
        Expr::Add(a, b) => format!("({} + {})", show_expr(a), show_expr(b)),
        Expr::Mul(a, b) => format!("({} * {})", show_expr(a), show_expr(b)),
    }
}

// --- General recursion schemes on Expr ---

/// Catamorphism on Expr: fold bottom-up.
pub fn cata_expr_general<A>(
    f: &dyn Fn(&ExprF<A>) -> A,
    expr: &Expr,
) -> A {
    match expr {
        Expr::Val(n) => f(&ExprF::Val(*n)),
        Expr::Add(a, b) => {
            let ra = cata_expr_general(f, a);
            let rb = cata_expr_general(f, b);
            f(&ExprF::Add(ra, rb))
        }
        Expr::Mul(a, b) => {
            let ra = cata_expr_general(f, a);
            let rb = cata_expr_general(f, b);
            f(&ExprF::Mul(ra, rb))
        }
    }
}

/// Anamorphism on Expr: unfold top-down.
/// Uses i64 seeds for concrete type to avoid recursion limit issues.
pub fn ana_expr(f: &dyn Fn(&i64) -> ExprF<i64>, seed: &i64) -> Expr {
    match f(seed) {
        ExprF::Val(n) => Expr::Val(n),
        ExprF::Add(a, b) => Expr::add(ana_expr(f, &a), ana_expr(f, &b)),
        ExprF::Mul(a, b) => Expr::mul(ana_expr(f, &a), ana_expr(f, &b)),
    }
}

/// Hylomorphism: ana then cata (unfold then fold).
pub fn hylo_expr(
    unfold: impl Fn(&i64) -> ExprF<i64>,
    fold: impl Fn(&ExprF<i64>) -> i64,
    seed: &i64,
) -> i64 {
    let intermediate = ana_expr(&unfold, seed);
    cata_expr_general(&fold, &intermediate)
}

/// Paramorphism on Expr: fold with access to original subexpressions.
pub fn para_expr<A>(
    f: &dyn Fn(&ExprF<(Expr, A)>) -> A,
    expr: &Expr,
) -> A {
    match expr {
        Expr::Val(n) => f(&ExprF::Val(*n)),
        Expr::Add(a, b) => {
            let ra = (a.as_ref().clone(), para_expr(f, a));
            let rb = (b.as_ref().clone(), para_expr(f, b));
            f(&ExprF::Add(ra, rb))
        }
        Expr::Mul(a, b) => {
            let ra = (a.as_ref().clone(), para_expr(f, a));
            let rb = (b.as_ref().clone(), para_expr(f, b));
            f(&ExprF::Mul(ra, rb))
        }
    }
}

/// Apomorphism: anamorphism with short-circuiting.
pub fn apo_expr(f: &dyn Fn(&i64) -> ExprF<Result<Expr, i64>>, seed: &i64) -> Expr {
    match f(seed) {
        ExprF::Val(n) => Expr::Val(n),
        ExprF::Add(Ok(a), Ok(b)) => Expr::add(a, b),
        ExprF::Add(Ok(a), Err(b)) => Expr::add(a, apo_expr(f, &b)),
        ExprF::Add(Err(a), Ok(b)) => Expr::add(apo_expr(f, &a), b),
        ExprF::Add(Err(a), Err(b)) => Expr::add(apo_expr(f, &a), apo_expr(f, &b)),
        ExprF::Mul(Ok(a), Ok(b)) => Expr::mul(a, b),
        ExprF::Mul(Ok(a), Err(b)) => Expr::mul(a, apo_expr(f, &b)),
        ExprF::Mul(Err(a), Ok(b)) => Expr::mul(apo_expr(f, &a), b),
        ExprF::Mul(Err(a), Err(b)) => Expr::mul(apo_expr(f, &a), apo_expr(f, &b)),
    }
}

// --- List recursion schemes ---

/// A recursive list type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum List<A> {
    Nil,
    Cons(A, Box<List<A>>),
}

impl<A: Clone> List<A> {
    pub fn nil() -> Self { List::Nil }
    pub fn cons(a: A, tail: List<A>) -> Self { List::Cons(a, Box::new(tail)) }

    pub fn from_vec(v: Vec<A>) -> Self {
        v.into_iter().rev().fold(List::Nil, |acc, a| List::cons(a, acc))
    }

    pub fn to_vec(&self) -> Vec<A> {
        let mut v = Vec::new();
        let mut current = self;
        loop {
            match current {
                List::Nil => break,
                List::Cons(a, tail) => {
                    v.push(a.clone());
                    current = tail;
                }
            }
        }
        v
    }
}

/// Catamorphism on List: fold.
pub fn cata_list<A, B: Clone>(
    nil: B,
    cons: &dyn Fn(&A, B) -> B,
    list: &List<A>,
) -> B {
    match list {
        List::Nil => nil,
        List::Cons(a, tail) => cons(a, cata_list(nil, cons, tail)),
    }
}

/// Anamorphism on List: unfold.
pub fn ana_list<A: Clone + 'static, B>(
    pred: &dyn Fn(&B) -> Option<(A, B)>,
    seed: &B,
) -> List<A> {
    match pred(seed) {
        None => List::Nil,
        Some((a, next)) => List::cons(a, ana_list(pred, &next)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_expr() {
        let expr = Expr::add(Expr::mul(Expr::val(2), Expr::val(3)), Expr::val(4));
        assert_eq!(eval_expr(&expr), 10);
    }

    #[test]
    fn test_show_expr() {
        let expr = Expr::add(Expr::val(1), Expr::val(2));
        assert_eq!(show_expr(&expr), "(1 + 2)");
    }

    #[test]
    fn test_cata_expr_sum() {
        let expr = Expr::add(Expr::val(1), Expr::add(Expr::val(2), Expr::val(3)));
        let sum = cata_expr_general(
            &|e| match e {
                ExprF::Val(n) => *n,
                ExprF::Add(a, b) => a + b,
                ExprF::Mul(a, b) => a * b,
            },
            &expr,
        );
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_cata_expr_size() {
        let expr = Expr::add(Expr::val(1), Expr::mul(Expr::val(2), Expr::val(3)));
        let size = cata_expr_general(
            &|e| match e {
                ExprF::Val(_) => 1,
                ExprF::Add(a, b) | ExprF::Mul(a, b) => 1 + a + b,
            },
            &expr,
        );
        assert_eq!(size, 5); // 3 vals + add + mul
    }

    #[test]
    fn test_ana_expr() {
        // Build expression: 1 + (2 + 3) from seed 6
        let expr = ana_expr(
            &|n: &i64| {
                if *n == 1 {
                    ExprF::Val(1)
                } else if *n == 5 {
                    ExprF::Add(2, 3)
                } else if *n == 2 {
                    ExprF::Val(2)
                } else if *n == 3 {
                    ExprF::Val(3)
                } else {
                    ExprF::Add(1, 5)
                }
            },
            &6,
        );
        assert_eq!(eval_expr(&expr), 6); // 1 + (2 + 3) = 6
    }

    #[test]
    fn test_hylo_expr() {
        // Hylomorphism: unfold then fold a flat expression
        let result = hylo_expr(
            |n: &i64| {
                match *n {
                    0 => ExprF::Add(1, 2),
                    1 | 2 => ExprF::Val(*n),
                    _ => ExprF::Val(0),
                }
            },
            |e| match e {
                ExprF::Val(n) => *n,
                ExprF::Add(a, b) => a + b,
                ExprF::Mul(a, b) => a * b,
            },
            &0i64,
        );
        assert_eq!(result, 3);
    }

    #[test]
    fn test_para_expr() {
        let expr = Expr::add(Expr::val(1), Expr::val(2));
        let count = para_expr(
            &|e| match e {
                ExprF::Val(_) => 1,
                ExprF::Add((_, a), (_, b)) | ExprF::Mul((_, a), (_, b)) => 1 + a + b,
            },
            &expr,
        );
        assert_eq!(count, 3);
    }

    #[test]
    fn test_list_from_to_vec() {
        let list = List::from_vec(vec![1, 2, 3]);
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_cata_list_sum() {
        let list = List::from_vec(vec![1, 2, 3, 4, 5]);
        let sum = cata_list(0, &|a, b| a + b, &list);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_cata_list_length() {
        let list = List::from_vec(vec![10, 20, 30]);
        let len = cata_list(0, &|_, n| n + 1, &list);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_ana_list() {
        let list: List<i32> = ana_list(
            &|n: &i32| {
                if *n <= 0 {
                    None
                } else {
                    Some((*n, n - 1))
                }
            },
            &3,
        );
        assert_eq!(list.to_vec(), vec![3, 2, 1]);
    }

    #[test]
    fn test_apo_expr() {
        // Simple apomorphism: short-circuit at 2
        let expr = apo_expr(
            &|n: &i64| {
                if *n <= 1 {
                    ExprF::Val(*n)
                } else if *n == 2 {
                    // Short-circuit: use pre-built subtree
                    ExprF::Add(Ok(Expr::val(1)), Ok(Expr::val(1)))
                } else {
                    ExprF::Add(Err(n - 1), Err(1i64))
                }
            },
            &3,
        );
        assert_eq!(eval_expr(&expr), 3);
    }
}
