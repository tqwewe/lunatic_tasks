use std::time::Duration;

use lunatic_tasks::TaskExt;

fn main() {
    let tasks = [5, 5, 2, 2, 3, 3, 4, 4, 5, 5]
        .into_iter()
        .map(|num| {
            (num, |num: i32| -> i32 {
                lunatic::sleep(Duration::from_millis(num as u64 * 200));
                println!("{num}");
                num
            } as fn(_) -> _)
        })
        .tasks_ordered(3);
    for result in tasks {
        println!("got result {result}");
    }
}
