use std::time::Duration;

use lunatic_tasks::TaskExt;

fn main() {
    let mut tasks = (0..5).rev().tasks_unordered(3, |num| {
        lunatic::sleep(Duration::from_millis(num as u64 * 200));
        num
    });
    assert_eq!(tasks.next(), Some(2)); // 4, 3, 2
    assert_eq!(tasks.next(), Some(3)); // 4, 3, 1
    assert_eq!(tasks.next(), Some(0)); // 4, 0, 1
    assert_eq!(tasks.next(), Some(1)); // 4, 1
    assert_eq!(tasks.next(), Some(4)); // 4
    assert_eq!(tasks.next(), None);
}
