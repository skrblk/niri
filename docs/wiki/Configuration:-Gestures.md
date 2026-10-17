### Overview

<sup>Since: 25.02</sup>

The `gestures` config section contains gesture settings.
For an overview of all niri gestures, see the [Gestures](./Gestures.md) wiki page.

Here's a quick glance at the available settings along with their default values.

```kdl
gestures {
    dnd-edge-view-scroll {
        trigger-width 30
        delay-ms 100
        max-speed 1500
    }

    dnd-edge-workspace-switch {
        trigger-height 50
        delay-ms 100
        max-speed 1500
    }

    hot-corners {
        // off
        top-left
        // top-right
        // bottom-left
        // bottom-right
    }
}
```

### `dnd-edge-view-scroll`

Scroll the tiling view when moving the mouse cursor against a monitor edge during drag-and-drop (DnD).
Also works on a touchscreen.

This will work for regular drag-and-drop (e.g. dragging a file from a file manager), and for window interactive move when targeting the tiling layout.

The options are:

- `trigger-width`: size of the area near the monitor edge that will trigger the scrolling, in logical pixels.
- `delay-ms`: delay in milliseconds before the scrolling starts.
Avoids unwanted scrolling when dragging things across monitors.
- `max-speed`: maximum scrolling speed in logical pixels per second.
The scrolling speed increases linearly as you move your mouse cursor from `trigger-width` to the very edge of the monitor.

```kdl
gestures {
    // Increase the trigger area and maximum speed.
    dnd-edge-view-scroll {
        trigger-width 100
        max-speed 3000
    }
}
```

### `dnd-edge-workspace-switch`

<sup>Since: 25.05</sup>

Scroll the workspaces up/down when moving the mouse cursor against a monitor edge during drag-and-drop (DnD) while in the overview.
Also works on a touchscreen.

The options are:

- `trigger-height`: size of the area near the monitor edge that will trigger the scrolling, in logical pixels.
- `delay-ms`: delay in milliseconds before the scrolling starts.
Avoids unwanted scrolling when dragging things across monitors.
- `max-speed`: maximum scrolling speed; 1500 corresponds to one screen height per second.
The scrolling speed increases linearly as you move your mouse cursor from `trigger-width` to the very edge of the monitor.

```kdl
gestures {
    // Increase the trigger area and maximum speed.
    dnd-edge-workspace-switch {
        trigger-height 100
        max-speed 3000
    }
}
```

### `hot-corners`

<sup>Since: 25.05</sup>

Put your mouse at the very top-left corner of a monitor to toggle the overview.
Also works during drag-and-dropping something.

`off` disables the hot corners.

```kdl
// Disable the hot corners.
gestures {
    hot-corners {
        off
    }
}
```

<sup>Since: 25.11</sup> You can choose specific hot corners by name: `top-left`, `top-right`, `bottom-left`, `bottom-right`.
If no corners are explicitly set, the top-left corner will be active by default.

```kdl
// Enable the top-right and bottom-right hot corners.
gestures {
    hot-corners {
        top-right
        bottom-right
    }
}
```

You can also customize hot corners per-output [in the output config](./Configuration:-Outputs.md#hot-corners).

### `free-view-scroll`

This is a downstream option in the skrblk/niri build, disabled by default.

Normally, releasing a tiling-view scrolling gesture aligns the view to a column
boundary. With `free-view-scroll`, the view instead stops exactly where the
gesture ends, or slides on by the amount set with `free-view-scroll-momentum`,
and the drag itself stops at the first and last column. This allows windows to
remain partially visible.

```kdl
gestures {
    free-view-scroll
}
```

This applies to mouse panning, touchpad swipes, and scrolling gestures in the
overview. Fullscreen columns still align to the screen. Clicking a window that
is at least partially visible focuses it without moving the view. Other focus
changes, such as from the keyboard, IPC or focus-follows-mouse, continue to use
the upstream view movement behavior; this option does not lock the view
independently of keyboard focus.

Config updates and window commits preserve the position while the active column
remains at least partially visible. If a display or window resize would hide it
completely, the usual visibility adjustment brings it back into view.

To restore snapping for subsequent gestures, omit the option or set
`free-view-scroll false`.

### `free-view-scroll-momentum`

This is a downstream option in the skrblk/niri build. It only has an effect
together with `free-view-scroll`.

It sets how far the view slides on after a scrolling gesture is let go, in
seconds of the release speed. The view keeps moving in the gesture's direction,
slowing down, until it has covered the release speed times this number, or until
it reaches the first or last column. It never aligns to a column. The default, 0,
stops the view exactly where the gesture is let go; about 0.33 slides as far as
upstream's momentum carries a fling. The value can be from 0 to 10.

```kdl
gestures {
    free-view-scroll
    free-view-scroll-momentum 0.2
}
```

The slide continues the gesture; it is not a view movement animation, so it also
runs with the `horizontal-view-movement` animation off. The global `animations`
settings still apply: with `off` the view goes straight to the end point, and
`slowdown` slows the slide down. Starting a new gesture during the slide catches
the view where it is.
