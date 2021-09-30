#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut),
    feature(ptr_metadata, ptr_internals, type_alias_impl_trait)
)]

use cocoa::appkit::{
    CGFloat, NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps,
    NSApplicationActivationPolicyRegular, NSBackingStoreBuffered, NSColor, NSRunningApplication,
    NSTextField, NSView, NSViewHeightSizable, NSViewWidthSizable, NSWindow, NSWindowStyleMask,
};
use cocoa::base::{id, nil, BOOL, NO};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};

use objc::{class, msg_send, sel, sel_impl};
use sycamore::reactive::ReactiveScope;

use std::cell::RefCell;

use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use sycamore;

use sycamore::prelude::*;

#[derive(Debug, Clone)]
pub struct MacNode(Rc<MacNodeInner>);

#[derive(Debug, Clone)]
pub struct MacNodeInner {
    // node: String,
    ns_obj: id,
    node: MacNodeType,
    children: RefCell<Vec<MacNode>>,
    parent: RefCell<Weak<Self>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacNodeType {
    Text { text: String },
    View { direction: Direction },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Horizontal,
    Veritcal,
}

impl PartialEq for MacNode {
    fn eq(&self, other: &Self) -> bool {
        self.0.node == other.0.node
            && (self.0.children.borrow().as_ref() as &Vec<MacNode>)
                == (other.0.children.borrow().as_ref() as &Vec<MacNode>)
    }
}

impl Eq for MacNode {}

impl Hash for MacNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}

impl GenericNode for MacNode {
    fn element(tag: &str) -> Self {
        match tag {
            "view" => {
                let ns_obj = unsafe {
                    let obj = NSView::initWithFrame_(
                        NSView::alloc(nil),
                        NSRect::new(NSPoint::new(0., 0.), NSSize::new(200., 50.)),
                    );
                    obj.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
                    obj
                };
                Self(Rc::new(MacNodeInner {
                    ns_obj: ns_obj,
                    node: MacNodeType::View {
                        direction: Direction::Veritcal,
                    },
                    children: RefCell::new(vec![]),
                    parent: RefCell::new(Weak::new()), // no parent
                }))
            }
            _ => panic!("Not sure how to create mac node from {} tag", tag),
        }
    }

    fn text_node(text: &str) -> Self {
        // @TODO
        let ns_obj = unsafe {
            let obj = NSTextField::alloc(nil);
            obj.setEditable_(0);
            obj.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
            obj.setEditable_(NO);
            // obj.setHorizontallyResizable_(NO);
            // obj.setVerticallyResizable_(NO);
            NSTextField::initWithFrame_(
                obj,
                NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 50.)),
            );
            obj.setStringValue_(ns_string(text));
            obj
        };
        Self(Rc::new(MacNodeInner {
            ns_obj: ns_obj,
            node: MacNodeType::Text {
                text: text.to_string(),
            },
            children: RefCell::new(vec![]),
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
        self.0.children.borrow_mut().push(child.clone());
        // .append(&child.clone());
        // self.children.clear();
        // self.append_child(child)
        // todo!()
    }

    fn first_child(&self) -> Option<Self> {
        todo!()
    }

    fn insert_child_before(&self, new_node: &Self, reference_node: Option<&Self>) {
        new_node.0.parent.replace(Rc::downgrade(&self.0));
        match reference_node {
            None => self.append_child(new_node),
            Some(reference) => {
                let mut children = self.0.children.borrow_mut();
                let index = children
                    .iter()
                    .enumerate()
                    .find_map(|(i, child)| (child == reference).then(|| i))
                    .expect("reference node is not a child of this node");
                children.insert(index, new_node.clone());
                unsafe {
                    // @TODO insert in correct order
                    self.0.ns_obj.addSubview_(new_node.0.ns_obj);
                }
            }
        }
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

pub fn render_to_mac(template: impl FnOnce() -> Template<MacNode>) -> Result<(), String> {
    let mut mac_nodes: Vec<MacNode> = vec![];
    let scope = create_root(|| {
        for node in template().flatten() {
            mac_nodes.push(node);
        }
    });

    thread_local! {
        static GLOBAL_SCOPES: std::cell::RefCell<Vec<ReactiveScope>> = std::cell::RefCell::new(Vec::new());
    }

    GLOBAL_SCOPES.with(|global_scopes| global_scopes.borrow_mut().push(scope));

    unsafe {
        let _visual_effect = class!(NSVisualEffectView);
        // println!("GOT CLASS {:?}", wee);
        let _pool = NSAutoreleasePool::new(nil);

        let app = NSApp();
        app.setActivationPolicy_(NSApplicationActivationPolicyRegular);

        // create Menu Bar
        // let menubar = NSMenu::new(nil).autorelease();
        // let app_menu_item = NSMenuItem::new(nil).autorelease();
        // menubar.addItem_(app_menu_item);
        // app.setMainMenu_(create_app_menu_bar());

        // create Application menu

        // create Window
        let window = NSWindow::alloc(nil)
            .initWithContentRect_styleMask_backing_defer_(
                NSRect::new(NSPoint::new(0., 0.), NSSize::new(200., 200.)),
                NSWindowStyleMask::NSTitledWindowMask,
                NSBackingStoreBuffered,
                NO,
            )
            .autorelease();
        window.setOpaque_(1);
        // window.setStyleMask_(styleMask);
        // let style_mask = window.styleMask();
        // let blah = NSWindowStyleMask::NSResizableWindowMask;

        // println!("STYLE MASK {:?}", window.styleMask().bits());
        let mut style_mask = window.styleMask();
        style_mask |= NSWindowStyleMask::NSResizableWindowMask;
        style_mask |= NSWindowStyleMask::NSClosableWindowMask;
        style_mask |= NSWindowStyleMask::NSMiniaturizableWindowMask;
        // style_mask = NSWindowStyleMask::NSTitledWindowMask.difference(style_mask);
        // style_mask |= NSWindowStyleMask::NSTitledWindowMask;
        // style_mask |= NSWindowStyleMask::NSBorderlessWindowMask;
        // style_mask = NSWindowStyleMask::NSResizableWindowMask;
        style_mask |= NSWindowStyleMask::NSUnifiedTitleAndToolbarWindowMask;
        // style_mask |= NSWindowStyleMask::NSTitledWindowMask;
        // style_mask |= NSWindowStyleMask::NSBorderlessWindowMask;
        // style_mask |= NSWindowStyleMask::NSClosableWindowMask;

        // style_mask = NSWindowStyleMask::NSTitledWindowMask.difference(style_mask);

        window.setStyleMask_(style_mask);
        // NSWindowStyleMaskResizable
        let bg_color = NSColor::colorWithCalibratedRed_green_blue_alpha_(nil, 1.0, 0.5, 0.5, 0.5);
        window.setBackgroundColor_(bg_color);
        window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
        window.center();
        window.setTitle_(ns_string("Hello from Rust!".to_string()));
        window.makeKeyAndOrderFront_(nil);

        // fn create_text_view<T: AsRef<str>>(text: T) -> id {
        //     unsafe {
        //         let obj = NSTextField::alloc(nil);
        //         obj.setEditable_(0);
        //         obj.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
        //         obj.setEditable_(NO);
        //         // obj.setHorizontallyResizable_(NO);
        //         // obj.setVerticallyResizable_(NO);
        //         NSTextField::initWithFrame_(
        //             obj,
        //             NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 50.)),
        //         );
        //         obj.setStringValue_(ns_string(text));
        //         obj
        //     }
        // }
        // let text_field = create_text_view("hello!!!");
        // let text_field2 = create_text_view("world!!!");

        // let hey = node.0.ns_obj;
        let container = NSView::initWithFrame_(
            NSView::alloc(nil),
            NSRect::new(NSPoint::new(0., 0.), NSSize::new(200., 200.)),
        );
        container.setAutoresizingMask_(NSViewWidthSizable | NSViewHeightSizable);
        for mac_node in mac_nodes {
            container.addSubview_(mac_node.0.ns_obj);
        }
        // let mut wee: id = msg_send![class!(NSScrollView), alloc];
        // NSView::initWithFrame_(
        //     wee,
        //     NSRect::new(NSPoint::new(0., 0.), NSSize::new(100., 100.)),
        // // );
        // let meowz: id = msg_send![class!(NSLayoutConstraint), constraintWithItem: text_field
        //     attribute:NSLayoutAttribute::NSLayoutAttributeBottom
        //     relatedBy:NSLayoutRelation::NSLayoutRelationEqual
        //     toItem:wee
        //     attribute:NSLayoutAttribute::NSLayoutAttributeBottom
        //     multiplier:1.0 as CGFloat
        //     constant:5.0 as CGFloat];

        // let constraints = NSArray::arrayWithObjects(nil, &vec![meowz]);
        // // meowz.autorelease();
        // let count: id = msg_send![constraints, count];
        // println!("CONSTRINTS {:?}", count);
        // let _: id = msg_send![class!(NSLayoutConstraint), activateConstraints: constraints];

        // // let is_active: id = msg_send![meowz, isActive];
        // // println!("CONRTAINT ACTIVE {:?}", meowz);
        // // NSLayoutConstraint constraintWithItem:cancelBtn attribute:NSLayoutAttributeTop relatedBy:NSLayoutRelationEqual toItem:popupView attribute:NSLayoutAttributeTop multiplier:1.0f constant:5.0f] ];

        // wee.addSubview_(text_field);
        // wee.addSubview_(text_field2);
        // let zzz: id = msg_send![wee, constraints];
        // let zzz: id = msg_send![zzz, count];
        // println!("CONSTRINTS {:?}", zzz);
        // let wee: id = msg_send![wee, addConstraint: meowz];
        // text_field.setLayerContentsPlacement(
        //     NSViewLayerContentsPlacement::NSViewLayerContentsPlacementScaleAxesIndependently,
        // );

        // text_field2.setLayerContentsPlacement(
        //     NSViewLayerContentsPlacement::NSViewLayerContentsPlacementScaleAxesIndependently,
        // );
        // let cls = objc::runtime::Class::get("NSLayoutConstraint").unwrap();
        // window
        //     ..setLayerContentsPlacement(
        //         NSViewLayerContentsPlacement::NSViewLayerContentsPlacementScaleAxesIndependently,
        //     );

        // let wee: id = msg_send![wee, addSubview: text_field];
        // window.setLayerContentsPlacement(
        //     NSViewLayerContentsPlacement::NSViewLayerContentsPlacementScaleAxesIndependently,
        // );

        window.setContentView_(container);

        let current_app = NSRunningApplication::currentApplication(nil);
        current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

        app.run();
    }

    Ok(())
}

fn ns_string<S: AsRef<str>>(text: S) -> id {
    unsafe { NSString::alloc(nil).init_str(text.as_ref()) }
}

#[repr(i64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NSLayoutAttribute {
    NSLayoutAttributeNotAnAttribute = 0,
    NSLayoutAttributeLeft = 1,
    NSLayoutAttributeRight = 2,
    NSLayoutAttributeTop = 3,
    NSLayoutAttributeBottom = 4,
    NSLayoutAttributeLeading,
    NSLayoutAttributeTrailing,
    NSLayoutAttributeWidth,
    NSLayoutAttributeHeight,
    NSLayoutAttributeCenterX,
    NSLayoutAttributeCenterY,
    NSLayoutAttributeBaseline,
    NSLayoutAttributeLastBaseline,
    NSLayoutAttributeFirstBaseline,
    NSLayoutAttributeLeftMargin,
    NSLayoutAttributeRightMargin,
    NSLayoutAttributeTopMargin,
    NSLayoutAttributeBottomMargin,
    NSLayoutAttributeLeadingMargin,
    NSLayoutAttributeTrailingMargin,
    NSLayoutAttributeCenterXWithinMargins,
    NSLayoutAttributeCenterYWithinMargins,
}

#[repr(i64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NSLayoutRelation {
    NSLayoutRelationLessThanOrEqual = -1,
    NSLayoutRelationEqual = 0,
    NSLayoutRelationGreaterThanOrEqual = 1,
}

// pub trait NSText: Sized {
//     unsafe fn setString_(self, string: id);
//     unsafe fn setHorizontallyResizable_(self, horizontallyResizable: BOOL);
//     unsafe fn setVerticallyResizable_(self, verticallyResizable: BOOL);
// }

// impl NSText for id {
//     unsafe fn setString_(self, string: id) {
//         msg_send![self, setString: string]
//     }

//     unsafe fn setHorizontallyResizable_(self, horizontallyResizable: BOOL) {
//         msg_send![self, setHorizontallyResizable: horizontallyResizable]
//     }

//     unsafe fn setVerticallyResizable_(self, verticallyResizable: BOOL) {
//         msg_send![self, setVerticallyResizable: verticallyResizable]
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
