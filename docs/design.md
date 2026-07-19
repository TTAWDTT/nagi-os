# Nagi OS Interface Design

## Design intent

Nagi means calm. The interface avoids a dense stream of terminal text and
treats the 80 x 25 VGA screen as a stable workspace. Cyan identifies
navigation, green identifies live or successful state, blue identifies user
input, yellow identifies storage, and red is reserved for kernel ownership or
failure. Gray is supplementary because curses terminals do not reproduce the
VGA gray palette consistently; VNC remains the color-accurate reference.

## Screen model

The screen has four persistent regions:

1. Header: identity, animated PIT-driven wind mark, and compact controls.
2. Command panel: current page, current filter, shortcuts, and candidates.
3. Content region: stable title, metrics/body, and a next-command hint.
4. Footer: ticks, trace state, breadcrumb, and current mode.

Page components use fixed rows and columns. Badges introduce state, paired
metrics support comparison, table headers stabilize scanning, progress bars
show proportions, and the final row suggests the next useful action. Subsystem
dump functions receive an explicit content column so they cannot overwrite the
command panel.

## Interaction model

- Left/Right, Home/End, Backspace/Delete edit the command line.
- Inline completion appears in the prompt; Tab or Right accepts it at line end.
- Up/Down selects a completion while filtering.
- Up/Down navigates an eight-entry history from an empty prompt; the draft is
  restored after returning past the newest entry.
- Unknown commands use edit distance to offer a concrete correction.
- `present` makes `n`, `b`, Left, and Right contextual page controls.
- `watch` exits when a normal command is submitted.

## Motion

The NAGI wind mark and live dashboard pulse are driven by PIT ticks, not host
animation. Motion is therefore evidence that the guest kernel is handling
interrupts. Refresh is limited to 4 Hz for readability. Timer and keyboard
handlers execute with interrupt-gate nesting disabled, so a watch redraw
cannot interrupt a keyboard-buffer mutation halfway through.

## Constraints

VGA text mode provides 16 colors, 80 columns, and 25 rows. It cannot provide
proportional type, alpha blending, arbitrary graphics, or reliable gray in
QEMU curses. The design uses spacing, hierarchy, symbols, stable geometry, and
meaningful motion rather than pretending those constraints do not exist.
