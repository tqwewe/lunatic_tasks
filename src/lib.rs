mod ordered;
mod unordered;

use lunatic::serializer::Bincode;

pub use ordered::TasksOrdered;
pub use unordered::TasksUnordered;

/// An extension trait for iterators that provide helpful methods for working with tasks in Lunatic.
pub trait TaskExt<C, M>
where
    Self: Iterator<Item = C> + Sized,
    M: 'static,
{
    /// Execute tasks buffered, with results returned in order.
    fn tasks_ordered(self, n: usize, f: fn(C) -> M) -> TasksOrdered<Self, C, M, Bincode> {
        self.tasks_ordered_with_serializer(n, f)
    }

    /// Execute tasks buffered, with results returned in order.
    ///
    /// This method differs from [`tasks_ordered`] in that you may provide a different `Serializer` than `Bincode`.
    fn tasks_ordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksOrdered<Self, C, M, S>;

    /// Execute tasks buffered, with results returned in any order as soon as results are available.
    fn tasks_unordered(self, n: usize, f: fn(C) -> M) -> TasksUnordered<Self, C, M, Bincode> {
        self.tasks_unordered_with_serializer(n, f)
    }

    /// Execute tasks buffered, with results returned in any order as soon as results are available.
    ///
    /// This method differs from [`tasks_unordered`] in that you may provide a different `Serializer` than `Bincode`.
    fn tasks_unordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksUnordered<Self, C, M, S>;
}

impl<I, C, M> TaskExt<C, M> for I
where
    Self: Iterator<Item = C>,
    M: 'static,
{
    fn tasks_ordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksOrdered<Self, C, M, S> {
        TasksOrdered::new(self, n, f)
    }

    fn tasks_unordered_with_serializer<S>(
        self,
        n: usize,
        f: fn(C) -> M,
    ) -> TasksUnordered<Self, C, M, S> {
        TasksUnordered::new(self, n, f)
    }
}
