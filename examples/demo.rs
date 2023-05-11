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

    // See https://github.com/leeoniya/uPlot/tree/master/docs
    let plot_opts = r##"
{
  title: "My Chart",
  width: 800,
  height: 600,
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
"##;

    let plotter = PlotterHandle::<2>::new(plot_opts);

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

    plotter.serve(3004).await
}
