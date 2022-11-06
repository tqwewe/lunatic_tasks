use std::collections::VecDeque;

use lunatic::{
    function::FuncRef,
    protocol::{Protocol, ProtocolCapture, Recv, Send as SendProtocol, TaskEnd},
    serializer::{Bincode, Serializer},
    Process,
};

/// Iterator for the [`tasks_ordered`](super::TaskExt::tasks_ordered) method.
pub struct TasksOrdered<I, C, M, S = Bincode>
where
    I: Iterator<Item = C>,
    M: 'static,
{
    iterator: I,
    f: fn(C) -> M,
    max: usize,
    tasks: VecDeque<Protocol<Recv<M, TaskEnd>, S>>,
}

impl<I, C, M, S> TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = C>,
    M: 'static,
{
    pub(super) fn new<T>(iterator: T, n: usize, f: fn(C) -> M) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        TasksOrdered {
            iterator: iterator.into_iter(),
            f,
            max: n,
            tasks: VecDeque::with_capacity(n),
        }
    }
}

impl<I, C, M, S> Iterator for TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = C>,
    M: 'static,
    S: Serializer<M>
        + Serializer<(C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(C, FuncRef<fn(C) -> M>)>>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let num_to_buffer = self.max - self.tasks.len();
        for _ in 0..num_to_buffer {
            let Some(capture) = self.iterator.next() else { break };
            self.tasks.push_back(Process::spawn_link(
                (capture, FuncRef::new(self.f)),
                |(capture, entry), protocol: Protocol<SendProtocol<M, TaskEnd>, S>| {
                    let result = entry(capture);
                    let _ = protocol.send(result);
                },
            ));
        }
        self.tasks.pop_front().map(|next| next.result())
    }
}
