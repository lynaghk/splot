pub mod prelude {
    pub use crate::{Config, PlotterHandle};
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

use log::*;

use std::sync::{Arc, RwLock};
use tokio::{net::ToSocketAddrs, sync::broadcast::Sender};

fn make_routes<const N: usize>(plotter: PlotterHandle<N>) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    #[derive(Clone)]
    struct AppState<const N: usize> {
        plotter: PlotterHandle<N>,
    }

    async fn stream_data<const N: usize>(
        State(state): State<AppState<N>>,
    ) -> impl axum::response::IntoResponse {
        let (v, mut rx) = {
            let snapshot = state.plotter.0.read().unwrap();
            // Subscribe to the broadcast channel while snapshot is locked so that we don't miss any updates.
            let rx = snapshot.data.tx.subscribe();
            (snapshot.data.serialize(), rx)
        };

        let s = stream! {
            // Define an error to satisfy the typechecker.
            type MyError = Result<Bytes, std::io::Error>;
            yield MyError::Ok(v.into());
            while let Ok(x) = rx.recv().await {
                yield Ok(x)
            };
        };
        Body::from_stream(s)
    }

    // TODO: eliminate this copy/paste somehow.
    async fn stream_text<const N: usize>(
        State(state): State<AppState<N>>,
    ) -> impl axum::response::IntoResponse {
        let (v, mut rx) = {
            let snapshot = state.plotter.0.read().unwrap();
            // Subscribe to the broadcast channel while snapshot is locked so that we don't miss any updates.
            let rx = snapshot.text.tx.subscribe();
            (snapshot.text.serialize(), rx)
        };

        let s = stream! {
            // Define an error to satisfy the typechecker.
            type MyError = Result<Bytes, std::io::Error>;
            yield MyError::Ok(v.into());
            while let Ok(x) = rx.recv().await {
                yield Ok(x)
            };
        };
        Body::from_stream(s)
    }

    let page = plotter.0.read().unwrap().html_page.clone();

    Router::new()
        .route("/", get(|| async { Html(page) }))
        .route("/data", get(stream_data))
        .route("/text", get(stream_text))
        .with_state(AppState {
            plotter: plotter.clone(),
        })
        .fallback_service(handle_404.into_service())
}

async fn serve<A: ToSocketAddrs>(app: Router, addr: A) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("splot listening on {}", listener.local_addr().unwrap());
    // TODO: switch this to http2, which will also require figuring out certs so browsers will actually use it.
    axum::serve(listener, app).await.unwrap();
}

struct StreamableStorage {
    tx: Sender<Bytes>,
    data: Vec<u8>,
}

impl StreamableStorage {
    fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(16);
        Self { data: vec![], tx }
    }

    fn push<T: IntoBytes>(&mut self, x: T) {
        let bs = x.into_bytes();
        self.data.extend_from_slice(&bs[..]);
        let _ = self.tx.send(bs);
    }

    fn serialize(&self) -> Vec<u8> {
        //TODO: this should probably return Bytes so we're not cloning the entire history everytime a new client connects.
        self.data.clone()
    }
}

trait IntoBytes {
    fn into_bytes(self) -> Bytes;
}

impl<const N: usize> IntoBytes for [f64; N] {
    fn into_bytes(self) -> Bytes {
        self.iter().flat_map(|y| y.to_be_bytes()).collect()
    }
}

impl IntoBytes for String {
    fn into_bytes(self) -> Bytes {
        self.into()
    }
}

struct Plotter<const N: usize> {
    data: StreamableStorage,
    text: StreamableStorage,
    html_page: String,
}

impl<const N: usize> Plotter<N> {
    fn new(config: &Config) -> Self {
        // HTML templating, lol
        let html_page = include_str!("index.html")
            .replace("UPLOT_CSS", include_str!("../vendor/uplot.css"))
            .replace("USER_CSS", &config.css)
            .replace(
                "GENERATED_JS",
                &format!(
                    "{} {}",
                    include_str!("../vendor/uplot.js"),
                    include_str!("main.js")
                        .replace("N_SERIES", &format!("{}", N))
                        .replace("PLOT_OPTS", &config.plot)
                ),
            );

        Self {
            data: StreamableStorage::new(),
            text: StreamableStorage::new(),
            html_page,
        }
    }

    fn push(&mut self, v: [f64; N]) {
        self.data.push(v);
    }

    fn push_text(&mut self, v: String) {
        self.text.push(v);
    }
}

#[derive(Clone, Default)]
pub struct Config {
    pub plot: String,
    pub css: String,
}

#[derive(Clone)]
pub struct PlotterHandle<const N: usize>(Arc<RwLock<Plotter<N>>>);
impl<const N: usize> PlotterHandle<N> {
    pub fn new(config: &Config) -> Self {
        Self(Arc::new(RwLock::new(Plotter::<N>::new(config))))
    }

    pub fn push(&mut self, v: [f64; N]) {
        self.0.write().unwrap().push(v);
    }

    pub fn push_text(&mut self, v: String) {
        self.0.write().unwrap().push_text(v);
    }

    pub async fn serve<A: ToSocketAddrs>(self, addr: A) {
        let app = make_routes(self);
        serve(app, addr).await
    }

    pub fn serve_blocking<A: ToSocketAddrs>(self, addr: A) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async { self.serve(addr).await })
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
