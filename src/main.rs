use opentelemetry::trace::TracerProvider as TracerProviderTrait;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{trace as sdktrace, trace::Tracer, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use std::env;
use std::thread;
use std::time::Duration;
use tracing::{error, info, info_span, instrument, span, Span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;

enum Event {
    Wait(Duration),
    Start,
    Stop,
}

#[instrument]
fn wait(d: Duration) {
    info!("Waiting");
    thread::sleep(d);
}

#[tokio::main]
async fn main() {
    let (hnd, provider) = setup_logging();

    let events = [
        Event::Start,
        Event::Wait(Duration::from_millis(10)),
        Event::Stop,
        Event::Wait(Duration::from_millis(100)),
        Event::Wait(Duration::from_millis(30)),
        Event::Stop,
        Event::Wait(Duration::from_millis(20)),
    ];

    let mut current = info_span!("Processing");
    for event in events {
        match event {
            Event::Start => current = info_span!("Start again"),
            Event::Stop => {
                current = Span::none();
            }
            Event::Wait(d) => wait(d),
        }
    }

    current = Span::none();

    provider.shutdown();
}

pub fn setup_logging() -> (Tracer, SdkTracerProvider) {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic();
    let otlp_exporter = match env::var("TRACER_ENDPOINT") {
        Ok(s) => {
            info!("Setting otel endpoint to {}", s);
            otlp_exporter.with_endpoint(s)
        }
        _ => otlp_exporter,
    };
    let otlp_exporter = otlp_exporter.build().unwrap();

    let service_name = "streaming-client";
    let trace_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .with_config(
            sdktrace::Config::default()
                .with_resource(Resource::builder().with_service_name(service_name).build()),
        )
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = trace_provider.tracer(service_name.to_string());

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());

    let filter = tracing_subscriber::EnvFilter::new("otel_experiment=info");

    let subscriber = filter
        .and_then(tracing_subscriber::fmt::Layer::default())
        .and_then(opentelemetry)
        .with_subscriber(Registry::default());

    tracing::subscriber::set_global_default(subscriber).unwrap();
    global::set_tracer_provider(trace_provider.clone());
    (tracer, trace_provider)
}
