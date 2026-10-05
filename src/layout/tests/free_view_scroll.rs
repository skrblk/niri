use super::*;

fn options(enabled: bool, centered: bool) -> Options {
    let config = Config::parse_mem(&format!(
        r#"
        layout {{
            gaps 0
            default-column-width {{ fixed 400; }}
            center-focused-column "{}"
        }}
        "#,
        if centered { "always" } else { "never" }
    ))
    .unwrap();
    let mut options = Options::from_config(&config);
    options.gestures.free_view_scroll = enabled;
    options
}

fn fixture(enabled: bool, centered: bool, count: usize) -> Layout<TestWindow> {
    let mut ops = vec![Op::AddOutput(1)];
    for id in 1..=count {
        let mut params = TestWindowParams::new(id);
        params.bbox = Rectangle::from_size(Size::from((400, 200)));
        ops.push(Op::AddWindow { params });
        ops.push(Op::Refresh { is_active: true });
        ops.push(Op::Communicate(id));
    }
    ops.push(Op::FocusColumnFirst);
    ops.push(Op::CompleteAnimations);
    check_ops_with_options(options(enabled, centered), ops)
}

fn pan(layout: &mut Layout<TestWindow>, delta: f64, touchpad: bool) {
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureBegin {
            output_idx: 1,
            workspace_idx: None,
            is_touchpad: touchpad,
        }],
    );
    check_ops_on_layout(layout, [Op::AdvanceAnimations { msec_delta: 16 }]);
    let timestamp = layout.clock.now_unadjusted();
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureUpdate {
            delta,
            timestamp,
            is_touchpad: touchpad,
        }],
    );
    // Hold still before release: no residual velocity, so the expected endpoint is literal.
    check_ops_on_layout(
        layout,
        [
            Op::AdvanceAnimations { msec_delta: 500 },
            Op::ViewOffsetGestureEnd {
                is_touchpad: Some(touchpad),
            },
            Op::CompleteAnimations,
        ],
    );
}

fn position(layout: &Layout<TestWindow>) -> f64 {
    layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .target_view_pos()
}

#[test]
fn mouse_pan_keeps_a_partial_column_position() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    assert!(
        (position(&layout) - 125.).abs() < 0.001,
        "position: {}",
        position(&layout)
    );
}

#[test]
fn touchpad_pan_keeps_a_partial_column_position() {
    let mut layout = fixture(true, false, 6);
    // 117.1875 * 1280 / 1200 = 125 logical pixels.
    pan(&mut layout, 117.1875, true);
    assert!(
        (position(&layout) - 125.).abs() < 0.001,
        "position: {}",
        position(&layout)
    );
}

#[test]
fn disabled_option_preserves_upstream_snapping() {
    let mut layout = fixture(false, false, 6);
    pan(&mut layout, 125., false);
    assert_eq!(position(&layout), 0.);
}

#[test]
fn free_scroll_respects_both_workspace_edges() {
    for (delta, expected) in [(-100_000., 0.), (100_000., 1120.)] {
        let mut layout = fixture(true, false, 6);
        pan(&mut layout, delta, false);
        assert_eq!(position(&layout), expected);
    }
}

#[test]
fn free_scroll_respects_centered_columns() {
    let mut layout = fixture(true, true, 6);
    assert_eq!(position(&layout), -440.);
    pan(&mut layout, 125., false);
    assert!((position(&layout) + 315.).abs() < 0.001);
}

#[test]
fn free_scroll_handles_a_single_column_and_empty_workspace() {
    for count in [0, 1] {
        let mut layout = fixture(true, false, count);
        pan(&mut layout, 125., false);
        assert!(position(&layout).is_finite());
        layout.verify_invariants();
    }
}

#[test]
fn config_update_does_not_erase_the_free_position() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    layout.update_options(options(true, false));
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(position(&layout), 125.);
}

#[test]
fn window_resize_commit_does_not_erase_the_free_position() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    let active = *layout.focus().unwrap().id();
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetColumnWidth(SizeChange::SetFixed(390)),
            Op::Refresh { is_active: true },
            Op::Communicate(active),
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), 125.);
}

#[test]
fn fullscreen_still_aligns_and_unfullscreen_restores_the_position() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    let active = *layout.focus().unwrap().id();
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetFullscreenWindow {
                window: active,
                is_fullscreen: true,
            },
            Op::Refresh { is_active: true },
            Op::Communicate(active),
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(
        layout
            .active_workspace()
            .unwrap()
            .scrolling()
            .view_offset()
            .target(),
        0.
    );
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetFullscreenWindow {
                window: active,
                is_fullscreen: false,
            },
            Op::Refresh { is_active: true },
            Op::Communicate(active),
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), 125.);
}

#[test]
fn fullscreen_gesture_still_snaps() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetFullscreenWindow {
                window: 1,
                is_fullscreen: true,
            },
            Op::Refresh { is_active: true },
            Op::Communicate(1),
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);
    assert_eq!(position(&layout), 0.);
}

#[test]
fn disabling_free_scroll_restores_gesture_snapping() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    layout.update_options(options(false, false));
    pan(&mut layout, 0., false);
    assert_eq!(position(&layout), 0.);
}

#[test]
fn focusing_another_column_keeps_upstream_behavior() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    assert_eq!(position(&layout), 1120.);
}

#[test]
fn shrinking_output_keeps_the_active_column_visible() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 100_000., false);
    assert_eq!(position(&layout), 1120.);
    let output = layout.active_output().unwrap().clone();
    output.change_current_state(
        Some(Mode {
            size: Size::from((400, 720)),
            refresh: 60000,
        }),
        None,
        None,
        None,
    );
    smithay::desktop::layer_map_for_output(&output).arrange();
    layout.update_output_size(&output);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(position(&layout), 2000.);
}

#[test]
fn shrinking_a_wide_column_does_not_leave_a_blank_view() {
    let mut layout = fixture(true, false, 1);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetColumnWidth(SizeChange::SetFixed(2000)),
            Op::Refresh { is_active: true },
            Op::Communicate(1),
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 700., false);
    assert_eq!(position(&layout), 700.);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetColumnWidth(SizeChange::SetFixed(400)),
            Op::Refresh { is_active: true },
            Op::Communicate(1),
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), 0.);
}
