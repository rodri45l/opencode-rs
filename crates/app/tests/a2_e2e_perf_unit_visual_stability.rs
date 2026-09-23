//! Port of packages/app/e2e/performance/unit/visual-stability.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
struct Region {
    present: Option<bool>,
    visible: Option<bool>,
    in_viewport: Option<bool>,
    top: Option<i64>,
    bottom: Option<i64>,
    width: Option<i64>,
    height: Option<i64>,
    opacity: Option<f64>,
    count: Option<i64>,
    node: Option<i64>,
    label: Option<String>,
    text: Option<String>,
    layout_top: Option<i64>,
    layout_bottom: Option<i64>,
    css_hidden: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Viewport {
    top: i64,
    bottom: i64,
    scroll_top: i64,
    scroll_height: i64,
    client_height: i64,
    distance_from_bottom: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Sample {
    at: i64,
    regions: HashMap<String, Region>,
    viewport: Option<Viewport>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Marker {
    at: i64,
    label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VisualStabilityTrace {
    markers: Vec<Marker>,
    samples: Vec<Sample>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VisualOptions {
    flow: Vec<String>,
    stable: Vec<String>,
    unique: Vec<String>,
    fixed: Vec<String>,
    motion: Vec<String>,
    required: Vec<String>,
    continuous_any: Vec<Vec<String>>,
    preserve_bottom_anchor: bool,
    acquire_bottom_anchor: bool,
    max_position_reversals: Option<i64>,
}

fn frame(at: i64, changing: Region, following: Option<Region>) -> Sample {
    let mut regions = HashMap::new();
    regions.insert("changing".to_string(), changing);
    if let Some(following) = following {
        regions.insert("following".to_string(), following);
    }
    Sample {
        at,
        regions,
        viewport: None,
    }
}

fn region(overrides: Region) -> Region {
    Region {
        present: Some(true),
        visible: Some(true),
        in_viewport: Some(true),
        top: Some(overrides.top.unwrap_or(0)),
        bottom: Some(overrides.bottom.unwrap_or(20)),
        width: Some(100),
        height: Some(20),
        opacity: Some(1.0),
        count: Some(1),
        node: Some(1),
        label: Some(String::new()),
        text: Some(String::new()),
        layout_top: Some(overrides.top.unwrap_or(0)),
        layout_bottom: Some(overrides.bottom.unwrap_or(20)),
        ..overrides
    }
}

fn viewport(distance_from_bottom: i64) -> Viewport {
    Viewport {
        top: 0,
        bottom: 400,
        scroll_top: 100,
        scroll_height: 500,
        client_height: 400,
        distance_from_bottom,
    }
}

// Local stubs (fast wave): real module lands later.
fn analyze_visual_stability(
    _trace: &VisualStabilityTrace,
    _options: &VisualOptions,
) -> Vec<String> {
    Vec::new()
}

fn analyze_visual_stability_by_marker(
    _trace: &VisualStabilityTrace,
    _options: &VisualOptions,
) -> Vec<String> {
    Vec::new()
}

fn analyze_visual_observations(_samples: &[Sample], _plan: &VisualOptions) -> Vec<String> {
    Vec::new()
}

fn legacy_visual_plan(_options: &VisualOptions) -> VisualOptions {
    VisualOptions::default()
}

fn define_visual_regions(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn map_visual_regions(
    _regions: &[String],
    _selector: &dyn Fn(&str) -> String,
) -> HashMap<String, String> {
    HashMap::new()
}

fn visual_plan(_regions: &[String], _invariants: &[String], per_marker: bool) -> VisualOptions {
    VisualOptions::default().with_per_marker(per_marker)
}

impl VisualOptions {
    fn with_per_marker(mut self, per_marker: bool) -> VisualOptions {
        self.max_position_reversals = Some(if per_marker { 1 } else { 0 });
        self
    }
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn accepts_continuous_visible_motion() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(
                0,
                region(Region {
                    width: Some(80),
                    bottom: Some(40),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(40),
                    bottom: Some(60),
                    ..Default::default()
                })),
            ),
            frame(
                16,
                region(Region {
                    width: Some(75),
                    bottom: Some(45),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(45),
                    bottom: Some(65),
                    ..Default::default()
                })),
            ),
            frame(
                32,
                region(Region {
                    width: Some(70),
                    bottom: Some(50),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(50),
                    bottom: Some(70),
                    ..Default::default()
                })),
            ),
        ],
    };
    let options = VisualOptions {
        flow: vec!["changing".into(), "following".into()],
        ..Default::default()
    };
    assert_eq!(
        analyze_visual_stability(&trace, &options),
        Vec::<String>::new()
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_repeated_geometry_reversals() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(
                0,
                region(Region {
                    width: Some(80),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                16,
                region(Region {
                    width: Some(60),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                32,
                region(Region {
                    width: Some(78),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                48,
                region(Region {
                    width: Some(62),
                    ..Default::default()
                }),
                None,
            ),
        ],
    };
    let issues = analyze_visual_stability(&trace, &VisualOptions::default());
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing width reversed 2 times")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_visible_blanking_label_reversal_and_overlap() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(
                0,
                region(Region {
                    label: Some("Exploring".into()),
                    opacity: Some(1.0),
                    bottom: Some(40),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(40),
                    bottom: Some(60),
                    ..Default::default()
                })),
            ),
            frame(
                16,
                region(Region {
                    label: Some("Explored".into()),
                    opacity: Some(0.2),
                    bottom: Some(50),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(49),
                    bottom: Some(69),
                    ..Default::default()
                })),
            ),
            frame(
                32,
                region(Region {
                    label: Some("Exploring".into()),
                    opacity: Some(1.0),
                    bottom: Some(50),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(50),
                    bottom: Some(70),
                    ..Default::default()
                })),
            ),
        ],
    };
    let options = VisualOptions {
        flow: vec!["changing".into(), "following".into()],
        ..Default::default()
    };
    let issues = analyze_visual_stability(&trace, &options);
    assert!(issues
        .iter()
        .any(|issue| issue.contains("opacity fell to 0.2")));
    assert!(issues.iter().any(|issue| issue.contains("label reverted")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("overlapped following by 1px")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_duplicate_regions_and_unexpected_remounts() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(
                0,
                region(Region {
                    node: Some(1),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                16,
                region(Region {
                    node: Some(2),
                    count: Some(2),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                32,
                region(Region {
                    node: Some(2),
                    ..Default::default()
                }),
                None,
            ),
        ],
    };
    let options = VisualOptions {
        stable: vec!["changing".into()],
        unique: vec!["changing".into()],
        ..Default::default()
    };
    let issues = analyze_visual_stability(&trace, &options);
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing appeared 2 times")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing remounted")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_bottom_anchor_loss_but_permits_movement_while_scrolled_away() {
    let mut anchored_sample_0 = frame(0, region(Region::default()), None);
    anchored_sample_0.viewport = Some(viewport(0));
    let mut anchored_sample_1 = frame(16, region(Region::default()), None);
    anchored_sample_1.viewport = Some(viewport(24));
    let anchored = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples: vec![anchored_sample_0, anchored_sample_1],
        },
        &VisualOptions {
            preserve_bottom_anchor: true,
            ..Default::default()
        },
    );

    let mut away_sample_0 = frame(0, region(Region::default()), None);
    away_sample_0.viewport = Some(viewport(80));
    let mut away_sample_1 = frame(16, region(Region::default()), None);
    away_sample_1.viewport = Some(viewport(104));
    let away = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples: vec![away_sample_0, away_sample_1],
        },
        &VisualOptions {
            preserve_bottom_anchor: true,
            ..Default::default()
        },
    );

    assert!(anchored
        .iter()
        .any(|issue| issue.contains("bottom anchor moved to 24px")));
    assert_eq!(away, Vec::<String>::new());
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_up_down_up_movement_while_preserving_a_bottom_anchor() {
    let mut samples = Vec::new();
    for (at, top, bottom) in [
        (0, 200, 240),
        (16, 180, 220),
        (32, 196, 236),
        (48, 176, 216),
    ] {
        let mut sample = frame(
            at,
            region(Region {
                top: Some(top),
                bottom: Some(bottom),
                ..Default::default()
            }),
            None,
        );
        sample.viewport = Some(viewport(0));
        samples.push(sample);
    }
    let issues = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples,
        },
        &VisualOptions {
            preserve_bottom_anchor: true,
            max_position_reversals: Some(0),
            ..Default::default()
        },
    );
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing top reversed 2 times")));
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing bottom reversed 2 times")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn ignores_overlap_entirely_outside_the_clipped_timeline_viewport() {
    let mut sample = frame(
        0,
        region(Region {
            top: Some(-200),
            bottom: Some(-100),
            ..Default::default()
        }),
        Some(region(Region {
            top: Some(-150),
            bottom: Some(-50),
            ..Default::default()
        })),
    );
    sample.viewport = Some(viewport(0));
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![sample],
    };
    let options = VisualOptions {
        flow: vec!["changing".into(), "following".into()],
        ..Default::default()
    };
    assert_eq!(
        analyze_visual_stability(&trace, &options),
        Vec::<String>::new()
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_visible_anchor_movement_while_allowing_virtual_scrollbar_movement() {
    let mut first = frame(
        0,
        region(Region {
            top: Some(100),
            bottom: Some(120),
            ..Default::default()
        }),
        None,
    );
    first.regions.insert(
        "anchor".into(),
        region(Region {
            top: Some(100),
            bottom: Some(120),
            ..Default::default()
        }),
    );
    first.viewport = Some(Viewport {
        scroll_top: 40,
        ..viewport(100)
    });
    let mut second = frame(
        16,
        region(Region {
            top: Some(100),
            bottom: Some(120),
            ..Default::default()
        }),
        None,
    );
    second.regions.insert(
        "anchor".into(),
        region(Region {
            top: Some(100),
            bottom: Some(120),
            ..Default::default()
        }),
    );
    second.viewport = Some(Viewport {
        scroll_top: 60,
        ..viewport(120)
    });

    let issues = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples: vec![first, second],
        },
        &VisualOptions {
            fixed: vec!["anchor".into()],
            ..Default::default()
        },
    );
    assert_eq!(issues, Vec::<String>::new());
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn analyzes_each_marked_event_independently() {
    let input = VisualStabilityTrace {
        markers: vec![
            Marker {
                at: 10,
                label: "grow".into(),
            },
            Marker {
                at: 40,
                label: "shrink".into(),
            },
        ],
        samples: vec![
            frame(
                0,
                region(Region {
                    top: Some(100),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                16,
                region(Region {
                    top: Some(90),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                32,
                region(Region {
                    top: Some(80),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                48,
                region(Region {
                    top: Some(90),
                    ..Default::default()
                }),
                None,
            ),
            frame(
                64,
                region(Region {
                    top: Some(100),
                    ..Default::default()
                }),
                None,
            ),
        ],
    };
    assert!(analyze_visual_stability(
        &input,
        &VisualOptions {
            max_position_reversals: Some(0),
            ..Default::default()
        }
    )
    .contains(&"changing top reversed 1 times".to_string()));
    assert_eq!(
        analyze_visual_stability_by_marker(
            &input,
            &VisualOptions {
                max_position_reversals: Some(0),
                motion: vec!["changing".into()],
                ..Default::default()
            },
        ),
        Vec::<String>::new()
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_regions_rendered_in_the_wrong_flow_order() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![frame(
            0,
            region(Region {
                top: Some(100),
                bottom: Some(120),
                ..Default::default()
            }),
            Some(region(Region {
                top: Some(60),
                bottom: Some(80),
                ..Default::default()
            })),
        )],
    };
    let options = VisualOptions {
        flow: vec!["changing".into(), "following".into()],
        ..Default::default()
    };
    let issues = analyze_visual_stability(&trace, &options);
    assert!(issues
        .iter()
        .any(|issue| issue.contains("changing rendered after following")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_an_in_viewport_transparent_frame_between_visible_frames() {
    let trace = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(0, region(Region::default()), None),
            frame(
                16,
                region(Region {
                    visible: Some(false),
                    opacity: Some(0.0),
                    in_viewport: Some(true),
                    ..Default::default()
                }),
                None,
            ),
            frame(32, region(Region::default()), None),
        ],
    };
    let issues = analyze_visual_stability(&trace, &VisualOptions::default());
    assert!(issues
        .iter()
        .any(|issue| issue.contains("blanked between visible frames")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_a_blank_frame_across_replacement_surfaces() {
    let mut first = frame(0, region(Region::default()), None);
    first
        .regions
        .insert("thinking".into(), region(Region::default()));
    first.regions.insert(
        "error".into(),
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
    );
    let mut second = frame(16, region(Region::default()), None);
    second.regions.insert(
        "thinking".into(),
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
    );
    second.regions.insert(
        "error".into(),
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
    );
    let mut third = frame(32, region(Region::default()), None);
    third.regions.insert(
        "thinking".into(),
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
    );
    third
        .regions
        .insert("error".into(), region(Region::default()));

    let issues = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples: vec![first, second, third],
        },
        &VisualOptions {
            continuous_any: vec![vec!["thinking".into(), "error".into()]],
            ..Default::default()
        },
    );
    assert!(issues
        .iter()
        .any(|issue| issue.contains("thinking | error blanked")));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn reports_a_required_region_that_never_renders() {
    let mut sample = frame(
        0,
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
        None,
    );
    sample.regions.clear();
    sample.regions.insert(
        "changing".into(),
        region(Region {
            present: Some(false),
            visible: Some(false),
            ..Default::default()
        }),
    );
    let issues = analyze_visual_stability(
        &VisualStabilityTrace {
            markers: Vec::new(),
            samples: vec![sample],
        },
        &VisualOptions {
            required: vec!["changing".into()],
            ..Default::default()
        },
    );
    assert!(issues.contains(&"changing never rendered".to_string()));
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn preserves_typed_region_names_while_mapping_definitions() {
    let regions = define_visual_regions(&["changing", "following"]);
    let selectors = map_visual_regions(&regions, &|name| format!("[data-{name}]"));
    assert_eq!(
        selectors.get("changing"),
        Some(&"[data-changing]".to_string())
    );
    assert_eq!(
        selectors.get("following"),
        Some(&"[data-following]".to_string())
    );
}

#[test]
#[ignore = "porting: e2e/performance/unit visual-stability not implemented"]
fn legacy_plan_adapter_preserves_analyzer_messages_and_order() {
    let input = VisualStabilityTrace {
        markers: Vec::new(),
        samples: vec![
            frame(
                0,
                region(Region {
                    label: Some("Exploring".into()),
                    opacity: Some(1.0),
                    bottom: Some(40),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(40),
                    bottom: Some(60),
                    ..Default::default()
                })),
            ),
            frame(
                16,
                region(Region {
                    label: Some("Explored".into()),
                    opacity: Some(0.2),
                    bottom: Some(50),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(49),
                    bottom: Some(69),
                    ..Default::default()
                })),
            ),
            frame(
                32,
                region(Region {
                    label: Some("Exploring".into()),
                    opacity: Some(1.0),
                    bottom: Some(50),
                    ..Default::default()
                }),
                Some(region(Region {
                    top: Some(50),
                    bottom: Some(70),
                    ..Default::default()
                })),
            ),
        ],
    };
    let options = VisualOptions {
        flow: vec!["changing".into(), "following".into()],
        stable: vec!["changing".into()],
        ..Default::default()
    };
    assert_eq!(
        analyze_visual_observations(&input.samples, &legacy_visual_plan(&options)),
        analyze_visual_stability(&input, &options)
    );
}
