use super::*;

/// Output 1 is 1280 × 720 logical pixels; gaps are 16 (the default).
fn maximized_window() -> Layout<TestWindow> {
    check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        Op::Communicate(1),
        Op::MaximizeWindowToEdges { id: Some(1) },
        Op::Communicate(1),
        Op::CompleteAnimations,
    ])
}

fn window(layout: &Layout<TestWindow>, id: usize) -> &TestWindow {
    layout
        .windows()
        .map(|(_, win)| win)
        .find(|win| win.0.id == id)
        .unwrap()
}

#[test]
fn resizing_a_maximized_window_keeps_its_size_and_unmaximizes_it() {
    let mut layout = maximized_window();
    let win = window(&layout, 1);
    assert_eq!(win.pending_sizing_mode(), SizingMode::Maximized);
    assert_eq!(win.0.bbox.get().size, Size::from((1280, 720)));

    let started = layout.interactive_resize_begin(1, ResizeEdge::RIGHT);
    layout.verify_invariants();

    let win = window(&layout, 1);
    assert_eq!(win.pending_sizing_mode(), SizingMode::Normal);
    // The width stays what the maximized window had, not the width it had before.
    assert_eq!(win.requested_size().unwrap().w, 1280);
    assert!(started);
}

#[test]
fn resizing_a_maximized_window_continues_from_its_maximized_width() {
    let mut layout = maximized_window();

    check_ops_on_layout(
        &mut layout,
        [
            Op::InteractiveResizeBegin {
                window: 1,
                edges: ResizeEdge::RIGHT,
            },
            Op::InteractiveResizeUpdate {
                window: 1,
                dx: -300.,
                dy: 0.,
            },
            Op::Communicate(1),
            Op::InteractiveResizeEnd { window: 1 },
            Op::CompleteAnimations,
        ],
    );

    let win = window(&layout, 1);
    assert_eq!(win.pending_sizing_mode(), SizingMode::Normal);
    assert_eq!(win.0.bbox.get().size.w, 980);
}

#[test]
fn resizing_a_fullscreen_window_still_does_not_start() {
    let mut layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        Op::Communicate(1),
        Op::FullscreenWindow(1),
        Op::Communicate(1),
    ]);

    let started = layout.interactive_resize_begin(1, ResizeEdge::RIGHT);
    layout.verify_invariants();

    let win = window(&layout, 1);
    assert_eq!(win.pending_sizing_mode(), SizingMode::Fullscreen);
    assert!(!started);
}

#[test]
fn resizing_a_window_maximized_from_floating_keeps_it_in_the_columns() {
    let mut layout = check_ops([
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        Op::ToggleWindowFloating { id: Some(1) },
        Op::Communicate(1),
        Op::MaximizeWindowToEdges { id: Some(1) },
        Op::Communicate(1),
        Op::CompleteAnimations,
    ]);

    let started = layout.interactive_resize_begin(1, ResizeEdge::RIGHT);
    layout.verify_invariants();

    assert!(started);
    let win = window(&layout, 1);
    assert_eq!(win.pending_sizing_mode(), SizingMode::Normal);
    assert_eq!(win.requested_size().unwrap().w, 1280);
    assert!(layout
        .active_workspace()
        .unwrap()
        .scrolling()
        .tiles()
        .any(|tile| tile.window().0.id == 1));
}
