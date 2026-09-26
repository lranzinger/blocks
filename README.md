# Blocks

A modern webassembly implementation of the classic Tetris game written in Rust using Macroquad, supporting both desktop and mobile browsers.

## Features

- Classic Tetris gameplay mechanics
- Responsive design that adapts to window size
- Touch controls for mobile devices
- Keyboard controls for desktop
- Progressive level system
- High score tracking with browser storage
- Visual effects for line clears
- 7-bag randomizer, every piece appears once per seven pieces
- Debug mode with FPS counter

## Controls

### Touch Controls

- Swipe left/right: Move piece, it follows the finger one cell at a time
- Swipe and hold: Keep moving in that direction
- Tap once: Rotate piece
- Hold: Drop piece

### Keyboard Controls

- Left/Right or A/D: Move piece
- Up/W: Rotate piece
- Down/S: Drop piece faster
- Space: Hard drop

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
