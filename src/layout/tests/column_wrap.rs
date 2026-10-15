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

fn focused(layout: &Layout<TestWindow>) -> usize {
    *layout.focus().unwrap().id()
}

/// Window ids in column order.
fn order(layout: &Layout<TestWindow>) -> Vec<usize> {
    let scrolling = layout.active_workspace().unwrap().scrolling();
    scrolling
        .columns()
        .map(|col| (1..=16).find(|id| col.contains(id)).unwrap())
        .collect()
}

fn settle(layout: &mut Layout<TestWindow>) {
    check_ops_on_layout(layout, [Op::CompleteAnimations]);
    layout.verify_invariants();
}

#[test]
fn wrapping_right_brings_the_first_of_two_wide_columns_in_from_the_right() {
    // Two 900 px columns on a 1280 px output: together wider than the screen.
    let mut layout = columns_of_width(2, 900);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    assert_eq!(focused(&layout), 2);
    // B right-aligned, A's right part shows on the left.
    assert_eq!(view(&layout), 520.);

    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);

    assert_eq!(focused(&layout), 1);
    assert_eq!(order(&layout), vec![2, 1]);
    // A came in from the right and is right-aligned; B now shows on the left.
    assert_eq!(view(&layout), 520.);

    // Again: B comes in from the right.
    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);
    assert_eq!(focused(&layout), 2);
    assert_eq!(order(&layout), vec![1, 2]);
    assert_eq!(view(&layout), 520.);
}

#[test]
fn wrapping_left_brings_the_last_of_two_wide_columns_in_from_the_left() {
    let mut layout = columns_of_width(2, 900);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    assert_eq!(view(&layout), 0.);

    check_ops_on_layout(&mut layout, [Op::FocusColumnLeftOrLast]);
    settle(&mut layout);

    assert_eq!(focused(&layout), 2);
    assert_eq!(order(&layout), vec![2, 1]);
    // B came in from the left and is left-aligned; A now shows on the right.
    assert_eq!(view(&layout), 0.);
}

#[test]
fn wrapping_moves_one_column_step_like_any_other_neighbor() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    assert_eq!(view(&layout), 1120.);

    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);

    assert_eq!(focused(&layout), 1);
    assert_eq!(order(&layout), vec![2, 3, 4, 5, 6, 1]);
    // One column step to the right: column 1 is right-aligned at the far end.
    assert_eq!(view(&layout), 1120.);
}

#[test]
fn wrapping_inside_the_strip_is_a_plain_step() {
    let mut layout = columns_of_width(6, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnFirst, Op::CompleteAnimations]);
    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);
    assert_eq!(focused(&layout), 2);
    assert_eq!(order(&layout), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn wrapping_with_one_or_no_columns_does_nothing() {
    for count in [0, 1] {
        let mut layout = columns_of_width(count, 400);
        check_ops_on_layout(
            &mut layout,
            [Op::FocusColumnRightOrFirst, Op::FocusColumnLeftOrLast],
        );
        settle(&mut layout);
        assert_eq!(order(&layout).len(), count);
    }
}

#[test]
fn the_old_jump_is_gone_from_the_wrap_actions() {
    // Upstream would jump back to A at the left edge (view 0); the loop keeps B on the left.
    let mut layout = columns_of_width(2, 900);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    check_ops_on_layout(
        &mut layout,
        [Op::FocusColumnRightOrFirst, Op::CompleteAnimations],
    );
    assert_eq!(focused(&layout), 1);
    assert_ne!(view(&layout), 0.);
}

// The loop also applies when every column is already on screen.
#[test]
fn wrapping_when_two_halves_fill_the_screen_exactly_swaps_them() {
    // Two 640 px columns on 1280 px with gaps 0, like two halves of a wide screen.
    let mut layout = columns_of_width(2, 640);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    let before = view(&layout);
    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);
    assert_eq!(focused(&layout), 1);
    // A moved behind B: B now on the left, the focused A on the right.
    assert_eq!(order(&layout), vec![2, 1]);
    assert_eq!(view(&layout), before);
}

#[test]
fn wrapping_a_group_narrower_than_the_screen_still_loops() {
    // Two 400 px columns on 1280 px: A jumps behind B and the group sits toward the side
    // being scrolled to; the empty space moves to the left.
    let mut layout = columns_of_width(2, 400);
    check_ops_on_layout(&mut layout, [Op::FocusColumnLast, Op::CompleteAnimations]);
    assert_eq!(view(&layout), 0.);
    check_ops_on_layout(&mut layout, [Op::FocusColumnRightOrFirst]);
    settle(&mut layout);
    assert_eq!(focused(&layout), 1);
    assert_eq!(order(&layout), vec![2, 1]);
    assert_eq!(view(&layout), -400.);
}
