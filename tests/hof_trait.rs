use rust_key_paths::{HofTrait, Kp, KpType};

#[derive(Debug)]
struct Root {
    nums: Vec<i32>,
    enabled: bool,
}

fn nums_kp() -> KpType<'static, Root, Vec<i32>> {
    Kp::new(
        |r: &Root| if r.enabled { Some(&r.nums) } else { None },
        |r: &mut Root| if r.enabled { Some(&mut r.nums) } else { None },
    )
}

fn root_enabled() -> Root {
    Root {
        nums: vec![1, 2, 3, 4],
        enabled: true,
    }
}

fn root_disabled() -> Root {
    Root {
        nums: vec![1, 2, 3, 4],
        enabled: false,
    }
}

#[test]
fn hof_map_returns_mapped_value_and_none_edge_case() {
    let re = root_enabled();
    let rd = root_disabled();
    let kp = nums_kp();
    let map_len = kp.map(|v| v.len());
    let v1 = map_len.get(&re);
    let v2 = map_len.get(&rd);
    assert_eq!(v1, Some(4));
    assert_eq!(v2, None);
}

#[test]
fn hof_filter_map_returns_some_or_none_edge_case() {
    let re = root_enabled();
    let rd = root_disabled();
    let kp = nums_kp();
    let first_even = kp.filter_map(|v| v.iter().copied().find(|n| n % 2 == 0));
    assert_eq!(first_even.get(&re), Some(2));
    assert_eq!(first_even.get(&rd), None);
}

#[test]
fn hof_filter_returns_kp_and_handles_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let small_root = Root {
        nums: vec![1, 2],
        enabled: true,
    };
    let kp = nums_kp();
    let filtered = kp.filter(|v| v.len() > 3);

    assert_eq!(filtered.get(&enabled).map(|v| v.len()), Some(4));
    assert_eq!(filtered.get(&disabled).map(|v| v.len()), None);
    assert_eq!(filtered.get(&small_root).map(|v| v.len()), None);
}

#[test]
fn hof_inspect_runs_side_effect_and_none_edge_case() {
    use std::cell::Cell;
    use std::rc::Rc;

    let re = root_enabled();
    let rd = root_disabled();
    let calls = Rc::new(Cell::new(0));
    let calls_in_closure = Rc::clone(&calls);
    let kp = nums_kp();
    let inspect = kp.inspect(move |_| calls_in_closure.set(calls_in_closure.get() + 1));

    assert_eq!(inspect.get(&re).map(|v| v.len()), Some(4));
    assert_eq!(calls.get(), 1);

    assert_eq!(inspect.get(&rd).map(|v| v.len()), None);
    assert_eq!(calls.get(), 1);
}

#[test]
fn hof_flat_map_flattens_iterator_and_empty_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let flat = kp.flat_map(|v| v.iter().copied().filter(|n| n % 2 == 0).collect::<Vec<_>>());
    assert_eq!(flat(&enabled), vec![2, 4]);
    assert_eq!(flat(&disabled), Vec::<i32>::new());
}

#[test]
fn hof_fold_value_folds_and_returns_init_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let fold = kp.fold_value(10, |acc, v| acc + v.iter().sum::<i32>());
    assert_eq!(fold(&enabled), 20);
    assert_eq!(fold(&disabled), 10);
}

#[test]
fn hof_any_checks_predicate_and_false_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let any_gt_three = kp.any(|v| v.iter().any(|n| *n > 3));
    assert!(any_gt_three(&enabled));
    assert!(!any_gt_three(&disabled));
}

#[test]
fn hof_all_checks_predicate_and_true_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let all_positive = kp.all(|v| v.iter().all(|n| *n > 0));
    assert!(all_positive(&enabled));
    assert!(all_positive(&disabled));
}

#[test]
fn hof_count_items_counts_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let count = kp.count_items(|v| v.len());
    assert_eq!(count(&enabled), Some(4));
    assert_eq!(count(&disabled), None);
}

#[test]
fn hof_find_in_finds_value_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let find = kp.find_in(|v| v.iter().copied().find(|n| *n == 3));
    assert_eq!(find(&enabled), Some(3));
    assert_eq!(find(&disabled), None);
}

#[test]
fn hof_take_takes_n_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let take_two = kp.take(2, |v, n| v.iter().take(n).copied().collect::<Vec<_>>());
    assert_eq!(take_two(&enabled), Some(vec![1, 2]));
    assert_eq!(take_two(&disabled), None);
}

#[test]
fn hof_skip_skips_n_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let skip_two = kp.skip(2, |v, n| v.iter().skip(n).copied().collect::<Vec<_>>());
    assert_eq!(skip_two(&enabled), Some(vec![3, 4]));
    assert_eq!(skip_two(&disabled), None);
}

#[test]
fn hof_partition_value_partitions_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let partition =
        kp.partition_value(|v| v.iter().copied().partition::<Vec<_>, _>(|n| n % 2 == 0));
    assert_eq!(
        partition(&enabled),
        Some((vec![2, 4], vec![1, 3]))
    );
    assert_eq!(partition(&disabled), None);
}

#[test]
fn hof_min_value_returns_min_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let min = kp.min_value(|v| v.iter().min().copied());
    assert_eq!(min(&enabled), Some(1));
    assert_eq!(min(&disabled), None);
}

#[test]
fn hof_max_value_returns_max_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let max = kp.max_value(|v| v.iter().max().copied());
    assert_eq!(max(&enabled), Some(4));
    assert_eq!(max(&disabled), None);
}

#[test]
fn hof_sum_value_returns_sum_and_none_edge_case() {
    let enabled = root_enabled();
    let disabled = root_disabled();
    let kp = nums_kp();
    let sum = kp.sum_value(|v| v.iter().sum::<i32>());
    assert_eq!(sum(&enabled), Some(10));
    assert_eq!(sum(&disabled), None);
}

#[test]
fn hof_trait_keeps_readable_get_set_behavior() {
    let mut root = root_enabled();
    let kp = nums_kp();

    assert_eq!(kp.get(&root), Some(&vec![1, 2, 3, 4]));
    if let Some(nums) = rust_key_paths::Writable::set(&kp, &mut root) {
        nums.push(5);
    }
    assert_eq!(kp.get(&root), Some(&vec![1, 2, 3, 4, 5]));
}
