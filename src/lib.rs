pub mod prelude {
    pub use crate::PlotterHandle;
}

use async_stream::stream;
use axum::{
    body::{Body, Bytes},
    extract::State,
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};

//use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast::Sender;

fn make_routes<const N: usize>(plotter: PlotterHandle<N>) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    #[derive(Clone)]
    struct AppState<const N: usize> {
        plotter: PlotterHandle<N>,
    }

    async fn stream_body<const N: usize>(
        State(state): State<AppState<N>>,
    ) -> impl axum::response::IntoResponse {
        // Need an error type so Body::from_stream works
        type MyError = Result<Bytes, std::io::Error>;
        let (v, mut rx) = {
            let snapshot = state.plotter.0.read().unwrap();

            // need to subscribe to the broadcast channel while snapshot is locked so that we don't miss any updates.
            let rx = snapshot.tx.subscribe();
            (snapshot.serialize(), rx)
        };

        let s = stream! {
            yield MyError::Ok(v.into());
            while let Ok(x) = rx.recv().await {
                yield Ok(x)
            };
        };
        Body::from_stream(s)
    }

    let page = plotter.0.read().unwrap().page.clone();

    Router::new()
        .route("/", get(|| async { Html(page) }))
        .route("/data", get(stream_body))
        .with_state(AppState {
            plotter: plotter.clone(),
        })
        .fallback_service(handle_404.into_service())
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    // TODO: switch this to http2, which will also require figuring out certs so browsers will actually use it.
    axum::serve(listener, app).await.unwrap();
}

struct Plotter<const N: usize> {
    tx: Sender<Bytes>,
    data: Vec<[f64; N]>,
    page: String,
}

impl<const N: usize> Plotter<N> {
    fn new(plot_opts: &str) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(16);

        // HTML templating, lol
        let page = include_str!("index.html")
            .replace("UPLOT_CSS", include_str!("../vendor/uplot.css"))
            .replace(
                "GENERATED_JS",
                &format!(
                    "{} {}",
                    include_str!("../vendor/uplot.js"),
                    include_str!("main.js")
                        .replace("N_SERIES", &format!("{}", N))
                        .replace("PLOT_OPTS", plot_opts)
                ),
            );

        Self {
            tx,
            data: Vec::new(),
            page,
        }
    }

    fn push(&mut self, v: [f64; N]) {
        self.data.push(v);
        let _ = self
            .tx
            .send(v.iter().flat_map(|y| y.to_be_bytes()).collect());
    }

    fn serialize(&self) -> Vec<u8> {
        self.data
            .iter()
            .flat_map(|x| x.iter().flat_map(|y| y.to_be_bytes()))
            .collect()
    }
}

#[derive(Clone)]
pub struct PlotterHandle<const N: usize>(Arc<RwLock<Plotter<N>>>);
impl<const N: usize> PlotterHandle<N> {
    pub fn new(plot_opts: &str) -> Self {
        Self(Arc::new(RwLock::new(Plotter::<N>::new(plot_opts))))
    }

    pub fn push(&mut self, v: [f64; N]) {
        self.0.write().unwrap().push(v);
    }

    pub async fn serve(self, port: u16) {
        let app = make_routes(self);
        serve(app, port).await
    }
}

// TODO: Rust API.

// let plot = PlotBuilder::new()
//     .port(1000)
//     .with_series([
//         Series {
//             label: "Time",
//             bounds: (0., 1000.),
//             ..Default::default()
//         },
//         Series {
//             label: "Temp",
//             bounds: (0., 120.),
//             color: "red",
//             inline: json!({foo: "bar"}),
//             ..Default::default()
//         },
//         Series {
//             label: "Pressure",
//             bounds: (0., 120.),
//             color: "blue",
//             inline: json!({foo: "bar"}),
//             ..Default::default()
//         },
//     ])
//     .with_charts([
//         Chart { series: vec!["Temp"] },
//         Chart {
//             series: vec!["Pressure"],
//         },
//     ])
//     .start()?;
// //.into_axum_service -> (plot, service)