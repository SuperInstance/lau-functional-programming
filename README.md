# lau-functional-programming

Functional programming theory and patterns implemented in Rust.

## Features

- **Lambda Calculus**: Terms, alpha/beta/eta reduction, normal forms, Church encodings (numerals, booleans, pairs, arithmetic, Y combinator)
- **Combinatory Logic**: SKI combinators, weak reduction, bracket abstraction
- **Type Theory**: Simply-typed lambda calculus, Hindley-Milner type inference, unification with occurs check, generalization
- **Monads**: Option, Result, State, Reader, Writer, IO-like; Semigroup and Monoid traits
- **Functors & Applicatives**: Tagged, Const, Identity functors; functor composition
- **Foldables & Traversables**: fold_left, fold_right, fold_map for Vec and Option
- **Lazy Evaluation**: Memoized thunks, infinite streams (iterate, repeat, map, filter)
- **Pattern Matching Theory**: Exhaustiveness checking, redundancy detection, overlap analysis
- **Recursion Schemes**: Catamorphism, anamorphism, hylomorphism, paramorphism, apomorphism on expression and list types
- **Algebraic Data Types**: Product/Sum types, universal properties, cardinality, polynomial functors, Catalan numbers, matrix exponentiation via nalgebra
- **Agent Composition Pipelines**: Functional composition, fan-out/fan-in, DAG processing with topological sort and cycle detection

## Dependencies

- `serde` — Serialization for term/type representations
- `nalgebra` — Linear algebra for ADT cardinality computations

## Usage

```rust
use lau_functional_programming::lambda::*;
use lau_functional_programming::monad::*;
use lau_functional_programming::recursion::*;

// Lambda calculus
let expr = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
let reduced = beta_normalize(&expr, 100);

// Church numerals
let two = church_numeral(2);
let three = church_numeral(3);
let sum = Term::app(Term::app(church_plus(), two), three);

// Monads
let result = Some(42).bind(|x| if x > 0 { Some(x * 2) } else { None });

// Recursion schemes
let expr = Expr::add(Expr::val(1), Expr::mul(Expr::val(2), Expr::val(3)));
let eval = cata_expr_general(&|e| match e {
    ExprF::Val(n) => *n,
    ExprF::Add(a, b) => a + b,
    ExprF::Mul(a, b) => a * b,
}, &expr);
```

## License

MIT
