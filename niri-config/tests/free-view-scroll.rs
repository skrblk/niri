use niri_config::Config;

#[test]
fn accepts_free_view_scroll_option() {
    let parsed = Config::parse_mem("gestures {\n    free-view-scroll\n}\n").unwrap();
    assert!(parsed.gestures.free_view_scroll);
}

#[test]
fn free_scroll_defaults_off_and_can_be_explicitly_disabled() {
    assert!(!Config::parse_mem("").unwrap().gestures.free_view_scroll);
    let parsed = Config::parse_mem("gestures {\n    free-view-scroll false\n}\n").unwrap();
    assert!(!parsed.gestures.free_view_scroll);
}

#[test]
fn free_scroll_rejects_non_boolean_arguments() {
    assert!(Config::parse_mem("gestures {\n    free-view-scroll 5\n}\n").is_err());
}

#[test]
fn free_scroll_momentum_is_seconds_from_zero_to_ten() {
    assert_eq!(
        Config::parse_mem("")
            .unwrap()
            .gestures
            .free_view_scroll_momentum,
        0.
    );
    let set = Config::parse_mem(
        "gestures {\n    free-view-scroll\n    free-view-scroll-momentum 0.25\n}\n",
    )
    .unwrap();
    assert!(set.gestures.free_view_scroll);
    assert_eq!(set.gestures.free_view_scroll_momentum, 0.25);
    assert_eq!(
        Config::parse_mem("gestures {\n    free-view-scroll-momentum 1\n}\n")
            .unwrap()
            .gestures
            .free_view_scroll_momentum,
        1.
    );
    for bad in ["", " -0.5", " 11", " true"] {
        let text = format!("gestures {{\n    free-view-scroll-momentum{bad}\n}}\n");
        assert!(Config::parse_mem(&text).is_err(), "accepted: {text}");
    }
}
