use super::*;

fn click_window(layout: &mut Layout<TestWindow>, id: usize) {
    layout.activate_window_from_pointer(&id);
    layout.verify_invariants();
}

fn current_position(layout: &Layout<TestWindow>) -> f64 {
    layout.active_workspace().unwrap().scrolling().view_pos()
}

#[test]
fn pointer_focus_keeps_partially_visible_windows_at_both_edges() {
    // At x=125, window 1 is clipped on the left and window 4 on the right.
    for id in [1, 4] {
        let mut layout = fixture(true, false, 6);
        pan(&mut layout, 125., false);
        assert_ne!(*layout.focus().unwrap().id(), id);

        click_window(&mut layout, id);

        assert_eq!(*layout.focus().unwrap().id(), id);
        assert_eq!(current_position(&layout), 125.);
        check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
        assert_eq!(position(&layout), 125.);
    }
}

#[test]
fn pointer_focus_does_not_center_a_visible_window() {
    let mut layout = fixture(true, true, 6);
    pan(&mut layout, 125., false);
    assert_eq!(position(&layout), -315.);

    click_window(&mut layout, 2);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);

    assert_eq!(*layout.focus().unwrap().id(), 2);
    assert_eq!(position(&layout), -315.);
}

#[test]
fn focus_commits_do_not_move_the_clicked_partial_window() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);

    click_window(&mut layout, 1);
    // TestWindow::communicate skips unchanged geometry, so model the client's
    // activation-only surface commit explicitly.
    layout.update_window(&1, None);
    layout.verify_invariants();
    check_ops_on_layout(
        &mut layout,
        [
            Op::Refresh { is_active: true },
            Op::Communicate(1),
            Op::Communicate(3),
            Op::CompleteAnimations,
        ],
    );

    assert_eq!(*layout.focus().unwrap().id(), 1);
    assert_eq!(position(&layout), 125.);
}

#[test]
fn ordinary_activation_still_reveals_the_target_window() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    check_ops_on_layout(&mut layout, [Op::FocusWindow(1), Op::CompleteAnimations]);
    assert_eq!(*layout.focus().unwrap().id(), 1);
    assert_eq!(position(&layout), 0.);
}

#[test]
fn pointer_activation_keeps_upstream_behavior_when_free_scroll_is_disabled() {
    let mut layout = fixture(false, false, 6);
    click_window(&mut layout, 4);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(*layout.focus().unwrap().id(), 4);
    assert_eq!(position(&layout), 320.);
}

#[test]
fn pointer_activation_does_not_leave_an_offscreen_target_hidden() {
    let mut layout = fixture(true, false, 6);
    pan(&mut layout, 125., false);
    click_window(&mut layout, 6);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(*layout.focus().unwrap().id(), 6);
    assert_eq!(position(&layout), 1120.);
}

#[test]
fn pointer_focus_keeps_a_partially_visible_fullscreen_window_in_place() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetFullscreenWindow {
                window: 4,
                is_fullscreen: true,
            },
            Op::Refresh { is_active: true },
            Op::Communicate(4),
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);
    assert_eq!(position(&layout), 125.);

    click_window(&mut layout, 4);
    layout.update_window(&4, None);
    layout.verify_invariants();
    check_ops_on_layout(
        &mut layout,
        [
            Op::Refresh { is_active: true },
            Op::Communicate(4),
            Op::CompleteAnimations,
        ],
    );

    assert_eq!(*layout.focus().unwrap().id(), 4);
    assert_eq!(position(&layout), 125.);
}

#[test]
fn pointer_focus_preserves_an_ongoing_manual_pan() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [Op::ViewOffsetGestureBegin {
            output_idx: 1,
            workspace_idx: None,
            is_touchpad: false,
        }],
    );
    let timestamp = layout.clock.now_unadjusted();
    check_ops_on_layout(
        &mut layout,
        [Op::ViewOffsetGestureUpdate {
            delta: 125.,
            timestamp,
            is_touchpad: false,
        }],
    );

    click_window(&mut layout, 4);
    assert_eq!(*layout.focus().unwrap().id(), 4);
    assert_eq!(current_position(&layout), 125.);
    assert!(layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .view_offset()
        .is_gesture());

    let timestamp = layout.clock.now_unadjusted();
    check_ops_on_layout(
        &mut layout,
        [Op::ViewOffsetGestureUpdate {
            delta: 10.,
            timestamp,
            is_touchpad: false,
        }],
    );
    assert_eq!(current_position(&layout), 135.);
}

#[test]
fn clicking_an_inactive_tab_in_a_partial_column_keeps_the_view() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::FocusWindow(4),
            Op::ConsumeWindowIntoColumn,
            Op::SetColumnDisplay(ColumnDisplay::Tabbed),
            Op::FocusWindow(4),
            Op::Refresh { is_active: true },
            Op::Communicate(4),
            Op::Communicate(5),
            Op::FocusColumnFirst,
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);
    assert_eq!(position(&layout), 125.);
    assert!(layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .tiles_with_render_positions()
        .any(|(tile, _, visible)| *tile.window().id() == 5 && !visible));

    click_window(&mut layout, 5);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);

    assert_eq!(*layout.focus().unwrap().id(), 5);
    assert_eq!(position(&layout), 125.);
}

#[test]
fn pointer_focus_still_raises_a_floating_window_without_moving_the_tiled_view() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetWindowFloating {
                id: Some(6),
                floating: true,
            },
            Op::FocusColumnFirst,
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);

    click_window(&mut layout, 6);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);

    assert_eq!(*layout.focus().unwrap().id(), 6);
    assert!(layout.active_workspace().unwrap().floating_is_active());
    assert_eq!(position(&layout), 125.);
}

#[test]
fn a_delayed_fullscreen_tab_commit_still_aligns_the_view() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::FocusWindow(4),
            Op::ConsumeWindowIntoColumn,
            Op::SetColumnDisplay(ColumnDisplay::Tabbed),
            Op::FocusWindow(5),
            Op::SetFullscreenWindow {
                window: 4,
                is_fullscreen: true,
            },
            Op::Refresh { is_active: true },
            // Window 5 has not yet committed its fullscreen state. Window 4
            // already makes the column fullscreen and fixes its width at 1280.
            Op::Communicate(4),
            Op::FocusColumnFirst,
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);

    click_window(&mut layout, 5);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(*layout.focus().unwrap().id(), 5);
    assert_eq!(position(&layout), 125.);

    check_ops_on_layout(&mut layout, [Op::Communicate(5), Op::CompleteAnimations]);
    assert_eq!(position(&layout), 1200.);
}

#[test]
fn a_fullscreen_client_resize_still_aligns_even_with_unchanged_tile_bounds() {
    let mut layout = fixture(true, false, 6);
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetFullscreenWindow {
                window: 4,
                is_fullscreen: true,
            },
            Op::Refresh { is_active: true },
            Op::Communicate(4),
            Op::CompleteAnimations,
        ],
    );
    pan(&mut layout, 125., false);
    click_window(&mut layout, 4);
    check_ops_on_layout(&mut layout, [Op::CompleteAnimations]);
    assert_eq!(position(&layout), 125.);

    check_ops_on_layout(
        &mut layout,
        [
            Op::SetForcedSize {
                id: 4,
                size: Some(Size::from((800, 720))),
            },
            Op::Communicate(4),
            Op::CompleteAnimations,
        ],
    );
    assert_eq!(position(&layout), 1200.);
}
