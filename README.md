# lau-functional-programming

> Functional programming theory and patterns in Rust: lambda calculus, combinatory logic, type theory, monads, functors, recursion schemes, and more

A Rust library that implements the theoretical foundations of functional programming as executable code. Not a FP "utility crate" — this is a **teaching and exploration tool** that makes abstract concepts concrete: lambda terms you can reduce, Church encodings that actually compute, type inference you can watch unify, monads you can bind, and recursion schemes you can cata/ana/hylo over.

## What This Does

This crate implements 10 interconnected pillars of functional programming theory:

1. **Lambda Calculus** — terms, free variables, capture-avoiding substitution, α/β/η-reduction, normal forms, and the full suite of Church encodings (numerals, booleans, pairs, arithmetic, Y combinator)
2. **Combinatory Logic** — SKI combinators, weak reduction, bracket abstraction (λ→SKI translation)
3. **Type Theory** — simply-typed lambda calculus with Hindley-Milner type inference, Robinson unification with occurs check, generalization (let-polymorphism)
4. **Monads** — Option, Result, State, Reader, Writer, and IO-like monads; Semigroup and Monoid traits
5. **Functors & Applicatives** — Functor/Applicative/Monad trait hierarchy; Tagged, Const, Identity functors; functor composition
6. **Foldables** — fold_left, fold_right, fold_map for Vec and Option
7. **Lazy Evaluation** — memoized thunks (evaluate-at-most-once), infinite streams with map/filter/iterate/repeat
8. **Pattern Matching Theory** — exhaustiveness checking, redundancy detection, overlap analysis for pattern trees
9. **Recursion Schemes** — catamorphism (fold), anamorphism (unfold), hylomorphism (unfold+fold), paramorphism (fold with history), apomorphism (unfold with short-circuit) on expression and list types
10. **Agent Composition Pipelines** — functional composition, fan-out/fan-in, DAG processing with topological sort and cycle detection

## Key Idea

Functional programming isn't just "functions as values." It's a deep mathematical framework where:

- **Computation** is β-reduction in the lambda calculus
- **Types** are theorems and programs are proofs (Curry-Howard)
- **Effects** are captured by monads (computational burying of side effects)
- **Recursion** is decomposed into data-shaped folds and unfolds (recursion schemes)
- **Composition** is the fundamental operation — everything composes: functions, functors, monads, agents

This crate makes each of these ideas tangible and runnable.

## Install

```toml
[dependencies]
lau-functional-programming = "0.1.0"
```

Or clone directly:

```bash
git clone https://github.com/SuperInstance/lau-functional-programming.git
cd lau-functional-programming
cargo build
```

### Dependencies

- `serde` 1 (with `derive`) — serialization of terms, types, and data structures
- `nalgebra` 0.33 — linear algebra for ADT cardinality computations

## Quick Start

### Lambda Calculus

```rust
use lau_functional_programming::lambda::*;

// Build the identity function: λx.x
let id = Term::abs("x", Term::var("x"));

// Apply it: (λx.x) y
let app = Term::app(id, Term::var("y"));
let result = beta_normalize(&app, 100);
assert_eq!(result, Term::var("y")); // β-reduces to y

// Church numerals
let two = church_numeral(2);
let three = church_numeral(3);
assert_eq!(decode_church_numeral(&two), Some(2));

// Church arithmetic: 2 + 3 = 5
let sum = Term::app(Term::app(church_plus(), two), three);
let result = beta_normalize(&sum, 200);
assert_eq!(decode_church_numeral(&result), Some(5));

// Church arithmetic: 2 × 3 = 6
let product = Term::app(Term::app(church_mult(), two), three);
let result = beta_normalize(&product, 200);
assert_eq!(decode_church_numeral(&result), Some(6));

// Church booleans
let t = Term::app(Term::app(church_true(), Term::var("a")), Term::var("b"));
assert_eq!(beta_normalize(&t, 10), Term::var("a")); // true selects first

// Church pairs
let pair = church_pair(Term::var("x"), Term::var("y"));
let fst = beta_normalize(&Term::app(church_fst(), pair), 100);
assert_eq!(fst, Term::var("x"));
```

### SKI Combinators

```rust
use lau_functional_programming::combinatory::*;

// I combinator: I x = x
let result = comb_normalize(&Comb::app(i(), s()), 10);
assert_eq!(result, s());

// K combinator: K x y = x
let result = comb_normalize(&Comb::app(Comb::app(k(), s()), i()), 10);
assert_eq!(result, s());

// S K K x = x (SKK is identity)
let skk = Comb::app(Comb::app(s(), k()), k());
let result = comb_normalize(&Comb::app(skk, i()), 10);
assert_eq!(result, i());

// Convert lambda to SKI
use lau_functional_programming::lambda::Term;
let id = Term::abs("x", Term::var("x"));
let ski_form = lambda_to_ski(&id);
```

### Type Inference

```rust
use lau_functional_programming::type_theory::*;

// Infer the type of: (λx:Int. x) 42
let expr = Expr::App(
    Box::new(Expr::Abs("x".into(), Type::int(), Box::new(Expr::Var("x".into())))),
    Box::new(Expr::Lit(Literal::Int(42))),
);
let mut env = TypeEnv::new();
let ty = env.infer(&expr).unwrap();
assert_eq!(ty, Type::int());

// Unification: (Int -> t0) ~ (t1 -> Bool)
let s = unify(
    &Type::arrow(Type::int(), Type::Var(0)),
    &Type::arrow(Type::Var(1), Type::bool()),
).unwrap();
assert_eq!(s[&0], Type::bool()); // t0 = Bool
assert_eq!(s[&1], Type::int()); // t1 = Int

// Occurs check prevents infinite types
let result = unify(&Type::Var(0), &Type::arrow(Type::Var(0), Type::int()));
assert!(result.is_err()); // infinite type: t0 ~ t0 -> Int
```

### Monads

```rust
use lau_functional_programming::monad::*;

// Option monad
let result = Some(42).bind(|x| if x > 0 { Some(x * 2) } else { None });
assert_eq!(result, Some(84));

// Result monad
let result: Result<i32, &str> = Ok(42).bind(|x| if x > 0 { Ok(x) } else { Err("negative") });
assert_eq!(result, Ok(42));

// Writer monad (accumulates log)
let w = Writer::new(42, "hello".to_string());
let result = w.bind(|x| Writer::new(x + 1, " world".to_string()));
assert_eq!(result.value, 43);
assert_eq!(result.log, "hello world");

// State monad
let s = State::<i32, i32>::new(|s| (s + 1, s));
let (val, state) = s.run_state(10);
assert_eq!(val, 11);

// Reader monad
let r = Reader::<i32, i32>::new(|env| env * 2);
assert_eq!(r.run_reader(&21), 42);

// Foldable
let sum = vec![1, 2, 3, 4, 5].fold_left(0, |acc, x| acc + x);
assert_eq!(sum, 15);
```

### Lazy Evaluation

```rust
use lau_functional_programming::lazy::*;

// Thunks: evaluate at most once (memoized)
let t = Thunk::new(|| { /* expensive computation */ 42 });
assert_eq!(t.force(), 42);  // Computes
assert_eq!(t.force(), 42);  // Returns cached

// Infinite streams
let naturals = Stream::naturals_from(0i32);
assert_eq!(naturals.take(5), vec![0, 1, 2, 3, 4]);

// Map over infinite stream
let doubled = naturals.stream_map(|x| x * 2);
assert_eq!(doubled.take(3), vec![0, 2, 4]);

// Filter infinite stream
let evens = Stream::iterate(1i32, |x| x + 1).stream_filter(|x| x % 2 == 0);
assert_eq!(evens.take(3), vec![2, 4, 6]);

// Repeat forever
let forever = Stream::repeat(42);
assert_eq!(forever.take(3), vec![42, 42, 42]);
```

### Recursion Schemes

```rust
use lau_functional_programming::recursion::*;

// Build an expression: (2 * 3) + 4
let expr = Expr::add(Expr::mul(Expr::val(2), Expr::val(3)), Expr::val(4));

// Catamorphism (fold): evaluate
let eval = cata_expr_general(
    &|e| match e {
        ExprF::Val(n) => *n,
        ExprF::Add(a, b) => a + b,
        ExprF::Mul(a, b) => a * b,
    },
    &expr,
);
assert_eq!(eval, 10);

// Catamorphism: count nodes
let size = cata_expr_general(
    &|e| match e {
        ExprF::Val(_) => 1,
        ExprF::Add(a, b) | ExprF::Mul(a, b) => 1 + a + b,
    },
    &expr,
);
assert_eq!(size, 5);

// Paramorphism: fold with access to original subexpressions
let count = para_expr(
    &|e| match e {
        ExprF::Val(_) => 1,
        ExprF::Add((_, a), (_, b)) | ExprF::Mul((_, a), (_, b)) => 1 + a + b,
    },
    &expr,
);

// List catamorphism (fold)
let list = List::from_vec(vec![1, 2, 3, 4, 5]);
let sum = cata_list(0, &|a, b| a + b, &list);
assert_eq!(sum, 15);

// List anamorphism (unfold)
let countdown: List<i32> = ana_list(&|n: &i32| if *n <= 0 { None } else { Some((*n, n - 1)) }, &3);
assert_eq!(countdown.to_vec(), vec![3, 2, 1]);

// Hylomorphism: unfold then fold in one step
let result = hylo_expr(
    |n: &i64| match *n { 0 => ExprF::Add(1, 2), 1 | 2 => ExprF::Val(*n), _ => ExprF::Val(0) },
    |e| match e { ExprF::Val(n) => *n, ExprF::Add(a, b) => a + b, ExprF::Mul(a, b) => a * b },
    &0i64,
);
assert_eq!(result, 3);
```

### Algebraic Data Types

```rust
use lau_functional_programming::adt::*;

// Product cardinality: |(A, B)| = |A| × |B|
assert_eq!(<Product<bool, bool>>::cardinality(), Some(4));

// Sum cardinality: |A + B| = |A| + |B|
assert_eq!(<Sum<bool, bool>>::cardinality(), Some(4));

// Option = 1 + T
assert_eq!(<Option<bool>>::cardinality(), Some(3)); // None, Some(true), Some(false)

// Catalan numbers: count binary trees with n nodes
assert_eq!(catalan(0), 1);
assert_eq!(catalan(3), 5);
assert_eq!(catalan(4), 14);

// Polynomial functor for recursive types
// Maybe: 1 + T → coefficients [1, 1], positions [0, 1]
let pf = PolyFunctor::new(vec![1, 1], vec![0, 1]);
assert_eq!(pf.cardinality_at(0), 1); // 1 + 0
assert_eq!(pf.cardinality_at(2), 3); // 1 + 2
```

### Pattern Matching Theory

```rust
use lau_functional_programming::pattern::*;

// Exhaustiveness: Some(_) + None covers everything
let arms = vec![
    MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
    MatchArm { pattern: Pattern::Constructor("None".into(), vec![]), guard: None },
];
let result = check_exhaustiveness(&arms, &["Some".into(), "None".into()]);
assert_eq!(result, Exhaustiveness::Exhaustive);

// Missing case: only Some(_)
let partial = vec![
    MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
];
let result = check_exhaustiveness(&partial, &["Some".into(), "None".into()]);
assert!(matches!(result, Exhaustiveness::NonExhaustive(_)));

// Redundancy: arm after wildcard is unreachable
let arms = vec![
    MatchArm { pattern: Pattern::Wildcard, guard: None },
    MatchArm { pattern: Pattern::Constructor("Some".into(), vec![Pattern::Wildcard]), guard: None },
];
assert_eq!(check_redundancy(&arms), vec![1]); // arm 1 is redundant
```

### Agent Pipelines

```rust
use lau_functional_programming::pipeline::*;

// Compose functions: g ∘ f
let h = compose(|x: i32| x + 1, |x: i32| x * 2);
assert_eq!(h(3), 8); // (3+1)*2

// DAG processing
let mut dag = Dag::new();
dag.add_node("extract_a", "extract", vec![]);
dag.add_node("extract_b", "extract", vec![]);
dag.add_node("transform", "transform", vec!["extract_a", "extract_b"]);
dag.add_node("load", "load", vec!["transform"]);

let order = dag.topological_order().unwrap();
// extract_a and extract_b come before transform, which comes before load

// Fan-out: send input to multiple agents
let results = fan_out(42, vec![
    Box::new(|x: i32| format!("a: {}", x)),
    Box::new(|x: i32| format!("b: {}", x)),
]);
assert_eq!(results, vec!["a: 42", "b: 42"]);
```

## API Reference

### Modules

| Module | Lines | Description |
|--------|-------|-------------|
| `lambda` | 477 | Lambda calculus: Term, substitution, β/η-reduction, Church encodings |
| `combinatory` | 216 | SKI combinators, weak reduction, bracket abstraction |
| `type_theory` | 333 | Simply-typed λ-calculus, Hindley-Milner inference, unification |
| `monad` | 437 | Functor/Applicative/Monad traits, Option/Result/State/Reader/Writer/IO, Semigroup, Monoid, Foldable |
| `functor` | 112 | Tagged, Const, Identity functors; functor composition |
| `lazy` | 240 | Memoized thunks, infinite streams with map/filter/iterate/repeat |
| `pattern` | 256 | Pattern types, exhaustiveness/redundancy/overlap checking |
| `recursion` | 372 | Fix point types, cata/ana/hylo/para/apo on Expr and List |
| `adt` | 312 | Product/Sum types, cardinality, polynomial functors, Catalan numbers |
| `pipeline` | 323 | Agent composition, DAG processing, fan-out/fan-in |

### Core Types

```rust
// Lambda calculus
pub enum Term { Var(String), Abs(String, Box<Term>), App(Box<Term>, Box<Term>) }

// Combinatory logic
pub enum Comb { S, K, I, App(Box<Comb>, Box<Comb>) }

// Type theory
pub enum Type { Base(String), Arrow(Box<Type>, Box<Type>), Var(usize), Forall(usize, Box<Type>) }
pub enum Expr { Var(String), Abs(String, Type, Box<Expr>), App(Box<Expr>, Box<Expr>), Let(...), Lit(...) }

// Monads
pub struct State<S, A> { run: Option<Box<dyn FnOnce(S) -> (A, S)>> }
pub struct Reader<R, A> { run: Box<dyn FnOnce(&R) -> A> }
pub struct Writer<W, A> { value: A, log: W }
pub enum IO<A> { Pure(A), Print(String, Box<IO<A>>), ReadLine(Box<IO<A>>) }

// Lazy
pub struct Thunk<A> { /* memoized computation */ }
pub enum Stream<A> { Empty, Cons(Thunk<A>, Thunk<Stream<A>>) }

// Recursion
pub struct Fix<F> { unfix: Box<F> }
pub enum ExprF<A> { Val(i64), Add(A, A), Mul(A, A) }
pub enum Expr { Val(i64), Add(Box<Expr>, Box<Expr>), Mul(Box<Expr>, Box<Expr>) }
pub enum List<A> { Nil, Cons(A, Box<List<A>>) }

// ADT
pub struct Product<A, B> { fst: A, snd: B }
pub enum Sum<A, B> { Left(A), Right(B) }
pub struct PolyFunctor { coefficients: Vec<u64>, positions: Vec<usize> }

// Pattern matching
pub enum Pattern { Wildcard, Constructor(String, Vec<Pattern>), Literal(LitPat), Var(String), Or(...) }

// Pipeline
pub struct Dag { nodes: Vec<Node> }
```

## How It Works

### Lambda Calculus Engine

The lambda calculus is the theoretical foundation of all functional programming. This implementation provides:

1. **Capture-avoiding substitution** — when substituting into a lambda body, if the bound variable would capture free variables in the substitution, the bound variable is automatically renamed (α-conversion)
2. **β-reduction** — the sole computation rule: (λx.M) N → M[x := N]. Implemented as leftmost-outermost (call-by-name) strategy
3. **η-reduction** — simplifies λx.f x → f when x is not free in f
4. **Church encodings** — everything is a lambda term: numbers are iterated application, booleans are selectors, pairs are lambda-encoded, arithmetic is composition

### Type Inference Pipeline

Hindley-Milner type inference works in three phases:

1. **Constraint generation** — walk the expression tree, generating unification constraints between type variables
2. **Unification** — solve constraints via Robinson's algorithm: recursively match type structures, bind variables, check for infinite types via occurs check
3. **Generalization** — at let-bindings, quantify over type variables not free in the environment

### Monad Architecture

The trait hierarchy follows the mathematical definition:

```
Functor<A>        — fmap: (A → B) → F<A> → F<B>
  ↕
Applicative<A>    — pure: A → F<A>, ap: F<(A → B)> → F<A> → F<B>
  ↕
Monad<A>          — pure: A → F<A>, bind: F<A> → (A → F<B>) → F<B>
```

Each concrete monad (Option, Result, State, Reader, Writer, IO) implements this hierarchy with its own semantics.

### Recursion Schemes

Recursion schemes separate the *shape of recursion* from the *business logic*:

- **Catamorphism** (fold): collapse structure bottom-up — replace each constructor with a function
- **Anamorphism** (unfold): build structure top-down — generate constructors from seeds
- **Hylomorphism**: ana then cata — build an intermediate structure, then collapse it (without materializing the full tree)
- **Paramorphism**: fold with access to the original subexpression at each step (useful for computations that need both the result and the original)
- **Apomorphism**: unfold with short-circuiting — at each step, you can either continue unfolding or return a pre-built subtree

## The Math

### Lambda Calculus

The untyped lambda calculus has three constructs:

- **Variables**: x, y, z
- **Abstraction**: λx.M (function definition)
- **Application**: M N (function call)

β-reduction is the sole computation rule: (λx.M) N → M[x := N]

Church encodings represent data purely with functions:
- **Numeral** n = λf.λx.fⁿ(x) — n-fold application
- **True** = λt.λf.t, **False** = λt.λf.f — selection
- **Pair** (a,b) = λf.f a b, **Fst** = λp.p (λx.λy.x) — encoding via application
- **Y** = λf.(λx.f(x x))(λx.f(x x)) — fixed-point combinator for recursion

### Simply-Typed Lambda Calculus

Types τ ::= α | τ₁ → τ₂ | ∀α.τ

**Typing rules:**
- (Var): Γ ⊢ x : τ if (x:τ) ∈ Γ
- (Abs): Γ,x:τ₁ ⊢ M : τ₂ ⟹ Γ ⊢ λx:τ₁.M : τ₁ → τ₂
- (App): Γ ⊢ f : τ₁ → τ₂, Γ ⊢ a : τ₁ ⟹ Γ ⊢ f a : τ₂

**Unification:** Robinson's algorithm with occurs check prevents infinite types like α = α → β.

### Category Theory Connections

- **Functor**: mapping between categories preserving structure — fmap id = id, fmap (f∘g) = fmap f ∘ fmap g
- **Monad**: a monoid in the category of endofunctors — pure is the unit, bind is the multiplication
- **Natural transformation**: a family of morphisms between functors — refinements are natural transformations between protocol sheaves

### Recursion Schemes

Given a functor F and its fixed point μF:

- **Cata**: (F A → A) → μF → A (algebra → fold)
- **Ana**: (A → F A) → A → μF (coalgebra → unfold)
- **Hylo**: (A → F B) → (F B → B) → A → B (unfold+fold)
- **Para**: (F (μF, A) → A) → μF → A (fold with originals)

### ADT Cardinality

The number of inhabitants of a type follows simple rules:
- |()| = 1, |Bool| = 2, |u8| = 256
- |(A, B)| = |A| × |B| (product)
- |A + B| = |A| + |B| (sum)
- |Option<A>| = |A| + 1 (None adds one more)
- |List<A>| = Σᵢ |A|ⁱ = 1/(1-|A|) when |A| < 1 (geometric series)

Catalan numbers count binary trees: Cₙ = (2n)! / ((n+1)!n!)

## Tests

The crate contains **126 unit tests** across all modules, covering:

- Lambda calculus: free variables, substitution (with capture avoidance), β/η reduction, Church encodings
- SKI combinators: I/K/S reduction, SKK identity, bracket abstraction
- Type theory: inference, unification, occurs check, generalization
- Monads: Option/Result bind, Writer accumulation, State execution, Reader environment
- Functors: Tagged, Const, Identity mapping
- Foldables: Vec and Option folding
- Lazy: thunk memoization, stream operations
- Pattern matching: exhaustiveness, redundancy, overlap
- Recursion schemes: cata/ana/hylo/para/apo on expressions and lists
- ADT: cardinality, polynomial functors, Catalan numbers, matrix power
- Pipelines: composition, DAG topological sort, cycle detection, fan-out/fan-in

```bash
cargo test
```

## License

MIT
