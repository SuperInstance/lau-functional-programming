//! Lazy evaluation: thunks, streams, infinite sequences.

use std::cell::RefCell;
use std::rc::Rc;

/// A lazy thunk: deferred computation that is evaluated at most once.
pub struct Thunk<A> {
    value: Rc<RefCell<Option<A>>>,
    #[allow(clippy::type_complexity)]
    compute: Rc<RefCell<Option<Box<dyn FnOnce() -> A>>>>,
}

impl<A: Clone + 'static> Thunk<A> {
    /// Create a new thunk from a computation.
    pub fn new(f: impl FnOnce() -> A + 'static) -> Self {
        Thunk {
            value: Rc::new(RefCell::new(None)),
            compute: Rc::new(RefCell::new(Some(Box::new(f)))),
        }
    }

    /// Force evaluation of the thunk (memoized).
    pub fn force(&self) -> A {
        // Check if already evaluated
        {
            let v = self.value.borrow();
            if let Some(ref a) = *v {
                return a.clone();
            }
        }
        // Take the computation
        let f = self.compute.borrow_mut().take();
        if let Some(f) = f {
            let result = f();
            *self.value.borrow_mut() = Some(result.clone());
            result
        } else {
            // Someone else computed it between our checks
            self.value.borrow().clone().unwrap()
        }
    }

    /// Create an already-evaluated thunk.
    pub fn pure(a: A) -> Self {
        Thunk {
            value: Rc::new(RefCell::new(Some(a))),
            compute: Rc::new(RefCell::new(None)),
        }
    }

    /// Map over a thunk lazily.
    pub fn map<B: Clone + 'static>(&self, f: impl FnOnce(A) -> B + 'static) -> Thunk<B> {
        let inner = self.clone();
        Thunk::new(move || f(inner.force()))
    }
}

impl<A> Clone for Thunk<A> {
    fn clone(&self) -> Self {
        Thunk {
            value: Rc::clone(&self.value),
            compute: Rc::clone(&self.compute),
        }
    }
}

/// A lazy linked list / stream.
#[derive(Clone)]
pub enum Stream<A: Clone + 'static> {
    Empty,
    Cons(Thunk<A>, Thunk<Stream<A>>),
}

impl<A: Clone + 'static> Stream<A> {
    pub fn empty() -> Self {
        Stream::Empty
    }

    pub fn cons(head: A, tail: Stream<A>) -> Self {
        Stream::Cons(Thunk::pure(head), Thunk::pure(tail))
    }

    /// Take n elements from the stream.
    pub fn take(&self, n: usize) -> Vec<A> {
        let mut result = Vec::new();
        let mut current = self.clone();
        for _ in 0..n {
            match current {
                Stream::Empty => break,
                Stream::Cons(head, tail) => {
                    result.push(head.force());
                    current = tail.force();
                }
            }
        }
        result
    }

    /// Map over a stream.
    pub fn stream_map<B: Clone + 'static, F: Fn(A) -> B + Clone + 'static>(&self, f: F) -> Stream<B> {
        match self {
            Stream::Empty => Stream::Empty,
            Stream::Cons(head, tail) => {
                let h = head.clone();
                let t = tail.clone();
                let f2 = f.clone();
                Stream::Cons(
                    Thunk::new(move || f(h.force())),
                    Thunk::new(move || t.force().stream_map(f2)),
                )
            }
        }
    }

    /// Filter a stream.
    pub fn stream_filter<F: Fn(&A) -> bool + Clone + 'static>(&self, pred: F) -> Stream<A> {
        match self {
            Stream::Empty => Stream::Empty,
            Stream::Cons(head, tail) => {
                let h = head.clone();
                let t = tail.clone();
                let val = h.force();
                if pred(&val) {
                    Stream::Cons(
                        Thunk::pure(val),
                        Thunk::new(move || t.force().stream_filter(pred.clone())),
                    )
                } else {
                    t.force().stream_filter(pred)
                }
            }
        }
    }

    /// Infinite stream of repeated application of f, starting from seed.
    pub fn iterate<F: Fn(A) -> A + Clone + 'static>(seed: A, f: F) -> Stream<A> {
        let f2 = f.clone();
        Stream::Cons(
            Thunk::pure(seed.clone()),
            Thunk::new(move || {
                let next = f(seed);
                Stream::iterate(next, f2.clone())
            }),
        )
    }

    /// Infinite stream repeating the same value.
    pub fn repeat(a: A) -> Stream<A> {
        let a2 = a.clone();
        Stream::Cons(
            Thunk::pure(a),
            Thunk::new(move || Stream::repeat(a2.clone())),
        )
    }

    /// Natural numbers starting from n.
    pub fn naturals_from(n: A) -> Stream<A>
    where
        A: std::ops::Add<Output = A> + From<u8> + Clone + 'static,
    {
        Stream::iterate(n, |x| x + A::from(1u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thunk_evaluates_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let t = Thunk::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            42
        });
        assert_eq!(t.force(), 42);
        assert_eq!(t.force(), 42);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_thunk_pure() {
        let t = Thunk::pure(42);
        assert_eq!(t.force(), 42);
    }

    #[test]
    fn test_thunk_map() {
        let t = Thunk::pure(42);
        let mapped = t.map(|x| x + 1);
        assert_eq!(mapped.force(), 43);
    }

    #[test]
    fn test_stream_take() {
        let s = Stream::cons(1, Stream::cons(2, Stream::cons(3, Stream::empty())));
        assert_eq!(s.take(2), vec![1, 2]);
    }

    #[test]
    fn test_stream_take_all() {
        let s = Stream::cons(1, Stream::cons(2, Stream::empty()));
        assert_eq!(s.take(5), vec![1, 2]);
    }

    #[test]
    fn test_stream_iterate() {
        let s = Stream::iterate(0i32, |x| x + 1);
        assert_eq!(s.take(5), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_stream_repeat() {
        let s = Stream::repeat(42);
        assert_eq!(s.take(3), vec![42, 42, 42]);
    }

    #[test]
    fn test_stream_map() {
        let s = Stream::iterate(1i32, |x| x + 1);
        let doubled = s.stream_map(|x| x * 2);
        assert_eq!(doubled.take(3), vec![2, 4, 6]);
    }

    #[test]
    fn test_stream_filter() {
        let s = Stream::iterate(1i32, |x| x + 1);
        let evens = s.stream_filter(|x| x % 2 == 0);
        assert_eq!(evens.take(3), vec![2, 4, 6]);
    }

    #[test]
    fn test_naturals() {
        let s = Stream::naturals_from(0i32);
        assert_eq!(s.take(5), vec![0, 1, 2, 3, 4]);
    }
}
