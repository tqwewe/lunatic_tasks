//! Utilities for working with tasks in Lunatic.
//!
//! # Example
//!
//! ```
//! let mut tasks = (0..5).rev().tasks_unordered(3, |num| {
//!     lunatic::sleep(Duration::from_millis(num as u64 * 200));
//!     num
//! });
//! assert_eq!(tasks.next(), Some(2));
//! assert_eq!(tasks.next(), Some(3));
//! assert_eq!(tasks.next(), Some(0));
//! assert_eq!(tasks.next(), Some(1));
//! assert_eq!(tasks.next(), Some(4));
//! assert_eq!(tasks.next(), None);
//! ```

#![warn(missing_docs)]

mod ordered;
mod unordered;

use lunatic::{
    function::FuncRef,
    protocol::ProtocolCapture,
    serializer::{Bincode, Serializer},
    Process, Tag,
};

pub use ordered::{SerializedTasksOrdered, TasksOrdered};
use serde::{Deserialize, Serialize};
pub use unordered::{SerializedTasksUnordered, TasksUnordered};

/// An extension trait for iterators that provide helpful methods for working with tasks in Lunatic.
pub trait TaskExt<C, M> {
    /// Execute tasks buffered, with results returned in order.
    ///
    /// # Example
    ///
    /// ```
    /// use lunatic_tasks::TaskExt;
    ///
    /// (0..10).tasks_ordered(2, |n| {
    ///     lunatic::sleep(Duration::from_millis(n as u64 * 200));
    ///     n
    /// })
    /// ```
    fn tasks_ordered(self, n: usize, f: fn(C) -> M) -> TasksOrdered<Self, C, M, Bincode>
    where
        Self: Sized + Iterator<Item = C>,
        C: Serialize + for<'de> Deserialize<'de>,
        M: Serialize + for<'de> Deserialize<'de>,
    {
        self.tasks_ordered_with_serializer(n, f)
    }

    /// Execute tasks buffered, with results returned in order.
    ///
    /// This method differs from [`TaskExt::tasks_ordered`] in that you may provide a different `Serializer` than `Bincode`.
    fn tasks_ordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksOrdered<Self, C, M, S>
    where
        Self: Sized + Iterator<Item = C>,
        S: Serializer<()>
            + Serializer<M>
            + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
            + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>;

    /// Execute tasks buffered, with results returned in any order as soon as results are available.
    ///
    /// # Example
    ///
    /// ```
    /// use lunatic_tasks::TaskExt;
    ///
    /// let mut tasks = (0..5).rev().tasks_unordered(3, |num| {
    ///     lunatic::sleep(Duration::from_millis(num as u64 * 200));
    ///     num
    /// });
    /// assert_eq!(tasks.next(), Some(2));
    /// assert_eq!(tasks.next(), Some(3));
    /// assert_eq!(tasks.next(), Some(0));
    /// assert_eq!(tasks.next(), Some(1));
    /// assert_eq!(tasks.next(), Some(4));
    /// assert_eq!(tasks.next(), None);
    /// ```
    fn tasks_unordered(self, n: usize, f: fn(C) -> M) -> TasksUnordered<Self, C, M, Bincode>
    where
        Self: Sized + Iterator<Item = C>,
        C: Serialize + for<'de> Deserialize<'de>,
        M: Serialize + for<'de> Deserialize<'de>,
    {
        self.tasks_unordered_with_serializer(n, f)
    }

    /// Execute tasks buffered, with results returned in any order as soon as results are available.
    ///
    /// This method differs from [`TaskExt::tasks_unordered`] in that you may provide a different `Serializer` than `Bincode`.
    fn tasks_unordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksUnordered<Self, C, M, S>
    where
        Self: Sized + Iterator<Item = C>,
        C: Serialize + for<'de> Deserialize<'de>,
        M: Serialize + for<'de> Deserialize<'de>,
        S: Serializer<()>
            + Serializer<M>
            + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
            + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>;
}

impl<I, C, M> TaskExt<C, M> for I {
    fn tasks_ordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksOrdered<Self, C, M, S>
    where
        Self: Sized + Iterator<Item = C>,
        S: Serializer<()>
            + Serializer<M>
            + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
            + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
    {
        TasksOrdered::new(self, n, f)
    }

    fn tasks_unordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksUnordered<Self, C, M, S>
    where
        Self: Sized + Iterator<Item = C>,
        C: Serialize + for<'de> Deserialize<'de>,
        M: Serialize + for<'de> Deserialize<'de>,
        S: Serializer<()>
            + Serializer<M>
            + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
            + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
    {
        TasksUnordered::new(self, n, f)
    }
}
