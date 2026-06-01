//! Functor and Applicative re-exports (core traits are in monad.rs).
//! This module adds extra functor/applicative utilities.

pub use crate::monad::{Applicative, Functor, Monad};

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// A tagged functor wrapper for testing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tagged<A> {
    pub tag: String,
    pub value: A,
}

impl<A> Tagged<A> {
    pub fn new(tag: &str, value: A) -> Self {
        Tagged { tag: tag.to_string(), value }
    }
}

impl<A: Clone> super::monad::Functor<A> for Tagged<A> {
    type Target<B> = Tagged<B>;
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Tagged<B> {
        Tagged { tag: self.tag, value: f(self.value) }
    }
}

/// Const functor: always returns the same constant, ignoring the mapped function.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Const<A, B> {
    pub value: A,
    _phantom: PhantomData<B>,
}

impl<A: Clone, B> Const<A, B> {
    pub fn new(a: A) -> Self {
        Const { value: a, _phantom: PhantomData }
    }
}

impl<A: Clone, B> super::monad::Functor<B> for Const<A, B> {
    type Target<C> = Const<A, C>;
    fn fmap<C>(self, _f: impl FnOnce(B) -> C) -> Const<A, C> {
        Const { value: self.value, _phantom: PhantomData }
    }
}

/// Identity functor: wraps a value transparently.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Identity<A>(pub A);

impl<A> super::monad::Functor<A> for Identity<A> {
    type Target<B> = Identity<B>;
    fn fmap<B>(self, f: impl FnOnce(A) -> B) -> Identity<B> {
        Identity(f(self.0))
    }
}

impl<A: Clone + 'static> super::monad::Monad<A> for Identity<A> {
    fn pure(a: A) -> Identity<A> { Identity(a) }
    fn bind<B>(self, f: impl FnOnce(A) -> Identity<B>) -> Identity<B> {
        f(self.0)
    }
}

/// Compose two functors: F (G A).
pub struct Compose<F, G, A> {
    pub outer: F,
    _phantom: std::marker::PhantomData<(G, A)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monad::Functor;

    #[test]
    fn test_tagged_functor() {
        let t = Tagged::new("test", 42);
        let mapped = t.fmap(|x| x + 1);
        assert_eq!(mapped, Tagged { tag: "test".to_string(), value: 43 });
    }

    #[test]
    fn test_const_functor() {
        let c: Const<i32, String> = Const::new(42);
        let mapped: Const<i32, i64> = c.fmap(|_s: String| 99i64);
        assert_eq!(mapped.value, 42);
    }

    #[test]
    fn test_identity_functor() {
        let i = Identity(42);
        let mapped = i.fmap(|x| x * 2);
        assert_eq!(mapped, Identity(84));
    }

    #[test]
    fn test_identity_monad() {
        use crate::monad::Monad;
        let r = Identity(42).bind(|x| Identity(x + 1));
        assert_eq!(r, Identity(43));
    }

    #[test]
    fn test_identity_monad_pure() {
        use crate::monad::Monad;
        let r = Identity::pure(42);
        assert_eq!(r, Identity(42));
    }
}
