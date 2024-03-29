use splot::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "splot=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    //Update the data every 10ms
    tokio::spawn({
        let mut plotter = plotter.clone();
        async move {
            let mut n = 0;
            loop {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();

                plotter.push_async([now, n as f64]).await;
                n += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    });

    plotter.serve("0.0.0.0:3004").await
}
