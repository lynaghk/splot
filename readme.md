# Streaming plots

A quick and easy way to add low-latency time series plots and browser-accessible logs to Rust applications.

Run the [demo](examples/demo.rs) via and navigate to http://localhost:3004/

    cargo run --example demo


## Priorities

- ease of use approaching `println`
- low resource usage both ends (suitable for Raspberry Pi servers and old phone/tablet clients)


## Non-priorities

- other plot types
- API stability; add to your project's `Cargo.toml` as:

      splot = { git = "https://github.com/lynaghk/splot", rev = "< SHA of commit you've reviewed here >" }


## TODO

Roughly in descending priority order:

- example of redirecting logging
- add Raspberry Pi usage example
- don't reset zoom/interaction when new data arrives
- add esp32 usage example (likely requires alternative HTTP server implementation)
- Rust API rather than inlining uPlot config as JS
- characterize disconnect/reconnect behavior when no updates (can we rely on browser/networking stack to heartbeat, or does this need to happen at application level?)
- compare current HTTP streaming approach with websockets (PRs welcome)


## notes on alternative plotting libs

https://huww98.github.io/TimeChart/demo/
11kB minified JS webgl chart using shadow DOM, looks tidy.
leans on d3 for SVG DOM scale, etc.
can only use single-precision floats for, so data may need to be scaled before it can be rendered.
first paint is 500ms faster for uPlot than TimeChart according to TimeChart author https://huww98.github.io/TimeChart/docs/performance

https://github.com/plotters-rs/plotters-wasm-demo
all rust drawing to canvas, no
80kB wasm blob
