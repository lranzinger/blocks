# Blocks

A modern webassembly implementation of the classic Tetris game written in Rust using Macroquad, supporting both desktop and mobile browsers.

## Features

- Modern Tetris rules: SRS rotation with wall kicks, lock delay, 7-bag randomizer
- Reserve (hold) for one piece to swap in later, and a preview of the next three pieces
- Scoring with soft and hard drop points, combos and back-to-back tetris bonus
- A new level every 10 lines, getting faster up to level 15
- Line clear animation with particles, score popups and screen shake on a tetris
- Layouts for portrait phones and landscape screens, sharp at any display scaling
- Pause button, automatic pause when switching tabs
- High score stored in the browser, vibration on supported phones

## Controls

### Touch Controls

- Swipe left/right: Move piece, keep the finger down to keep moving
- Tap: Rotate piece
- Hold the finger still or drag down slowly: Drop faster
- Flick down: Hard drop
- Flick up: Put the piece into the reserve, or swap it with the reserved one
- Pause button in the top right corner

### Keyboard Controls

- Left/Right or A/D: Move piece
- Up, W or X: Rotate clockwise
- Z, Y or Ctrl: Rotate counter-clockwise
- Down/S: Drop faster
- Space: Hard drop
- C or Shift: Reserve piece
- P or Escape: Pause
- Enter: Start, resume or restart

## Build Instructions

```bash
# Debug build
./build.sh debug

# Release build
./build.sh release
```

The web files end up in `dist/`. Serve that folder with any static web server, for example
`simple-http-server dist` or `python3 -m http.server -d dist`.

```bash
# Run the unit tests of the game logic
cargo test
```

## Deployment

Every push is built and checked (format, clippy, tests) by GitHub Actions and deployed to Cloudflare Pages:

- `main` goes to production.
- Every other branch gets its own preview URL (`<branch>.<project>.pages.dev`), which is listed in the
  summary of the workflow run. Production is not affected.

## Play Online

Visit [https://play.ranzinger.dev](https://play.ranzinger.dev) to play directly in your browser.
