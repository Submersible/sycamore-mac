use sycamore::prelude::*;
use sycamore_mac::render_to_mac;

#[component(App<G>)]
fn app() -> Template<G> {
    template! {
        view {
            view { "hello world" }
            CounterView()
        }
    }
}

#[component(CounterView<G>)]
fn counter_view() -> Template<G> {
    let text = Signal::new("hello".to_owned());

    let input: Template<G> = {
        let input: G = GenericNode::element("input");
        {
            GenericNode::set_attribute(&input, "value", "hey");
        }
        GenericNode::event(
            &input,
            "value",
            Box::new(cloned!((text) => move |event| {
                let new_text = event.value;
                println!("GOT EVENT {:?}", &new_text);
                text.set(new_text);

                // todo
            })),
        );
        Template::new_node(input)
    };

    template! {
        (input)
        view {
            (cloned!((text) => format!("Text: {}", text.get())))
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
        sycamore::render_to_string(|| template! { App() })
    );

    render_to_mac("com.test.window", || template! { App() }).unwrap();
}
