use lunatic::{function::FuncRef, serializer::Bincode, Mailbox, Process, Tag};
use serde::{Deserialize, Serialize};

/// Iterator for the [`tasks_unordered`](super::TaskExt::tasks_unordered) method.
pub struct TasksUnordered<I, C, M, S = Bincode>
where
    I: Iterator<Item = C>,
    M: 'static,
{
    iterator: I,
    f: fn(C) -> M,
    max: usize,
    process: Process<(Tag, M), S>,
    tags: Vec<Tag>,
}

impl<I, C, M, S> TasksUnordered<I, C, M, S>
where
    I: Iterator<Item = C>,
    M: 'static,
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
}

impl<I, C, M> Iterator for TasksUnordered<I, C, M>
where
    I: Iterator<Item = C>,
    C: Serialize + for<'de> Deserialize<'de>,
    M: 'static + Serialize + for<'de> Deserialize<'de>,
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        let num_to_buffer = self.max - self.tags.len();
        for _ in 0..num_to_buffer {
            let Some(capture) = self.iterator.next() else { break };

            let tag = Tag::new();
            Process::spawn_link(
                (self.process.clone(), tag, capture, FuncRef::new(self.f)),
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
