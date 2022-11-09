use std::vec;

use lunatic::{
    function::FuncRef,
    protocol::ProtocolCapture,
    serializer::{Bincode, Serializer},
    Mailbox, Process, Tag,
};
use serde::{Deserialize, Serialize};

/// Iterator for the [`tasks_unordered`](super::TaskExt::tasks_unordered) method.
pub struct TasksUnordered<I, C, M, S = Bincode>
where
    I: Iterator<Item = C>,
    C: Serialize + for<'de> Deserialize<'de>,
    M: Serialize + for<'de> Deserialize<'de>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    iterator: I,
    f: fn(C) -> M,
    max: usize,
    process: Process<(Tag, M), S>,
    tags: Vec<Tag>,
}

/// A serialized instance of [`TasksUnordered`].
#[derive(Serialize, Deserialize)]
#[serde(bound(serialize = "C: Serialize", deserialize = "C: for<'d> Deserialize<'d>"))]
pub struct SerializedTasksUnordered<C, M, S = Bincode> {
    items: Vec<C>,
    f: FuncRef<fn(C) -> M>,
    max: usize,
    process: Process<(Tag, M), S>,
}

impl<I, C, M, S> TasksUnordered<I, C, M, S>
where
    I: Iterator<Item = C>,
    C: Serialize + for<'de> Deserialize<'de>,
    M: Serialize + for<'de> Deserialize<'de>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    pub(super) fn new<T>(iterator: T, n: usize, f: fn(C) -> M) -> Self
    where
        T: IntoIterator<IntoIter = I>,
    {
        TasksUnordered {
            iterator: iterator.into_iter(),
            f,
            max: n,
            process: Process::this(),
            tags: Vec::with_capacity(n),
        }
    }

    /// Collects inner iterator into vec to serialize [`TasksUnordered`].
    ///
    /// Panics if active tasks are running.
    /// Tasks begin running when the iterator is consumed.
    pub fn serialize(self) -> SerializedTasksUnordered<C, M, S>
    where
        C: Serialize + for<'de> Deserialize<'de>,
    {
        if !self.tags.is_empty() {
            panic!("cannot serialize when there are active tasks running");
        }

        SerializedTasksUnordered {
            items: self.iterator.collect(),
            f: FuncRef::new(self.f),
            max: self.max,
            process: self.process,
        }
    }
}

impl<C, M, S> SerializedTasksUnordered<C, M, S>
where
    C: Serialize + for<'de> Deserialize<'de>,
    M: Serialize + for<'de> Deserialize<'de>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    /// Deserialize [`SerializedTasksUnordered`] back into [`TasksUnordered`] for use as an iterator.
    pub fn deserialize(self) -> TasksUnordered<vec::IntoIter<C>, C, M, S> {
        TasksUnordered {
            iterator: self.items.into_iter(),
            f: self.f.get(),
            max: self.max,
            process: self.process,
            tags: Vec::with_capacity(self.max),
        }
    }
}

impl<I, C, M, S> Iterator for TasksUnordered<I, C, M, S>
where
    I: Iterator<Item = C>,
    C: Serialize + for<'de> Deserialize<'de>,
    M: Serialize + for<'de> Deserialize<'de>,
    S: Serializer<()>
        + Serializer<M>
        + Serializer<(Process<(Tag, M), S>, Tag, C, FuncRef<fn(C) -> M>)>
        + Serializer<ProtocolCapture<(Process<M, S>, Tag, C, FuncRef<fn(C) -> M>)>>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let num_to_buffer = self.max - self.tags.len();
        for _ in 0..num_to_buffer {
            let Some(capture) = self.iterator.next() else { break };

            let tag = Tag::new();
            Process::spawn_link(
                (
                    Process::<(Tag, M)>::this(),
                    tag,
                    capture,
                    FuncRef::new(self.f),
                ),
                |(parent, tag, capture, entry), _: Mailbox<()>| {
                    let result = entry(capture);
                    parent.tag_send(tag, (tag, result));
                },
            );
            self.tags.push(tag);
        }
        if self.tags.is_empty() {
            return None;
        }

        let mailbox: Mailbox<(Tag, M)> = unsafe { Mailbox::new() };
        let (tag, item) = mailbox.tag_receive(&self.tags);
        self.tags.retain(|t| t != &tag);
        Some(item)
    }
}
