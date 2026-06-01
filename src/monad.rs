//! Monads: Option, Result, State, Reader, Writer, IO-like.
//! Functors, Applicatives, Foldables, Traversables.

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// === Functor ===

/// Trait for functors: types that can be mapped over.
pub trait Functor<A> {
    type Target<B>;
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Self::Target<B>;
}

// === Applicative ===

/// Trait for applicative functors.
pub trait Applicative<A>: Functor<A> {
    fn pure(a: A) -> Self::Target<A>;
    fn ap<B>(self, fs: Self::Target<impl FnOnce(A) -> B>) -> Self::Target<B>;
}

// === Monad ===

/// Core monad trait.
pub trait Monad<A>: Functor<A> {
    fn pure(a: A) -> Self::Target<A>;
    fn bind<B>(self, f: impl FnOnce(A) -> Self::Target<B>) -> Self::Target<B>;
}

// === Option monad ===

impl<A> Functor<A> for Option<A> {
    type Target<B> = Option<B>;
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Option<B> {
        self.map(f)
    }
}

impl<A> Monad<A> for Option<A> {
    fn pure(a: A) -> Option<A> { Some(a) }
    fn bind<B>(self, f: impl FnOnce(A) -> Option<B>) -> Option<B> {
        self.and_then(f)
    }
}

// === Result monad ===

impl<A, E> Functor<A> for Result<A, E> {
    type Target<B> = Result<B, E>;
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Result<B, E> {
        self.map(f)
    }
}

impl<A, E> Monad<A> for Result<A, E> {
    fn pure(a: A) -> Result<A, E> { Ok(a) }
    fn bind<B>(self, f: impl FnOnce(A) -> Result<B, E>) -> Result<B, E> {
        self.and_then(f)
    }
}

// === State monad ===

/// State monad: S -> (A, S)
#[derive(Serialize, Deserialize)]
pub struct State<S, A> {
    #[serde(skip)]
    pub run: Option<Box<dyn FnOnce(S) -> (A, S)>>,
    _phantom: PhantomData<S>,
}

impl<S: Clone + 'static, A: Clone + 'static> Clone for State<S, A> {
    fn clone(&self) -> Self {
        State {
            run: None, // Can't clone closures
            _phantom: PhantomData,
        }
    }
}

impl<S, A> std::fmt::Debug for State<S, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}

impl<S: Clone + 'static, A: Clone + 'static> State<S, A> {
    pub fn new(f: impl FnOnce(S) -> (A, S) + 'static) -> Self {
        State {
            run: Some(Box::new(f)),
            _phantom: PhantomData,
        }
    }

    pub fn pure(a: A) -> Self {
        State::new(move |s| (a, s))
    }

    pub fn put(s: S) -> Self
    where
        A: Default,
    {
        State::new(move |_| (A::default(), s))
    }

    pub fn modify(f: impl FnOnce(S) -> S + 'static, default_a: A) -> Self {
        State::new(move |s| (default_a, f(s)))
    }

    pub fn run_state(self, s: S) -> (A, S) {
        let f = self.run.unwrap();
        f(s)
    }
}

// === Reader monad ===

/// Reader monad: R -> A
pub struct Reader<R, A> {
    #[allow(clippy::type_complexity)]
    #[allow(clippy::type_complexity)]
    pub run: Box<dyn FnOnce(&R) -> A>,
    _phantom: PhantomData<R>,
}

impl<R: 'static, A: 'static> Reader<R, A> {
    pub fn new(f: impl FnOnce(&R) -> A + 'static) -> Self {
        Reader {
            run: Box::new(f),
            _phantom: PhantomData,
        }
    }

    pub fn pure(a: A) -> Self
    where
        A: Clone,
    {
        Reader::new(move |_| a)
    }

    pub fn run_reader(self, r: &R) -> A {
        (self.run)(r)
    }
}

// === Writer monad ===

/// Writer monad: (A, W) where W is a monoid.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Writer<W, A> {
    pub value: A,
    pub log: W,
}

impl<W: Clone + Semigroup, A: Clone> Writer<W, A> {
    pub fn new(a: A, w: W) -> Self {
        Writer { value: a, log: w }
    }

    pub fn pure(a: A, empty: W) -> Self {
        Writer { value: a, log: empty }
    }

    pub fn tell(w: W, empty: A) -> Self {
        Writer { value: empty, log: w }
    }

    pub fn bind<B>(self, f: impl FnOnce(A) -> Writer<W, B>) -> Writer<W, B>
    where
        B: Clone,
    {
        let Writer { value, log: log1 } = self;
        let Writer { value: new_val, log: log2 } = f(value);
        Writer {
            value: new_val,
            log: log1.combine(&log2),
        }
    }

    pub fn listen(self) -> Writer<W, (A, W)> {
        let log = self.log.clone();
        Writer {
            value: (self.value, log),
            log: self.log,
        }
    }
}

/// Semigroup: associative combine operation.
pub trait Semigroup {
    fn combine(&self, other: &Self) -> Self;
}

/// Monoid: semigroup with identity.
pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

impl Semigroup for String {
    fn combine(&self, other: &Self) -> Self {
        format!("{}{}", self, other)
    }
}

impl Monoid for String {
    fn empty() -> Self { String::new() }
}

impl Semigroup for Vec<String> {
    fn combine(&self, other: &Self) -> Self {
        let mut v = self.clone();
        v.extend(other.iter().cloned());
        v
    }
}

impl Monoid for Vec<String> {
    fn empty() -> Self { Vec::new() }
}

impl Semigroup for i64 {
    fn combine(&self, other: &Self) -> Self {
        self + other
    }
}

impl Monoid for i64 {
    fn empty() -> Self { 0 }
}

// === IO-like monad ===

/// A simple IO representation using descriptions of effects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IO<A> {
    Pure(A),
    Print(String, Box<IO<A>>),
    ReadLine(Box<IO<A>>),
}

impl<A: Clone + 'static> IO<A> {
    pub fn pure(a: A) -> Self {
        IO::Pure(a)
    }

    pub fn print(msg: String, next: IO<A>) -> Self {
        IO::Print(msg, Box::new(next))
    }
}

// === Foldable ===

/// Trait for foldable containers.
pub trait Foldable<A> {
    fn fold_left<B>(self, init: B, f: impl Fn(B, A) -> B) -> B;
    fn fold_right<B>(self, init: B, f: impl Fn(A, B) -> B) -> B;
    fn fold_map<M: Monoid>(self, f: impl Fn(A) -> M) -> M;
}

impl<A: Clone> Foldable<A> for Vec<A> {
    fn fold_left<B>(self, init: B, f: impl Fn(B, A) -> B) -> B {
        self.into_iter().fold(init, f)
    }
    fn fold_right<B>(self, init: B, f: impl Fn(A, B) -> B) -> B {
        self.into_iter().rev().fold(init, |acc, x| f(x, acc))
    }
    fn fold_map<M: Monoid>(self, f: impl Fn(A) -> M) -> M {
        self.into_iter().map(f).fold(M::empty(), |acc, m| acc.combine(&m))
    }
}

impl<A: Clone> Foldable<A> for Option<A> {
    fn fold_left<B>(self, init: B, f: impl Fn(B, A) -> B) -> B {
        match self {
            Some(a) => f(init, a),
            None => init,
        }
    }
    fn fold_right<B>(self, init: B, f: impl Fn(A, B) -> B) -> B {
        match self {
            Some(a) => f(a, init),
            None => init,
        }
    }
    fn fold_map<M: Monoid>(self, f: impl Fn(A) -> M) -> M {
        self.map_or(M::empty(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_functor() {
        let r = Some(42).fmap(|x| x + 1);
        assert_eq!(r, Some(43));
    }

    #[test]
    fn test_option_functor_none() {
        let r: Option<i32> = None.fmap(|x: i32| x + 1);
        assert_eq!(r, None);
    }

    #[test]
    fn test_option_monad_bind() {
        let r = Some(42).bind(|x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(r, Some(84));
    }

    #[test]
    fn test_option_monad_bind_none() {
        let r: Option<i32> = None.bind(|x: i32| Some(x));
        assert_eq!(r, None);
    }

    #[test]
    fn test_option_monad_pure() {
        let r = Option::<i32>::pure(42);
        assert_eq!(r, Some(42));
    }

    #[test]
    fn test_result_monad() {
        let r: Result<i32, &str> = Ok(42).bind(|x| if x > 0 { Ok(x) } else { Err("negative") });
        assert_eq!(r, Ok(42));
    }

    #[test]
    fn test_result_monad_err() {
        let r: Result<i32, &str> = Err("fail".into()).bind(|x: i32| Ok(x + 1));
        assert!(r.is_err());
    }

    #[test]
    fn test_writer_monad() {
        let w = Writer::new(42, "hello".to_string());
        let result = w.bind(|x| Writer::new(x + 1, " world".to_string()));
        assert_eq!(result.value, 43);
        assert_eq!(result.log, "hello world");
    }

    #[test]
    fn test_writer_listen() {
        let w = Writer::new(42, "log".to_string());
        let listened = w.listen();
        assert_eq!(listened.value, (42, "log".to_string()));
        assert_eq!(listened.log, "log");
    }

    #[test]
    fn test_semigroup_string() {
        let a = "hello".to_string();
        let b = " world".to_string();
        assert_eq!(a.combine(&b), "hello world");
    }

    #[test]
    fn test_monoid_string() {
        let empty = String::empty();
        assert_eq!(empty, "");
    }

    #[test]
    fn test_monoid_i64() {
        assert_eq!(i64::empty(), 0);
        assert_eq!(5i64.combine(&3), 8);
    }

    #[test]
    fn test_vec_foldable() {
        let v = vec![1, 2, 3, 4, 5];
        let sum = v.fold_left(0, |acc, x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_vec_fold_right() {
        let v = vec![1, 2, 3];
        let result = v.fold_right(0, |x, acc| x + acc);
        assert_eq!(result, 6);
    }

    #[test]
    fn test_vec_fold_map() {
        let v = vec![1i64, 2, 3];
        let result = v.fold_map(|x| x);
        assert_eq!(result, 6);
    }

    #[test]
    fn test_option_foldable() {
        assert_eq!(Some(42).fold_left(0, |acc, x| acc + x), 42);
        assert_eq!(None::<i32>.fold_left(0, |acc, x| acc + x), 0);
    }

    #[test]
    fn test_io_pure() {
        let io = IO::pure(42);
        assert_eq!(io, IO::Pure(42));
    }

    #[test]
    fn test_io_print() {
        let io = IO::print("hello".to_string(), IO::pure(42));
        match io {
            IO::Print(msg, next) => {
                assert_eq!(msg, "hello");
                assert_eq!(*next, IO::Pure(42));
            }
            _ => panic!("expected Print"),
        }
    }

    #[test]
    fn test_state_monad() {
        let s = State::<i32, i32>::new(|s| (s + 1, s));
        let (val, state) = s.run_state(10);
        assert_eq!(val, 11);
        assert_eq!(state, 10);
    }

    #[test]
    fn test_reader_monad() {
        let r = Reader::<i32, i32>::new(|env| env * 2);
        let result = r.run_reader(&21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_writer_tell() {
        let w = Writer::<String, ()>::tell("logged".to_string(), ());
        assert_eq!(w.log, "logged");
    }
}
