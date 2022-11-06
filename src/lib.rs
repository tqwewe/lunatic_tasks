mod ordered;
mod unordered;

use lunatic::serializer::Bincode;

pub use ordered::TasksOrdered;
pub use unordered::TasksUnordered;

pub trait TaskExt<C, M>
where
    Self: Iterator<Item = (C, fn(C) -> M)> + Sized,
    M: 'static,
{
    fn tasks_ordered(self, n: usize) -> TasksOrdered<Self, C, M, Bincode> {
        self.tasks_ordered_with_serializer(n)
    }

    fn tasks_ordered_with_serializer<S>(self, n: usize) -> TasksOrdered<Self, C, M, S>;

    fn tasks_unordered(self, n: usize) -> TasksUnordered<Self, C, M> {
        self.tasks_unordered_with_serializer(n)
    }

    fn tasks_unordered_with_serializer<S>(self, n: usize) -> TasksUnordered<Self, C, M, S>;
}

impl<I, C, M> TaskExt<C, M> for I
where
    Self: Iterator<Item = (C, fn(C) -> M)>,
    M: 'static,
{
    fn tasks_ordered_with_serializer<S>(self, n: usize) -> TasksOrdered<Self, C, M, S> {
        TasksOrdered::new(self, n)
    }

    fn tasks_unordered_with_serializer<S>(self, n: usize) -> TasksUnordered<Self, C, M, S> {
        TasksUnordered::new(self, n)
    }
}
