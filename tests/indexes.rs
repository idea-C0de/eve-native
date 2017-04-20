extern crate eve;
use eve::indexes::*;
use eve::ops::{EstimateIterPool, RoundHolder, Change};
use std::collections::HashMap;

#[test]
fn basic_hash_index() {
    let mut index = HashIndex::new();
    index.insert(1,1,1);
    index.insert(1,2,1);
    index.insert(2,3,1);
    index.insert(1,3,100);
    assert!(index.check(1,1,1));
    assert!(index.check(1,2,1));
    assert!(index.check(2,3,1));
    assert!(index.check(1,3,100));
    assert!(!index.check(100,300,100));
}

#[test]
fn basic2() {
    let mut index = HashIndex::new();
    index.insert(5,3,8);
    index.insert(9,3,8);
    assert!(index.check(5,3,8));
    assert!(index.check(9,3,8));
    assert!(!index.check(100,300,100));
}

#[test]
fn find_entities() {
    let mut index = HashIndexLevel::new();
    index.insert(1,1);
    index.insert(2,1);
    index.insert(300,1);
    let entities = index.get(0, 1).unwrap();
    assert!(entities.contains(&1));
    assert!(entities.contains(&2));
    assert!(entities.contains(&300));
    assert!(!entities.contains(&3));
}

#[test]
fn find_values() {
    let mut index = HashIndexLevel::new();
    index.insert(1,1);
    index.insert(1,2);
    index.insert(1,300);
    {
        let values = index.get(1, 0).unwrap();
        assert!(values.contains(&1));
        assert!(values.contains(&2));
        assert!(values.contains(&300));
        assert!(!values.contains(&3));
    }

    index.insert(5,8);
    index.insert(9,8);
    let values2 = index.get(9, 0).unwrap();
    assert!(values2.contains(&8));
}

#[test]
fn basic_propose() {
    let mut index = HashIndex::new();
    let mut pool = EstimateIterPool::new();
    index.insert(1,1,1);
    index.insert(2,1,1);
    index.insert(2,1,7);
    index.insert(3,1,1);
    index.insert(2,3,1);
    index.insert(1,3,100);
    let mut proposal1 = pool.get();
    index.propose(&mut proposal1, 0,1,1);
    assert_eq!(proposal1.estimate(), 3);
    let mut proposal2 = pool.get();
    index.propose(&mut proposal2, 2,1,0);
    assert_eq!(proposal2.estimate(), 2);
}


//---------------------------------------------------------------
// Distinct index
//---------------------------------------------------------------

fn round_counts_to_changes(counts: Vec<(u32, i32)>) -> Vec<Change> {
    let mut changes = vec![];
    let cur = Change { e: 1, a: 2, v: 3, n: 4, transaction: 1, round: 0, count: 0 };
    for &(round, count) in counts.iter() {
        changes.push(cur.with_round_count(round, count));
    }
    changes
}

fn test_distinct(counts: Vec<(u32, i32)>, expected: Vec<(u32, i32)>) {
    let mut index = HashIndex::new();
    let changes = round_counts_to_changes(counts);

    let mut final_results: HashMap<u32, i32> = HashMap::new();
    let mut distinct_changes = RoundHolder::new();
    for change in changes.iter() {
        index.distinct(change, &mut distinct_changes);
    }
    let mut iter = distinct_changes.iter();
    while let Some(distinct) = iter.next(&mut distinct_changes) {
        println!("distinct: {:?}", distinct);
        let cur = if final_results.contains_key(&distinct.round) { final_results[&distinct.round] } else { 0 };
        final_results.insert(distinct.round, cur + distinct.count);
    }

    for (round, count) in index.distinct_iter(changes[0].e, changes[0].a, changes[0].v) {
        let valid = match final_results.get(&round) {
            Some(&actual) => actual == count,
            None => count == 0,
        };
        assert!(valid, "iterator round {:?} :: expected {:?}, actual {:?}", round, count, final_results.get(&round));
    }

    println!("final {:?}", final_results);

    let mut expected_map = HashMap::new();
    for &(round, count) in expected.iter() {
        expected_map.insert(round, count);
        let valid = match final_results.get(&round) {
            Some(&actual) => actual == count,
            None => count == 0,
        };
        assert!(valid, "round {:?} :: expected {:?}, actual {:?}", round, count, final_results.get(&round));
    }

    for (round, count) in final_results.iter() {
        let valid = match expected_map.get(&round) {
            Some(&actual) => actual == *count,
            None => *count == 0,
        };
        assert!(valid, "round {:?} :: expected {:?}, actual {:?}", round, expected_map.get(&round), count);
    }

}

#[test]
fn basic_distinct() {
    test_distinct(vec![
                  (1,1),
                  (2,-1),

                  (1, 1),
                  (3, -1),
    ], vec![
    (1, 1),
    (3, -1)
    ])
}

#[test]
fn basic_2() {
    test_distinct(vec![
                  (1,1),
                  (2,-1),

                  (3, 1),
                  (4, -1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_2_reverse_order() {
    test_distinct(vec![
                  (3,1),
                  (4,-1),

                  (1, 1),
                  (2, -1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_2_undone() {
    test_distinct(vec![
                  (1,1),
                  (2,-1),

                  (3, 1),
                  (4, -1),

                  (1,-1),
                  (2,1),
    ], vec![
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_multiple() {
    test_distinct(vec![
                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),

                  (3, 1),
                  (3, 1),
                  (3, 1),
                  (4, -1),
                  (4, -1),
                  (4, -1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_multiple_reversed() {
    test_distinct(vec![
                  (3, 1),
                  (3, 1),
                  (3, 1),
                  (4, -1),
                  (4, -1),
                  (4, -1),

                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_interleaved() {
    test_distinct(vec![
                  (3, 1),
                  (4, -1),
                  (3, 1),
                  (4, -1),
                  (3, 1),
                  (4, -1),

                  (1,1),
                  (2,-1),
                  (1,1),
                  (2,-1),
                  (1,1),
                  (2,-1),
                  (1,1),
                  (2,-1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_multiple_negative_first() {
    test_distinct(vec![
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (1,1),
                  (1,1),
                  (1,1),

                  (4, -1),
                  (4, -1),
                  (4, -1),
                  (3, 1),
                  (3, 1),
                  (3, 1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_multiple_undone() {
    test_distinct(vec![
                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),

                  (3, 1),
                  (3, 1),
                  (3, 1),
                  (4, -1),
                  (4, -1),
                  (4, -1),

                  (1,-1),
                  (1,-1),
                  (1,-1),
                  (1,-1),
                  (2,1),
                  (2,1),
                  (2,1),
                  (2,1),
                  ], vec![
                      (3, 1),
                      (4, -1),
                  ])
}

#[test]
fn basic_multiple_undone_interleaved() {
    test_distinct(vec![
                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),

                  (1,-1),
                  (1,-1),
                  (1,-1),
                  (1,-1),

                  (3, 1),
                  (3, 1),
                  (3, 1),
                  (4, -1),
                  (4, -1),
                  (4, -1),

                  (2,1),
                  (2,1),
                  (2,1),
                  (2,1),
                  ], vec![
                      (3, 1),
                      (4, -1),
                  ])
}

#[test]
fn basic_multiple_different_counts() {
    test_distinct(vec![
                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),

                  (3, 1),
                  (4, -1),
    ], vec![
    (1, 1),
    (2, -1),
    (3, 1),
    (4, -1),
    ])
}

#[test]
fn basic_multiple_different_counts_extra_removes() {
    test_distinct(vec![
                  (1,1),
                  (1,1),
                  (1,1),
                  (1,1),
                  (2,-1),
                  (2,-1),
                  (2,-1),
                  (2,-1),

                  (1,-1),
                  (1,-1),
                  (1,-1),
                  (1,-1),
                  (2,1),
                  (2,1),
                  (2,1),
                  (2,1),

                  (3, 1),
                  (4, -1),
                  ], vec![
                      (3, 1),
                      (4, -1),
                  ])
}

#[test]
fn simple_round_promotion() {
    test_distinct(vec![
                  (8,1),
                  (9,-1),

                  (5,1),
                  (6,-1),
                  (8,-1),
                  (9,1),
    ], vec![
    (5, 1),
    (6, -1)
    ])
}

#[test]
fn full_promotion() {
    test_distinct(vec![
                  (9,1),
                  (9,1),
                  (10,-1),
                  (10,-1),

                  (9,1),
                  (9,1),
                  (10,-1),
                  (10,-1),

                  (9,-1),
                  (10,1),
                  (9,-1),
                  (10,1),

                  (9,-1),
                  (10,1),
                  (9,-1),
                  (10,1),
    ], vec![
        (9, 0),
        (10, 0)
    ])
}

#[test]
fn positive_full_promotion() {
    test_distinct(vec![
                  (7,1),
                  (8,-1),
                  (8,1),
                  (7,1),
                  (8,-1),
                  (4,1),
                  (8, -1),
                  (7, 1),
                  (8, -1),
                  (8, 1),
                  (5, -1),
                  (7, -3),
                  (8, 1),
                  (8, 3),
                  (5, 1),
                  (8, 1),
                  (8, -2),
                  (8, -1),
    ], vec![
    (4, 1),
    ])
}
