use splot::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "splot=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    //Update the data every 10ms
    tokio::spawn({
        let mut plotter = plotter.clone();
        async move {
            let mut n = 0;
            loop {
                plotter.push([n as f64, (2 * n) as f64]);
                n += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    });

    plotter.serve("0.0.0.0:3004").await
}
