use super::*;

/// Output 1 is 1280 logical pixels wide; six 400-wide columns, gaps 0, so the view can sit
/// between 0 (first column at the left) and 1120 (last column at the right). Upstream's release
/// aligns to 0, 320, 400, 720, 800 or 1120.
const EDGES: [f64; 6] = [0., 320., 400., 720., 800., 1120.];

/// Upstream's deceleration (0.997 per millisecond) as seconds of carried release speed.
fn upstream_amount() -> f64 {
    -1. / (1000. * 0.997f64.ln())
}

fn amount_options(free: bool, amount: f64, animation: bool) -> Options {
    let config = Config::parse_mem(&format!(
        r#"
        layout {{
            gaps 0
            default-column-width {{ fixed 400; }}
            center-focused-column "never"
        }}
        animations {{
            horizontal-view-movement {{ {} }}
        }}
        "#,
        if animation { "" } else { "off;" }
    ))
    .unwrap();
    let mut options = Options::from_config(&config);
    options.gestures.free_view_scroll = free;
    options.gestures.free_view_scroll_momentum = amount;
    options
}

fn six_columns(options: Options, last: bool) -> Layout<TestWindow> {
    let mut ops = vec![Op::AddOutput(1)];
    for id in 1..=6 {
        let mut params = TestWindowParams::new(id);
        params.bbox = Rectangle::from_size(Size::from((400, 200)));
        ops.push(Op::AddWindow { params });
        ops.push(Op::Refresh { is_active: true });
        ops.push(Op::Communicate(id));
    }
    ops.push(if last {
        Op::FocusColumnLast
    } else {
        Op::FocusColumnFirst
    });
    ops.push(Op::CompleteAnimations);
    check_ops_with_options(options, ops)
}

fn begin(layout: &mut Layout<TestWindow>) {
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureBegin {
            output_idx: 1,
            workspace_idx: None,
            is_touchpad: false,
        }],
    );
}

/// Drags by each delta 16 ms apart, without letting go.
fn drag(layout: &mut Layout<TestWindow>, deltas: &[f64]) {
    for &delta in deltas {
        check_ops_on_layout(layout, [Op::AdvanceAnimations { msec_delta: 16 }]);
        let timestamp = layout.clock.now_unadjusted();
        check_ops_on_layout(
            layout,
            [Op::ViewOffsetGestureUpdate {
                delta,
                timestamp,
                is_touchpad: false,
            }],
        );
    }
}

fn let_go(layout: &mut Layout<TestWindow>) {
    check_ops_on_layout(
        layout,
        [Op::ViewOffsetGestureEnd {
            is_touchpad: Some(false),
        }],
    );
}

/// Drags by each delta 16 ms apart and lets go while still moving.
fn fling(layout: &mut Layout<TestWindow>, deltas: &[f64]) {
    begin(layout);
    drag(layout, deltas);
    let_go(layout);
}

fn shown_view(layout: &Layout<TestWindow>) -> f64 {
    layout.active_workspace().unwrap().scrolling().view_pos()
}

fn next_frame(layout: &mut Layout<TestWindow>) -> f64 {
    check_ops_on_layout(layout, [Op::AdvanceAnimations { msec_delta: 16 }]);
    shown_view(layout)
}

fn animating(layout: &Layout<TestWindow>) -> bool {
    layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .view_offset()
        .is_animation_ongoing()
}

fn settled_view(layout: &mut Layout<TestWindow>) -> f64 {
    check_ops_on_layout(layout, [Op::CompleteAnimations]);
    layout.verify_invariants();
    layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .target_view_pos()
}

/// Three 10-pixel steps 16 ms apart: let go at 30, moving at 30 / 0.032 = 937.5 px/s.
const SLOW: [f64; 3] = [10., 10., 10.];
const SLOW_SPEED: f64 = 937.5;

#[test]
fn amount_zero_stays_where_let_go() {
    for animation in [true, false] {
        let mut layout = six_columns(amount_options(true, 0., animation), false);
        fling(&mut layout, &SLOW);
        assert!(!animating(&layout));
        let view = settled_view(&mut layout);
        assert!((view - 30.).abs() < 0.001, "view: {view}");
    }
}

#[test]
fn the_view_slides_on_by_the_amount_of_release_speed() {
    for amount in [0.1, 0.2, upstream_amount()] {
        let mut layout = six_columns(amount_options(true, amount, false), false);
        fling(&mut layout, &SLOW);
        let view = settled_view(&mut layout);
        let expected = 30. + SLOW_SPEED * amount;
        assert!(
            (view - expected).abs() < 0.001,
            "amount {amount}: {view} vs {expected}"
        );
        assert!(
            EDGES.iter().all(|edge| (view - edge).abs() > 1.),
            "view: {view}"
        );
    }
}

#[test]
fn the_slide_is_visible_with_the_view_animation_off() {
    let mut layout = six_columns(amount_options(true, 0.2, false), false);
    fling(&mut layout, &SLOW);
    assert!(
        animating(&layout),
        "the slide should run with the animation off"
    );
    let end = 30. + SLOW_SPEED * 0.2;
    let mut previous = shown_view(&layout);
    assert!(
        (previous - 30.).abs() < 0.001,
        "the slide starts where the drag let go"
    );
    let mut steps = Vec::new();
    for _ in 0..8 {
        let view = next_frame(&mut layout);
        assert!(
            view > previous && view < end,
            "frame {view} after {previous}, end {end}"
        );
        steps.push(view - previous);
        previous = view;
    }
    assert!(
        steps.windows(2).all(|w| w[1] < w[0]),
        "it should slow down: {steps:?}"
    );
    let view = settled_view(&mut layout);
    assert!((view - end).abs() < 0.001, "view: {view}");
}

#[test]
fn the_view_animation_setting_does_not_change_the_slide() {
    let mut on = six_columns(amount_options(true, 0.2, true), false);
    let mut off = six_columns(amount_options(true, 0.2, false), false);
    fling(&mut on, &SLOW);
    fling(&mut off, &SLOW);
    for _ in 0..5 {
        let (a, b) = (next_frame(&mut on), next_frame(&mut off));
        assert!((a - b).abs() < 0.001, "{a} vs {b}");
    }
    let (a, b) = (settled_view(&mut on), settled_view(&mut off));
    assert!((a - b).abs() < 0.001, "{a} vs {b}");
}

#[test]
fn the_slide_stops_at_the_last_column() {
    let mut layout = six_columns(amount_options(true, 1., false), false);
    fling(&mut layout, &[200., 200., 200.]);
    let mut previous = shown_view(&layout);
    let mut frames = 0;
    while animating(&layout) && frames < 200 {
        let view = next_frame(&mut layout);
        assert!(
            view >= previous && view <= 1120. + 1e-9,
            "frame {view} after {previous}"
        );
        previous = view;
        frames += 1;
    }
    assert!(frames > 1, "it should slide over several frames, not jump");
    let view = settled_view(&mut layout);
    assert!((view - 1120.).abs() < 0.001, "view: {view}");
}

#[test]
fn pushing_past_the_end_and_easing_back_slides_back() {
    // Start at the last column, push further right (the view stays), then move 20 left and let
    // go. The release speed is the view's, not the pointer's, so the slide goes left.
    let mut layout = six_columns(amount_options(true, upstream_amount(), false), true);
    assert!((settled_view(&mut layout) - 1120.).abs() < 0.001);
    begin(&mut layout);
    drag(&mut layout, &[300., 300., -20.]);
    let_go(&mut layout);
    let view = settled_view(&mut layout);
    let expected = 1100. - 20. / 0.032 * upstream_amount();
    assert!(
        (view - expected).abs() < 0.001,
        "view: {view} vs {expected}"
    );
}

#[test]
fn pushing_into_the_end_and_letting_go_does_not_move() {
    let mut layout = six_columns(amount_options(true, upstream_amount(), false), true);
    fling(&mut layout, &[300., 300., 300.]);
    assert!(!animating(&layout));
    let view = settled_view(&mut layout);
    assert!((view - 1120.).abs() < 0.001, "view: {view}");
}

#[test]
fn the_amount_alone_keeps_upstream_snapping() {
    for deltas in [&SLOW[..], &[150., 150., 150.][..], &[100_000., -50.][..]] {
        let mut plain = six_columns(amount_options(false, 0., true), false);
        fling(&mut plain, deltas);
        let plain_end = settled_view(&mut plain);

        let mut with_amount = six_columns(amount_options(false, 0.5, true), false);
        fling(&mut with_amount, deltas);
        let amount_end = settled_view(&mut with_amount);

        assert!(
            (plain_end - amount_end).abs() < 0.001,
            "{plain_end} vs {amount_end}"
        );
        assert!(
            EDGES.iter().any(|edge| (amount_end - edge).abs() < 0.001),
            "{amount_end}"
        );
    }
}
