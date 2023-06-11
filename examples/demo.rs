use splot::prelude::*;

pub fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let config = Config {
        n_data: 10_000, // circular buffer of 10k data
        n_text: 1_000,  // and 1k text lines

        // See https://github.com/leeoniya/uPlot/tree/master/docs
        plot: r##"
{
  title: "My Chart",
  width: document.body.clientWidth,
  height: Math.min(document.body.clientHeight - 100, 600),
  series: [
    {},
    {
      spanGaps: false,

      // in-legend display
      label: "Foo",
      //value: (self, rawValue) => "$" + rawValue.toFixed(2),

      // series style
      stroke: "red",
      width: 1,
      fill: "rgba(255, 0, 0, 0.3)",
      dash: [10, 5],
    }
  ],
  axes: [{},
         {size: 70}]
}
"##
        .into(),

        css: r##"
.uplot {
  font-family: monospace;
  margin: auto;
}
"##
        .into(),
    };

    let plotter = PlotterHandle::new(&config);

    // Update the data every 10ms
    std::thread::spawn({
        let mut plotter = plotter.clone();
        move || {
            let mut n = 0;
            loop {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();

                plotter.push([now, n as f64]);
                n += 1;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    });

    // Update the text every 100ms
    std::thread::spawn({
        let mut plotter = plotter.clone();
        move || {
            let mut n = 0;
            loop {
                plotter.push_text(format!("Log line number: {n}\n"));
                n += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    });

    plotter.serve_blocking("0.0.0.0:3004")
}
