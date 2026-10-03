//! Preserve paragraph-cache contracts while supporting Rust 1.93 and 1.99.

use rhwp::model::paragraph::SingleLineOverflowMemo;

#[test]
fn width_memo_updates_preserve_stored_partition_invalidation() {
    let memo = SingleLineOverflowMemo::default();
    let width = SingleLineOverflowMemo::width_key(120.0);
    memo.set(width, true);
    assert_eq!(memo.get(width), Some(true));
    assert!(!memo.stored_partition_is_dirty());

    memo.invalidate_layout_inputs();
    assert_eq!(memo.get(width), None);
    memo.set(width, false);
    memo.set(0, true);
    assert_eq!(memo.get(width), Some(false));
    assert!(memo.stored_partition_is_dirty());

    memo.clear();
    assert!(memo.is_unjudged());
    assert!(memo.stored_partition_is_dirty());
}

#[test]
fn concurrent_width_memo_updates_keep_partition_provenance() {
    let memo = SingleLineOverflowMemo::default();
    memo.invalidate_layout_inputs();
    std::thread::scope(|scope| {
        for worker in 0..4 {
            let memo = &memo;
            scope.spawn(move || {
                for iteration in 0..1_000 {
                    let width = SingleLineOverflowMemo::width_key(100.0 + worker as f64);
                    memo.set(width, iteration % 2 == 0);
                }
            });
        }
    });
    assert!(memo.stored_partition_is_dirty());
    let width = SingleLineOverflowMemo::width_key(150.0);
    memo.set(width, true);
    assert_eq!(memo.get(width), Some(true));
    assert!(memo.stored_partition_is_dirty());
}
