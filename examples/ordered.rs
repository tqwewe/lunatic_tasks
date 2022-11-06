use std::time::Duration;

use lunatic_tasks::TaskExt;

fn main() {
    let tasks = (0..10).tasks_ordered(3, |num| {
        lunatic::sleep(Duration::from_millis(num as u64 * 200));
        println!("{num}");
        num
    });
    for result in tasks {
        println!("got result {result}");
    }
}
