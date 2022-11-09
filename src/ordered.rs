use std::{collections::VecDeque, vec};

use lunatic::{
    function::FuncRef,
    protocol::ProtocolCapture,
    serializer::{Bincode, Serializer},
    Mailbox, Process, Tag,
};
use serde::{Deserialize, Serialize};

/// Iterator for the [`tasks_ordered`](super::TaskExt::tasks_ordered) method.
pub struct TasksOrdered<I, C, M, S = Bincode>
where
    I: Iterator<Item = C>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    iterator: I,
    f: fn(C) -> M,
    max: usize,
    process: Process<M, S>,
    tags: VecDeque<Tag>,
}

/// A serialized instance of [`TasksOrdered`].
#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "C: Serialize", deserialize = "C: for<'d> Deserialize<'d>"))]
pub struct SerializedTasksOrdered<C, M, S = Bincode> {
    items: Vec<C>,
    f: FuncRef<fn(C) -> M>,
    max: usize,
    process: Process<M, S>,
}

impl<I, C, M, S> TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = C>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    pub(super) fn new<T>(iterator: T, n: usize, f: fn(C) -> M) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        TasksOrdered {
            iterator: iterator.into_iter(),
            f,
            max: n,
            process: Process::this(),
            tags: VecDeque::with_capacity(n),
        }
    }

    /// Collects inner iterator into vec to serialize [`TasksOrdered`].
    ///
    /// Panics if active tasks are running.
    /// Tasks begin running when the iterator is consumed.
    pub fn serialize(self) -> SerializedTasksOrdered<C, M, S>
    where
        C: Serialize + for<'de> Deserialize<'de>,
    {
        if !self.tags.is_empty() {
            panic!("cannot serialize when there are active tasks running");
        }

        SerializedTasksOrdered {
            items: self.iterator.collect(),
            f: FuncRef::new(self.f),
            max: self.max,
            process: self.process,
        }
    }
}

impl<C, M, S> SerializedTasksOrdered<C, M, S>
where
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    /// Deserialize [`SerializedTasksOrdered`] back into [`TasksOrdered`] for use as an iterator.
    pub fn deserialize(self) -> TasksOrdered<vec::IntoIter<C>, C, M, S> {
        TasksOrdered {
            iterator: self.items.into_iter(),
            f: self.f.get(),
            max: self.max,
            process: self.process,
            tags: VecDeque::with_capacity(self.max),
        }
    }
}

impl<I, C, M, S> Iterator for TasksOrdered<I, C, M, S>
where
    I: Iterator<Item = C>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let num_to_buffer = self.max - self.tags.len();
        for _ in 0..num_to_buffer {
            let Some(capture) = self.iterator.next() else { break };

            let tag = Tag::new();
            Process::<_, S>::spawn_link(
                (Process::this(), tag, capture, FuncRef::new(self.f)),
                |(parent, tag, capture, entry), _: Mailbox<(), S>| {
                    let result = entry(capture);
                    parent.tag_send(tag, result);
                },
            );
            self.tags.push_back(tag);
        }
        match self.tags.pop_front() {
            Some(tag) => {
                let mailbox: Mailbox<M, S> = unsafe { Mailbox::new() };
                let item = mailbox.tag_receive(&[tag]);
                Some(item)
            }
            None => None,
        }
    }
}
