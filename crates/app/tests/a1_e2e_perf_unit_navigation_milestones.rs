//! Port of packages/app/e2e/performance/unit/navigation-milestones.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
struct Milestone {
    first_observed_ms: Option<i64>,
    stable_observed_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Summary {
    samples: usize,
    milestones: BTreeMap<String, Milestone>,
    all: Milestone,
}

// Local stub (fast wave): real module lands later.
fn summarize_navigation_milestones(_samples: &[(i64, Vec<(String, bool)>)]) -> Summary {
    Summary {
        samples: 0,
        milestones: BTreeMap::new(),
        all: Milestone {
            first_observed_ms: None,
            stable_observed_ms: None,
        },
    }
}

fn milestones(pairs: &[(&str, bool)]) -> Vec<(String, bool)> {
    pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
}

fn milestone(first: i64, stable: Option<i64>) -> Milestone {
    Milestone {
        first_observed_ms: Some(first),
        stable_observed_ms: stable,
    }
}

#[test]
#[ignore = "porting: e2e/performance/unit/navigation-milestones not implemented"]
fn reports_first_and_stable_paint_for_each_navigation_milestone() {
    let mut expected_milestones = BTreeMap::new();
    expected_milestones.insert("content".to_string(), milestone(32, Some(64)));
    expected_milestones.insert("tab".to_string(), milestone(48, Some(80)));
    assert_eq!(
        summarize_navigation_milestones(&[
            (16, milestones(&[("content", false), ("tab", false)])),
            (32, milestones(&[("content", true), ("tab", false)])),
            (48, milestones(&[("content", true), ("tab", true)])),
            (64, milestones(&[("content", true), ("tab", true)])),
            (80, milestones(&[("content", true), ("tab", true)])),
        ]),
        Summary {
            samples: 5,
            milestones: expected_milestones,
            all: milestone(48, Some(80)),
        }
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit/navigation-milestones not implemented"]
fn reports_missing_stability_when_a_milestone_appears_in_the_final_samples() {
    let mut expected_milestones = BTreeMap::new();
    expected_milestones.insert("content".to_string(), milestone(32, None));
    assert_eq!(
        summarize_navigation_milestones(&[
            (16, milestones(&[("content", false)])),
            (32, milestones(&[("content", true)])),
        ]),
        Summary {
            samples: 2,
            milestones: expected_milestones,
            all: milestone(32, None),
        }
    );
}
