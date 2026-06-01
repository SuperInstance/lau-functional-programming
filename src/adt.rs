//! Algebraic data types and their universal properties.
//! Sum types (coproduct), product types, and their relationships.

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

/// Product type (tuple-like).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Product<A, B> {
    pub fst: A,
    pub snd: B,
}

impl<A, B> Product<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Product { fst: a, snd: b }
    }
}

/// Sum type (either-like).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sum<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Sum<A, B> {
    pub fn left(a: A) -> Self { Sum::Left(a) }
    pub fn right(b: B) -> Self { Sum::Right(b) }

    pub fn either<C>(self, f: impl FnOnce(A) -> C, g: impl FnOnce(B) -> C) -> C {
        match self {
            Sum::Left(a) => f(a),
            Sum::Right(b) => g(b),
        }
    }
}

/// Unit type.
pub type Unit = ();

/// Void type (uninhabitable).
pub enum Void {}

/// Maybe type (Option alias for ADT discussion).
pub type Maybe<A> = Option<A>;

// --- Universal properties ---

/// Universal property of products: for any type C with morphisms to A and B,
/// there exists a unique morphism to Product<A, B>.
pub fn product_universal<A: Clone, B: Clone, C>(
    f: impl Fn(&C) -> A,
    g: impl Fn(&C) -> B,
    c: &C,
) -> Product<A, B> {
    Product::new(f(c), g(c))
}

/// Universal property of coproducts (sums): for any type C with morphisms from A and B,
/// there exists a unique morphism from Sum<A, B>.
pub fn coproduct_universal<A, B, C>(
    f: impl FnOnce(A) -> C,
    g: impl FnOnce(B) -> C,
    sum: Sum<A, B>,
) -> C {
    sum.either(f, g)
}

// --- ADT cardinality ---

/// Cardinality of a type (number of inhabitants).
pub trait Cardinality {
    fn cardinality() -> Option<u64>;
}

impl Cardinality for () {
    fn cardinality() -> Option<u64> { Some(1) }
}

impl Cardinality for bool {
    fn cardinality() -> Option<u64> { Some(2) }
}

impl Cardinality for u8 {
    fn cardinality() -> Option<u64> { Some(256) }
}

impl<A: Cardinality, B: Cardinality> Cardinality for Product<A, B> {
    fn cardinality() -> Option<u64> {
        Some(A::cardinality()? * B::cardinality()?)
    }
}

impl<A: Cardinality, B: Cardinality> Cardinality for Sum<A, B> {
    fn cardinality() -> Option<u64> {
        Some(A::cardinality()? + B::cardinality()?)
    }
}

impl<A: Cardinality> Cardinality for Option<A> {
    fn cardinality() -> Option<u64> {
        Some(A::cardinality()? + 1)
    }
}

/// Cardinality of a struct with named fields (product).
pub fn product_cardinality(parts: &[u64]) -> u64 {
    parts.iter().product()
}

/// Cardinality of an enum (sum).
pub fn sum_cardinality(variants: &[u64]) -> u64 {
    variants.iter().sum()
}

// --- Polynomial functors ---

/// A polynomial functor representing the shape of an ADT.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PolyFunctor {
    /// Coefficients for each position.
    pub coefficients: Vec<u64>,
    /// Exponents (number of recursive positions per variant).
    pub positions: Vec<usize>,
}

impl PolyFunctor {
    pub fn new(coefficients: Vec<u64>, positions: Vec<usize>) -> Self {
        PolyFunctor { coefficients, positions }
    }

    /// Cardinality as a function of the recursive parameter's cardinality.
    pub fn cardinality_at(&self, n: u64) -> u64 {
        self.coefficients
            .iter()
            .zip(self.positions.iter())
            .map(|(c, p)| c * n.pow(*p as u32))
            .sum()
    }
}

// --- Initial algebra interpretation ---

/// Interpret a polynomial functor as a matrix equation for counting inhabitants.
/// Uses nalgebra for the computation.
pub fn count_inhabitants_matrix(
    equations: &[(Vec<f64>, f64)], // (row coefficients, constant term)
) -> Option<DVector<f64>> {
    let n = equations.len();
    if n == 0 {
        return None;
    }

    let mut mat_data = Vec::new();
    let mut const_data = Vec::new();

    for (row, c) in equations {
        // T = a*T + b => (1-a)*T = b
        let mut row_vec = Vec::new();
        for (i, coeff) in row.iter().enumerate() {
            if i < n {
                if i < row.len() {
                    row_vec.push(-coeff);
                } else {
                    row_vec.push(0.0);
                }
            }
        }
        // Add identity: T_i coefficient
        for (i, _) in row.iter().enumerate() {
            if i < n {
                row_vec[i] += if i < row.len() - 0 { 0.0 } else { 0.0 };
            }
        }
        mat_data.extend(row.iter().cloned());
        const_data.push(*c);
    }

    let mat = DMatrix::from_row_slice(n, n, &mat_data);
    let const_vec = DVector::from_vec(const_data);

    mat.lu().solve(&const_vec)
}

/// Compute the number of binary trees with n nodes (Catalan number).
pub fn catalan(n: u64) -> u64 {
    let mut c = 1u64;
    for i in 0..n {
        c = c * 2 * (2 * i + 1) / (i + 1);
    }
    c / (n + 1)
}

/// Matrix exponentiation for counting paths in ADT automata.
pub fn matrix_power(base: &DMatrix<f64>, exp: usize) -> DMatrix<f64> {
    if exp == 0 {
        return DMatrix::identity(base.nrows(), base.ncols());
    }
    if exp == 1 {
        return base.clone();
    }
    let half = matrix_power(base, exp / 2);
    let result = &half * &half;
    if exp % 2 == 0 {
        result
    } else {
        result * base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_universal() {
        let p = product_universal(|x: &i32| x * 2, |x: &i32| x + 1, &5);
        assert_eq!(p, Product::new(10, 6));
    }

    #[test]
    fn test_sum_left() {
        let s: Sum<i32, String> = Sum::left(42);
        let result = s.either(|x| x.to_string(), |s| s);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_sum_right() {
        let s: Sum<i32, String> = Sum::right("hello".into());
        let result = s.either(|x| x.to_string(), |s| s);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_coproduct_universal() {
        let s: Sum<i32, i32> = Sum::left(42);
        let result = coproduct_universal(|x| x * 2, |x| x + 1, s);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_cardinality_unit() {
        assert_eq!(<()>::cardinality(), Some(1));
    }

    #[test]
    fn test_cardinality_bool() {
        assert_eq!(<bool>::cardinality(), Some(2));
    }

    #[test]
    fn test_cardinality_product() {
        assert_eq!(<Product<bool, bool>>::cardinality(), Some(4));
    }

    #[test]
    fn test_cardinality_sum() {
        assert_eq!(<Sum<bool, bool>>::cardinality(), Some(4));
    }

    #[test]
    fn test_cardinality_option() {
        assert_eq!(<Option<bool>>::cardinality(), Some(3));
    }

    #[test]
    fn test_product_cardinality() {
        assert_eq!(product_cardinality(&[2, 3, 4]), 24);
    }

    #[test]
    fn test_sum_cardinality() {
        assert_eq!(sum_cardinality(&[2, 3]), 5);
    }

    #[test]
    fn test_poly_functor() {
        // Maybe: 1 + T (constant 1, one recursive position)
        let pf = PolyFunctor::new(vec![1, 1], vec![0, 1]);
        assert_eq!(pf.cardinality_at(0), 1); // 1 + 0 = 1
        assert_eq!(pf.cardinality_at(1), 2); // 1 + 1 = 2
        assert_eq!(pf.cardinality_at(2), 3); // 1 + 2 = 3
    }

    #[test]
    fn test_catalan() {
        assert_eq!(catalan(0), 1);
        assert_eq!(catalan(1), 1);
        assert_eq!(catalan(2), 2);
        assert_eq!(catalan(3), 5);
        assert_eq!(catalan(4), 14);
    }

    #[test]
    fn test_matrix_power() {
        let m = DMatrix::from_row_slice(2, 2, &[1.0, 1.0, 1.0, 0.0]);
        let m2 = matrix_power(&m, 2);
        assert!((m2[(0, 0)] - 2.0).abs() < 1e-10);
        assert!((m2[(0, 1)] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_nalgebra_basic() {
        let m = DMatrix::from_row_slice(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        let v = DVector::from_vec(vec![1.0, 2.0]);
        let result = &m * &v;
        assert_eq!(result[0], 1.0);
        assert_eq!(result[1], 2.0);
    }
}
