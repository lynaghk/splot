use splot::prelude::*;

pub fn main() {
    env_logger::init();

    let config = Config {
        // See https://github.com/leeoniya/uPlot/tree/master/docs
        plot: r##"
{
  title: "My Chart",
  width: document.body.clientWidth,
  height: Math.min(document.body.clientHeight - 100, 600),
  series: [
    {time: false},
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

    let plotter = PlotterHandle::<2>::new(&config);

    // Update the data every 10ms
    std::thread::spawn({
        let mut plotter = plotter.clone();
        move || {
            let mut n = 0;
            loop {
                plotter.push([n as f64, (2 * n) as f64]);
                n += 1;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    });

    plotter.serve_blocking("0.0.0.0:3004")
}
