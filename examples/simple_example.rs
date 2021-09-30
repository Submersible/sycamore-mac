use sycamore::context::{use_context, ContextProvider, ContextProviderProps};
use sycamore::prelude::*;
use sycamore_mac::render_to_mac;

#[component(App<G>)]
fn app() -> Template<G> {
    template! {
        "My App"
        ContextProvider(ContextProviderProps {
            value: Counter(Signal::new(0)),
            children: || template! {
                CounterView()
            }
        })
    }
}

#[derive(Clone)]
struct Counter(Signal<i32>);

#[component(CounterView<G>)]
fn counter_view() -> Template<G> {
    let counter = use_context::<Counter>();
    let text = Signal::new("hey".to_string());

    template! {
        view {
            View(template! {
                "Count: "
                (counter.0.get())
            })
            view {
                (text.get())
            }
        }
    }
}

#[component(View<G>)]
fn view(children: Template<G>) -> Template<G> {
    // @TODO Provide a cleaner interface for children:
    // https://github.com/sycamore-rs/sycamore/issues/107
    template! {
        view {
            (children)
        }
    }
}

fn main() {
    println!(
        "HTML: {:?}",
        sycamore::render_to_string(|| {
            let nodes = template! { App() };
            println!("=== SsrNode ===\n{:?}", nodes);
            nodes
        })
    );

    render_to_mac("com.test.window", || {
        let nodes = template! {
            view {
                view { "hello world" }
                view { "horizontal layout" }
            }
        };
        println!("=== MacNode ===\n{:?}", nodes);
        nodes
    })
    .unwrap();
}
