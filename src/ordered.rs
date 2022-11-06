use std::collections::VecDeque;

use lunatic::{
    function::FuncRef,
    protocol::{Protocol, ProtocolCapture, Recv, TaskEnd},
    serializer::{Bincode, Serializer},
    Process,
};

pub struct TasksOrdered<I, C, M, S = Bincode>
where
    I: Iterator<Item = (C, fn(C) -> M)>,
    M: 'static,
{
    iterator: I,
    max: usize,
    tasks: VecDeque<Protocol<Recv<M, TaskEnd>, S>>,
}

impl<I, C, M, S> TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = (C, fn(C) -> M)>,
    M: 'static,
{
    pub(super) fn new<T>(iterator: T, n: usize) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        TasksOrdered {
            iterator: iterator.into_iter(),
            max: n,
            tasks: VecDeque::with_capacity(n),
        }
    }
}

impl<I, C, M, S> Iterator for TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = (C, fn(C) -> M)>,
    M: 'static,
    S: Serializer<M>
        + Serializer<(C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(C, FuncRef<fn(C) -> M>)>>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let num_to_buffer = self.max - self.tasks.len();
        for _ in 0..num_to_buffer {
            let Some((capture, entry)) = self.iterator.next() else { break };
            self.tasks.push_back(Process::spawn_link(
                (capture, FuncRef::new(entry)),
                |(capture, entry), protocol: Protocol<lunatic::protocol::Send<M, TaskEnd>, S>| {
                    let result = entry(capture);
                    let _ = protocol.send(result);
                },
            ));
        }
        self.tasks.pop_front().map(|next| next.result())
    }
}
