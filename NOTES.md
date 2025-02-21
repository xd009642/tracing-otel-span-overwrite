
Hayden idea

Wrap the future (async fn main() in this example) in an outer future which does Span Shenanigans (see below):

Upon receiving Start:
* Create info_span!("Start again")
* Put span into an `Arc<Mutex>`
* Span::enter the span

Upon receiving Stop:
* I guess remove span from `Arc<Mutex>` 
* Span::exit the span

Span Shenanigans:
* When poll is called, check `Arc<Mutex>` for span, if Some, call Span::enter
* Poll inner future (async fn main() in this example)
* After polling inner future, check thread local for span, if Some, call Span::exit.

Oh, yeah. thread local is no good. You actually need a field on SpanShenanigans 
which you then pass to the inner future (so it's got to be Arc-Mutex-ed).
which is the same thing that Instrumented does for #[instrument], just that 
you're going to allow it to be modified while the inner future is being polled.

Me idea

hmm So I think I can instead create an opentelemetry::SpanContext and at the 
start of the loop call SpanContext::attach(), then use the extension trait 
methods to set that as the span parent for the tracing spans. And just not 
have an outer tracing span instead :ThinkingFast:

Hayden idea

Refactor your code to make it more amenable to tracing span
