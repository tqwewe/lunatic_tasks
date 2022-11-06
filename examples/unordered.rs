use std::time::Duration;

use lunatic_tasks::TaskExt;

fn main() {
    let tasks = [5, 5, 2, 2, 3, 3, 4, 4, 5, 5]
        .into_iter()
        .tasks_unordered(3, |num| {
            lunatic::sleep(Duration::from_millis(num as u64 * 200));
            println!("{num}");
            num
        });
    for result in tasks {
        println!("got result {result}");
    }
}
