use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode, header};
use axum::response::{Html, IntoResponse};
use axum::{Router, routing::get};
use axum_htmx::HxRequest;
use axum_prometheus::PrometheusMetricLayer;
use include_dir::{Dir, include_dir};
use log::{info, warn};
use metrics_process::Collector;
use mime_guess::from_path;
use minijinja::{Environment, context};
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::Mutex;
use tower::Service;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/f");
use clap::Parser;
use tower_http::classify::ServerErrorsFailureClass;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(long, env = "EXAMPLE_CONFIG")]
    example_config: String,
}

#[derive(Clone)]
struct AppState {
    env: Environment<'static>,
    config: Config,
}
#[derive(Clone)]
struct StaticFileService;
impl Service<Request<Body>> for StaticFileService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path().trim_start_matches('/');

        let response = if let Some(file) = ASSETS_DIR.get_file(path) {
            let mime_type = from_path(path).first_or_octet_stream();
            warn!("{:?}", mime_type);
            Response::builder()
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .body(Body::from(file.contents().to_vec()))
                .unwrap()
        } else {
            StatusCode::NOT_FOUND.into_response()
        };

        std::future::ready(Ok(response))
    }
}

static READY: AtomicBool = AtomicBool::new(false);

async fn live() -> impl IntoResponse {
    StatusCode::OK
}

async fn ready() -> impl IntoResponse {
    if READY.load(Ordering::Relaxed) {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn favicon() -> impl IntoResponse {
    let favicon_path = "favicon.ico";
    if let Some(file) = ASSETS_DIR.get_file(favicon_path) {
        let mime_type = from_path(favicon_path).first_or_octet_stream();
        Response::builder()
            .header(header::CONTENT_TYPE, mime_type.as_ref())
            .body(Body::from(file.contents().to_vec()))
            .unwrap()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Booting");

    let collector = Collector::default();
    collector.describe();
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let config = Config::parse();

    let mut jinja = Environment::new();
    minijinja_embed::load_templates!(&mut jinja);
    jinja.set_loader(minijinja::path_loader("templates"));
    let state = Arc::new(Mutex::new(AppState { env: jinja, config }));

    let mut app = Router::new()
        .route("/", get(root))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/favicon.ico", get(favicon))
        // .route("/data", get(data))
        .with_state(state.clone())
        .route(
            "/metrics",
            get(|| async move {
                collector.collect();
                metric_handle.render()
            }),
        );


    #[cfg(not(debug_assertions))]
    {
        app = app.nest_service("/assets", StaticFileService);
    }

    #[cfg(debug_assertions)]
    {
        app = app.nest_service("/assets", ServeDir::new("assets/f/"));
    }
    let app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::default().include_headers(true))
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _span: &tracing::Span| {
                    tracing::error!("something went wrong {error}")
                },
            ),
    ).layer(prometheus_layer);;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    READY.store(true, Ordering::Relaxed);
    Ok(())
}

// remote

// handlers
async fn root(
    State(state): State<Arc<Mutex<AppState>>>,
    HxRequest(boosted): HxRequest,
) -> Result<Html<String>, StatusCode> {
    let s = state.lock().await;
    let template_str = if boosted { "_index.html" } else { "index.html" };
    let template = s.env.get_template(template_str).unwrap();
    let rendered = template
        .render(context! {
            title => "Home",
        })
        .unwrap();

    Ok(Html(rendered))
}
