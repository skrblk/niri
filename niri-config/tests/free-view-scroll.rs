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
