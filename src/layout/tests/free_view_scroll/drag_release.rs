use super::*;

/// Output 1 is 1280 logical pixels wide; gaps are 0, so column positions are exact.
fn wide_options(width: u32) -> Options {
    let config = Config::parse_mem(&format!(
        r#"
        layout {{
            gaps 0
            default-column-width {{ fixed {width}; }}
            center-focused-column "never"
        }}
        "#
    ))
    .unwrap();
    let mut options = Options::from_config(&config);
    options.gestures.free_view_scroll = true;
    options
}

fn columns_of_width(count: usize, width: i32) -> Layout<TestWindow> {
    let mut ops = vec![Op::AddOutput(1)];
    for id in 1..=count {
        let mut params = TestWindowParams::new(id);
        params.bbox = Rectangle::from_size(Size::from((width, 200)));
        ops.push(Op::AddWindow { params });
        ops.push(Op::Refresh { is_active: true });
        ops.push(Op::Communicate(id));
    }
    ops.push(Op::CompleteAnimations);
    check_ops_with_options(wide_options(width as u32), ops)
}

fn view(layout: &Layout<TestWindow>) -> f64 {
    layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .target_view_pos()
}

fn settle(layout: &mut Layout<TestWindow>) {
    check_ops_on_layout(layout, [Op::CompleteAnimations]);
    layout.verify_invariants();
}

/// Drags by each delta in turn, 16 ms apart, then lets go at once: the pointer is still
/// moving, so the gesture has velocity when it ends.
fn drag_and_let_go_moving(layout: &mut Layout<TestWindow>, deltas: &[f64], touchpad: bool) {
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureBegin {
            output_idx: 1,
            workspace_idx: None,
            is_touchpad: touchpad,
        }],
    );
    for &delta in deltas {
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
    }
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureEnd {
            is_touchpad: Some(touchpad),
        }],
    );
}

#[test]
fn letting_go_mid_motion_keeps_the_view_where_it_was() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    drag_and_let_go_moving(&mut layout, &[40., 40., 45.], false);
    let scrolling = layout.active_workspace().unwrap().scrolling();
    // No glide afterwards: nothing is animating and the view sits at the dragged spot.
    assert!(!scrolling.view_offset().is_animation_ongoing());
    assert!(
        (view(&layout) - 125.).abs() < 0.001,
        "view: {}",
        view(&layout)
    );
    settle(&mut layout);
    assert!(
        (view(&layout) - 125.).abs() < 0.001,
        "view: {}",
        view(&layout)
    );
}

#[test]
fn letting_go_mid_motion_on_a_touchpad_keeps_the_view_too() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    // 3 × 39.0625 × 1280 / 1200 = 125 logical pixels.
    drag_and_let_go_moving(&mut layout, &[39.0625, 39.0625, 39.0625], true);
    settle(&mut layout);
    assert!(
        (view(&layout) - 125.).abs() < 0.001,
        "view: {}",
        view(&layout)
    );
}

#[test]
fn dragging_stops_at_the_last_column_and_turns_back_at_once() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    // Far past the end, then 50 back within the same drag.
    drag_and_let_go_moving(&mut layout, &[100_000., -50.], false);
    settle(&mut layout);
    // Last column right-aligned is 6 × 400 − 1280 = 1120; 50 back from there.
    assert!(
        (view(&layout) - 1070.).abs() < 0.001,
        "view: {}",
        view(&layout)
    );
}

#[test]
fn dragging_stops_at_the_first_column_and_turns_back_at_once() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    drag_and_let_go_moving(&mut layout, &[-100_000., 30.], false);
    settle(&mut layout);
    assert!(
        (view(&layout) - 30.).abs() < 0.001,
        "view: {}",
        view(&layout)
    );
}
