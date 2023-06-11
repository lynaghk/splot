pub mod prelude {
    pub use crate::{Config, PlotterHandle};
}
mod buffer;

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

use std::sync::Arc;
use tokio::net::ToSocketAddrs;
use tokio::sync::RwLock;

fn make_routes<const N: usize>(plotter: PlotterHandle<N>) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    #[derive(Clone)]
    struct AppState<const N: usize> {
        plotter: PlotterHandle<N>,
    }

    // Define an error to satisfy the typechecker.
    type MyError = Result<Bytes, std::io::Error>;

    async fn stream_data<const N: usize>(
        State(state): State<AppState<N>>,
    ) -> impl axum::response::IntoResponse {
        let s = stream! {
                let mut idx = state.plotter.0.data.read().await.bottom();
                loop {
                    let r = { state.plotter.0.data.read().await.get(idx) };
                    match r {
                        buffer::GetResult::Ok(x) => {
                            yield MyError::Ok(x.into_bytes());
                            idx += 1;
                        }
                        buffer::GetResult::Expired => {
                            debug!("Slow consumer, closing connection");
                            break;
                        }
                        buffer::GetResult::WaitUntil(notify) => {
                            notify.notified().await;
                            continue;
                        }
                    }
                }
        };

        Body::from_stream(s)
    }

    // TODO: eliminate this copy/paste somehow.
    async fn stream_text<const N: usize>(
        State(state): State<AppState<N>>,
    ) -> impl axum::response::IntoResponse {
        let s = stream! {
                let mut idx = state.plotter.0.text.read().await.bottom();
                loop {
                    let r = { state.plotter.0.text.read().await.get(idx) };
                    match r {
                        buffer::GetResult::Ok(x) => {
                            yield MyError::Ok(IntoBytes::into_bytes(x));
                            idx += 1;
                        }
                        buffer::GetResult::Expired => {
                            debug!("Slow consumer, closing connection");
                            break;
                        }
                        buffer::GetResult::WaitUntil(notify) => {
                            notify.notified().await;
                            continue;
                        }
                    }
                }
        };

        Body::from_stream(s)
    }

    let page = plotter.0.html_page.clone();

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
    data: RwLock<buffer::R<[f64; N]>>,
    text: RwLock<buffer::R<String>>,
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
            data: RwLock::new(buffer::R::new([0.; N], config.n_data)),
            text: RwLock::new(buffer::R::new("".to_string(), config.n_text)),
            html_page,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub plot: String,
    pub css: String,
    pub n_data: usize,
    pub n_text: usize,
}

#[derive(Clone)]
pub struct PlotterHandle<const N: usize>(Arc<Plotter<N>>);
impl<const N: usize> PlotterHandle<N> {
    pub fn new(config: &Config) -> Self {
        Self(Arc::new(Plotter::<N>::new(config)))
    }

    pub fn push(&mut self, v: [f64; N]) {
        self.0.data.blocking_write().push(v);
    }

    pub fn push_text(&mut self, v: String) {
        self.0.text.blocking_write().push(v);
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
