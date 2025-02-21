


Wrap the future (async fn main() in this example) in an outer future which does Span Shenanigans (see below):

Upon receiving Start:
* Create info_span!("Start again")
* Put span into a thread local
* Span::enter the span

Upon receiving Stop:
* Remove span from thread local
* Span::exit the span

Span Shenanigans:
* When poll is called, check thread local for span, if Some, call Span::enter
* Poll inner future (async fn main() in this example)
* After polling inner future, check thread local for span, if Some, call Span::exit.
