#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        unused_mut,
        unused_unsafe,
        unreachable_code,
        unused_must_use
    ),
    feature(ptr_metadata, ptr_internals, type_alias_impl_trait)
)]

use cacao::button::Button;
use cacao::color::Color;
use cacao::foundation::NSString;
use cacao::input::{TextField, TextFieldDelegate};
use cacao::layout::{Layout, LayoutConstraint};
use cacao::macos::window::Window;
use cacao::macos::{App, AppDelegate};
use cacao::text::{Font, Label, TextAlign};
use cacao::view::View;
use core_graphics::base::CGFloat;
use objc::{msg_send, sel, sel_impl};
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use sycamore;
use sycamore::generic_node::Event;
use sycamore::prelude::*;
use sycamore::reactive::ReactiveScope;

#[derive(Debug, Clone)]
pub struct MacNode(Rc<MacNodeInner>);

#[derive(Debug)]
pub struct MacNodeInner {
    // node: String,
    view: View,
    // @TODO can't seem to mutate this with `ref mut` ...
    node: Rc<RefCell<MacNodeType>>,
    parent: RefCell<Weak<Self>>,
}

#[derive(Debug)]
pub enum MacNodeType {
    Text {
        text: RefCell<String>,
        view: Label,
    },
    View {
        direction: Direction,
        children: RefCell<Vec<ConstrainedMacNode>>,
    },
    Button {
        text: RefCell<String>,
        button: Button,
    },
    Input {
        text: RefCell<String>,
        text_field: TextField<EventEmitter>,
    },
}

#[derive(Debug)]
pub struct ConstrainedMacNode {
    node: MacNode,
    constraints: Vec<LayoutConstraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Horizontal,
    Veritcal,
}

impl PartialEq for MacNode {
    fn eq(&self, other: &Self) -> bool {
        self.0.node == other.0.node
        // @TODO compare parents?
    }
}

impl PartialEq for MacNodeType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MacNodeType::Text { text: a, .. }, MacNodeType::Text { text: b, .. })
            | (MacNodeType::Button { text: a, .. }, MacNodeType::Button { text: b, .. })
            | (MacNodeType::Input { text: a, .. }, MacNodeType::Input { text: b, .. }) => a == b,
            (
                MacNodeType::View {
                    direction: a,
                    children: c1,
                    ..
                },
                MacNodeType::View {
                    direction: b,
                    children: c2,
                    ..
                },
            ) => {
                if a != b {
                    return false;
                }
                let c1 = c1.borrow(); //.iter().map(|x| Rc::new(x.node)).collect();
                let c2 = c2.borrow();
                c1.len() == c2.len() && c1.iter().zip(c2.iter()).all(|(a, b)| a.node == b.node)
            }
            _ => false,
        }
    }
}

impl Eq for MacNode {}

impl Hash for MacNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

trait LayoutConstraintPriority<T> {
    fn set_priority<F: Into<f64>>(&mut self, priority: F) -> T;
}

impl LayoutConstraintPriority<LayoutConstraint> for LayoutConstraint {
    fn set_priority<F: Into<f64>>(&mut self, priority: F) -> LayoutConstraint {
        self.priority = priority.into();

        unsafe {
            let o = self.priority as CGFloat;
            // println!("@TODO");
        }
        self.to_owned()
    }
}

pub struct EventEmitter {
    callbacks: RefCell<Vec<Rc<dyn Fn(&str) + 'static>>>,
}

impl fmt::Debug for EventEmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventEmitter").finish()
    }
}

// impl Drop for EventEmitter {
//     fn drop(&mut self) {
//         panic!("NO NO NO!!!");
//     }
// }

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            callbacks: RefCell::new(vec![]),
        }
    }
    pub fn add_callback<F: Fn(&str) + 'static>(&self, callback: F) {
        self.callbacks.borrow_mut().push(Rc::new(callback));
    }
}

impl TextFieldDelegate for EventEmitter {
    const NAME: &'static str = "TextFieldDelegate:EventEmitter";

    fn text_should_begin_editing(&self, value: &str) -> bool {
        println!("[objc:textShouldBeginEditing]: {}", value);
        true
    }

    fn text_did_change(&self, value: &str) {
        for callback in self.callbacks.borrow().iter() {
            (callback)(value);
        }
    }

    fn text_did_end_editing(&self, value: &str) {
        println!("[objc:textDidEndEditing] {}", value);
    }
}

impl GenericNode for MacNode {
    fn element(tag: &str) -> Self {
        match tag {
            "view" => {
                let mut view = View::new();
                view.is_handle = true;

                Self(Rc::new(MacNodeInner {
                    view,
                    node: Rc::new(RefCell::new(MacNodeType::View {
                        direction: Direction::Veritcal,
                        children: RefCell::new(vec![]),
                    })),
                    parent: RefCell::new(Weak::new()), // no parent
                }))
            }
            "button" => {
                let view = View::new();
                let mut button = Button::new("");
                button.set_action(|| {
                    println!("@TODO this is not firing!");
                });

                view.add_subview(&button);

                LayoutConstraint::activate(&[
                    button.top.constraint_equal_to(&view.top).offset(40.),
                    button.left.constraint_equal_to(&view.left),
                    button
                        .width
                        .constraint_greater_than_or_equal_to_constant(200.0),
                    button.width.constraint_equal_to(&view.width),
                ]);

                Self(Rc::new(MacNodeInner {
                    view,
                    node: Rc::new(RefCell::new(MacNodeType::Button {
                        text: RefCell::new("".to_string()),
                        button,
                    })),
                    parent: RefCell::new(Weak::new()), // no parent
                }))
            }
            "input" => {
                let text = "".to_owned();
                let mut view = View::new();
                let mut text_field = TextField::with(EventEmitter::new());

                text_field.set_background_color(Color::rgba(0, 255, 0, 50));
                text_field.set_text_alignment(TextAlign::Left);
                text_field.set_font(&Font::system(40.));
                view.add_subview(&text_field);
                view.is_handle = true;

                LayoutConstraint::activate(&[
                    text_field.left.constraint_equal_to(&view.left).offset(100.),
                    text_field.height.constraint_equal_to_constant(80.),
                    text_field.top.constraint_equal_to(&view.top).offset(40.),
                    text_field.width.constraint_equal_to_constant(400.0),
                    view.width.constraint_equal_to(&text_field.width),
                    view.height
                        .constraint_greater_than_or_equal_to(&text_field.height),
                ]);

                Self(Rc::new(MacNodeInner {
                    view,
                    node: Rc::new(RefCell::new(MacNodeType::Input {
                        text: RefCell::new(text),
                        text_field,
                    })),
                    parent: RefCell::new(Weak::new()), // no parent
                }))
            }
            _ => panic!("@TODO Not sure how to create mac node from tag {:?}", tag),
        }
    }

    fn text_node(text: &str) -> Self {
        let label = Label::new();
        label.set_font(&Font::system(40.));
        label.set_text(text);
        label.set_text_color(Color::rgb(255, 255, 255));
        label.set_text_alignment(TextAlign::Left);

        let mut view = View::new();
        view.add_subview(&label);
        view.is_handle = true;

        LayoutConstraint::activate(&[
            label.top.constraint_equal_to(&view.top).offset(40.),
            label.left.constraint_equal_to(&view.left).offset(0.),
            label
                .height
                .constraint_greater_than_or_equal_to_constant(40.0),
            label
                .width
                .constraint_greater_than_or_equal_to_constant(200.0),
            label.width.constraint_less_than_or_equal_to(&view.width),
        ]);

        Self(Rc::new(MacNodeInner {
            view,
            node: Rc::new(RefCell::new(MacNodeType::Text {
                text: RefCell::new(text.to_string()),
                view: label,
            })),
            parent: RefCell::new(Weak::new()),
        }))
    }

    fn marker() -> Self {
        todo!()
    }

    fn set_attribute(&self, name: &str, value: &str) {
        let ref node = *self.0.node.as_ref().borrow();
        match (name, node) {
            ("value", MacNodeType::Button { text, button, .. }) => {
                text.replace(value.to_string());
                let title = NSString::new(value);
                button.objc.with_mut(|obj| unsafe {
                    let _: () = msg_send![obj, setTitle:&*title];
                });
            }
            (
                "value",
                MacNodeType::Input {
                    text, text_field, ..
                },
            ) => {
                text.replace(value.to_string());
                text_field.set_text(value);
            }
            _ => println!(
                "@TODO Not sure how to set attribute {:?}={:?} on {:?}",
                &name, &value, node
            ),
        }
    }

    fn remove_attribute(&self, name: &str) {
        todo!()
    }

    fn set_class_name(&self, value: &str) {
        todo!()
    }

    fn set_property(&self, name: &str, value: &sycamore::rt::JsValue) {
        todo!()
    }

    fn remove_property(&self, name: &str) {
        todo!()
    }

    fn append_child(&self, child: &Self) {
        self.0.parent.replace(Rc::downgrade(&self.0));
        let ref node = *self.0.node.as_ref().borrow();
        match node {
            MacNodeType::View { children, .. } => {
                let mut children = children.borrow_mut();

                let container = &self.0.view;
                let view = &child.0.view;
                container.add_subview(view);

                LayoutConstraint::activate(&[
                    view.left.constraint_equal_to(&container.left),
                    view.height.constraint_equal_to_constant(200.),
                    view.width
                        .constraint_less_than_or_equal_to(&container.width),
                ]);

                let constraints = if let Some(last) = children.last() {
                    let before = &last.node.0.view;
                    let constriant = view
                        .top
                        .constraint_greater_than_or_equal_to(&before.bottom)
                        .set_priority(10.);
                    // println!("===== PRIORITY=!!!! {}", constriant.priority);
                    // LayoutConstraint::activate(&[]);
                    LayoutConstraint::activate(&[
                        // before
                        //     .bottom
                        //     .constraint_less_than_or_equal_to(&view.top)
                        //     .set_priority(20.),
                        view.top
                            .constraint_greater_than_or_equal_to(&before.bottom)
                            .set_priority(20.),
                    ]);
                    // vec![view.bottom.constraint_equal_to(&container.bottom)]
                    vec![]
                } else {
                    LayoutConstraint::activate(&[view.top.constraint_equal_to(&container.top)]);
                    // vec![view.bottom.constraint_equal_to(&container.bottom)]
                    vec![]
                };
                LayoutConstraint::activate(&constraints[..]);
                children.push(ConstrainedMacNode {
                    node: child.clone(),
                    constraints,
                });
            }
            _ => todo!(),
        }
    }

    fn first_child(&self) -> Option<Self> {
        todo!()
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        self.append_child(new_node)

        // new_node.0.parent.replace(Rc::downgrade(&self.0));
        // match reference_node {
        //     None => self.append_child(new_node),
        //     Some(reference) => {
        //         let mut children = self.0.children.borrow_mut();
        //         let index = children
        //             .iter()
        //             .enumerate()
        //             .find_map(|(i, child)| (child == reference).then(|| i))
        //             .expect("reference node is not a child of this node");
        //         children.insert(index, new_node.clone());
        //         unsafe {
        //             // @TODO insert in correct order
        //             self.0.ns_obj.addSubview_(new_node.0.ns_obj);
        //         }
        //     }
        // }
    }

    fn remove_child(&self, child: &Self) {
        todo!()
    }

    fn replace_child(&self, old: &Self, new: &Self) {
        todo!()
    }

    fn insert_sibling_before(&self, child: &Self) {
        todo!()
    }

    fn parent_node(&self) -> Option<Self> {
        todo!()
    }

    fn next_sibling(&self) -> Option<Self> {
        todo!()
    }

    fn remove_self(&self) {
        todo!()
    }

    fn event(&self, name: &str, handler: Box<sycamore::generic_node::EventHandler>) {
        let ref node = *self.0.node.as_ref().borrow();
        match (name, node) {
            (
                "value",
                MacNodeType::Input {
                    text,
                    text_field:
                        TextField {
                            delegate: Some(delegate),
                            ..
                        },
                    ..
                },
            ) => {
                let text = text.clone();
                delegate.add_callback(move |new_text| {
                    // let meow = meow.lock();
                    let event = Event {
                        name: "value".to_string(),
                        value: new_text.to_owned(),
                    };
                    text.replace(new_text.to_string());
                    (handler)(event);
                });
            }
            _ => println!("@TODO Not sure how bind event {:?} on {:?}", &name, &node),
        }
    }

    fn update_inner_text(&self, text: &str) {
        println!("@TODO Why am I getting empty text? {:?}", &text);
        let ref node = *self.0.node.as_ref().borrow();
        match (text, node) {
            ("", MacNodeType::View { children, .. }) => {
                let mut children = children.borrow_mut();
                for hey in children.iter() {
                    hey.node.0.view.remove_from_superview();
                }
                children.clear();
            }
            _ => self.set_attribute("value", &text),
        }
    }

    fn dangerously_set_inner_html(&self, html: &str) {
        todo!()
    }

    fn clone_node(&self) -> Self {
        todo!()
    }
}

#[derive(Default)]
struct BasicApp {
    window: Window,
    container: View,
}

impl BasicApp {
    fn insert(&mut self, view: &View) {
        self.container.add_subview(view);

        LayoutConstraint::activate(&[
            view.top.constraint_equal_to(&self.container.top),
            view.width.constraint_equal_to(&self.container.width),
            view.height.constraint_equal_to(&self.container.height),
        ]);
    }
}

impl AppDelegate for BasicApp {
    fn did_finish_launching(&self) {
        App::activate();

        // let mut button = Button::new("WEEEEEE");
        // let handler = button.set_action(|| {
        //     println!("CLICK!!");
        // });

        // self.container.add_subview(&button);

        // LayoutConstraint::activate(&[
        //     button
        //         .top
        //         .constraint_equal_to(&self.container.top)
        //         .offset(200.),
        //     button
        //         .left
        //         .constraint_equal_to(&self.container.left)
        //         .offset(0.),
        //     // text_field.height.constraint_equal_to_constant(80.0),
        //     button
        //         .width
        //         .constraint_greater_than_or_equal_to_constant(200.0),
        //     // button.width.constraint_equal_to(&view.width),
        //     // view.height
        //     //     .constraint_greater_than_or_equal_to(&label.height),
        //     // view.height.constraint_equal_to(&label.height),
        //     // label.height.constraint_less_than_or_equal_to(&view.height),
        // ]);

        self.window.set_title("Hello From Rust");
        self.window.set_minimum_content_size(300., 200.);
        self.window.set_background_color(Color::rgba(0, 0, 0, 100));
        self.window.set_content_view(&self.container);

        // example blue box
        let mut blue = View::new();
        blue.set_background_color(Color::rgb(0, 0, 255));
        self.container.add_subview(&blue);
        LayoutConstraint::activate(&[
            blue.top
                .constraint_equal_to(&self.container.top)
                .offset(100.),
            blue.left
                .constraint_equal_to(&self.container.left)
                .offset(100.),
            blue.width.constraint_less_than_or_equal_to_constant(40.),
            blue.height.constraint_less_than_or_equal_to_constant(40.),
        ]);
        blue.is_handle = true;

        self.window.show();
    }
}

pub fn render_to_mac<S: AsRef<str>>(
    bundle_id: S,
    template: impl FnOnce() -> Template<MacNode>,
) -> Result<(), String> {
    let scope = create_root(|| {
        let mut app = BasicApp::default();
        let mut views = vec![];

        for node in template().flatten() {
            app.insert(&node.0.view);
            views.push(node);
        }

        App::new(bundle_id.as_ref(), app).run();
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));

    Ok(())
}
