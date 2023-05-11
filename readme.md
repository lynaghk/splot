# Streaming plots

Work-in-progress.

Low-latency time series plot --- streams binary data from server over persistent HTTP connection.

Priorities:

- quick/easy to use (like `println`)
- low resource usage both ends (raspberry pi servers; old android tablet clients)

The [demo](examples/demo.rs) can be run via:

    cargo run --example demo


## TODO

- Rust API rather than inlining JS config
- "easy mode" (hide async and threads complexity --- possible to do in one or two lines only?)
- charts should size to browser width
- don't reset zoom/interaction when new data arrives
- ? configurable circular buffers so plots can stream indefinitely without memory leaks
- auto-reconnect when HTTP connection closes (would need to reload entire page, in case plot config has changed => simple live-reloading)
- characterize disconnect/reconnect behavior when no updates (can we rely on browser/networking stack to heartbeat, or does this need to happen at application level?)
- compare current HTTP streaming approach with websockets (PRs welcome =P)
- optimize HTTP streaming updates further --- can probably do some kind of high-water-mark per connection and use shared Vec, rather than passing new data across channels (unnecessary copies and memory usage)


## notes on alternative plotting libs

https://huww98.github.io/TimeChart/demo/
11kB minified JS webgl chart using shadow DOM, looks tidy. 
leans on d3 for SVG DOM scale, etc.
can only use single-precision floats for, so data may need to be scaled before it can be rendered.
first paint is 500ms faster for uPlot than TimeChart according to TimeChart author https://huww98.github.io/TimeChart/docs/performance

https://github.com/plotters-rs/plotters-wasm-demo
all rust drawing to canvas, no 
80kB wasm blob
