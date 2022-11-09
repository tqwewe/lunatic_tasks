use std::time::Duration;

use lunatic::spawn_link;
use lunatic_tasks::TaskExt;

fn main() {
    spawn_link!(|| {
        let tasks = (1..10).tasks_ordered(3, |num| {
            (0..num)
                .tasks_unordered(2, |num| num + 10)
                .collect::<Vec<_>>()
        });
        for tasks in tasks {
            for task in tasks {
                println!("got result {task}");
            }
        }
    });
    lunatic::sleep(Duration::from_millis(500));
}
