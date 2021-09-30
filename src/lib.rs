#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut, unused_unsafe),
    feature(ptr_metadata, ptr_internals, type_alias_impl_trait)
)]

use cacao::color::Color;
use cacao::foundation::id;
use cacao::layout::{Layout, LayoutConstraint};
use cacao::text::{Font, Label, TextAlign};
use cacao::view::View;

use cacao::macos::window::Window;
use cacao::macos::{App, AppDelegate};

use core_graphics::base::CGFloat;
use objc::{class, msg_send, sel, sel_impl};

use sycamore::reactive::ReactiveScope;

use std::cell::RefCell;

use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use sycamore;

use sycamore::prelude::*;

#[derive(Debug, Clone)]
pub struct MacNode(Rc<MacNodeInner>);

#[derive(Debug)]
pub struct MacNodeInner {
    // node: String,
    view: View,
    node: MacNodeType,
    // children: RefCell<Vec<MacNode>>,
    parent: RefCell<Weak<Self>>,
}

#[derive(Debug)]
pub enum MacNodeType {
    Text {
        text: String,
        view: Label,
    },
    View {
        direction: Direction,
        children: RefCell<Vec<ConstrainedMacNode>>,
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
        // && (self.0.children.borrow().as_ref() as &Vec<MacNode>)
        //     == (other.0.children.borrow().as_ref() as &Vec<MacNode>)
    }
}

impl PartialEq for MacNodeType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MacNodeType::Text { text: a, .. }, MacNodeType::Text { text: b, .. }) => a == b,
            (MacNodeType::View { direction: a, .. }, MacNodeType::View { direction: b, .. }) => {
                a == b
            }
            _ => false,
        }
        // self.0.node == other.0.node
        //     && (self.0.children.borrow().as_ref() as &Vec<MacNode>)
        //         == (other.0.children.borrow().as_ref() as &Vec<MacNode>)
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
            println!("@TODO");
        }
        self.to_owned()
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
                    node: MacNodeType::View {
                        direction: Direction::Veritcal,
                        children: RefCell::new(vec![]),
                    },
                    parent: RefCell::new(Weak::new()), // no parent
                }))
            }
            _ => panic!("Not sure how to create mac node from {} tag", tag),
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
            label.width.constraint_less_than_or_equal_to(&view.width),
            // view.height
            //     .constraint_greater_than_or_equal_to(&label.height),
            // view.height.constraint_equal_to(&label.height),
            // label.height.constraint_less_than_or_equal_to(&view.height),
        ]);

        Self(Rc::new(MacNodeInner {
            view,
            node: MacNodeType::Text {
                text: text.to_string(),
                view: label,
            },
            // children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::new()), // no parent
        }))
    }

    fn marker() -> Self {
        todo!()
    }

    fn set_attribute(&self, name: &str, value: &str) {
        todo!()
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
        // @TODO

        match &self.0.node {
            MacNodeType::View { children, .. } => {
                println!("APPEND CHILD !!!");
                let mut children = children.borrow_mut();

                let container = &self.0.view;
                let view = &child.0.view;
                container.add_subview(view);

                LayoutConstraint::activate(&[
                    // view.top.constraint_greater_than_or_equal_to(&container.top),
                    // view.bottom
                    //     .constraint_less_than_or_equal_to(&container.bottom),
                    view.left.constraint_equal_to(&container.left),
                    view.height.constraint_equal_to_constant(200.),
                    view.width
                        .constraint_less_than_or_equal_to(&container.width),
                ]);

                let constraints = if let Some(last) = children.last() {
                    // LayoutConstraint::deactivate(&last.constraints[..]);
                    println!(" ======= COOL LAST ======");

                    let before = &last.node.0.view;

                    let constriant = view
                        .top
                        .constraint_greater_than_or_equal_to(&before.bottom)
                        .set_priority(6969.);
                    println!("===== PRIORITY=!!!! {}", constriant.priority);
                    // LayoutConstraint::activate(&[]);
                    LayoutConstraint::activate(&[
                        // before
                        //     .bottom
                        //     .constraint_less_than_or_equal_to(&view.top)
                        //     .set_priority(-16969.),
                        view.top
                            .constraint_greater_than_or_equal_to(&before.bottom)
                            .set_priority(6969.),
                    ]);

                    // toodoo
                    // vec![view.bottom.constraint_equal_to(&container.bottom)]
                    vec![]
                } else {
                    LayoutConstraint::activate(&[view.top.constraint_equal_to(&container.top)]);

                    // let meow = ;

                    // LayoutConstraint::activate(&[meow]);
                    // vec![view.bottom.constraint_equal_to(&container.bottom)]
                    vec![]
                    // TODO
                };

                LayoutConstraint::activate(&constraints[..]);

                children.push(ConstrainedMacNode {
                    node: child.clone(),
                    constraints,
                })
            }
            _ => todo!(),
        }
        // self.0.children.borrow_mut().push(child.clone());
        // .append(&child.clone());
        // self.children.clear();
        // self.append_child(child)
        // todo!()
    }

    fn first_child(&self) -> Option<Self> {
        todo!()
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        todo!()
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
        todo!()
    }

    fn update_inner_text(&self, text: &str) {
        todo!()
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
        println!("LOADED BASE WINDOW");
        self.window.set_title("AutoLayout Example!!");
        self.window.set_minimum_content_size(300., 300.);
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

        for node in template().flatten() {
            app.insert(&node.0.view);
        }

        App::new(bundle_id.as_ref(), app).run();
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));

    Ok(())
}
