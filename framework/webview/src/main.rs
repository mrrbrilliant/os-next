#![allow(dead_code, unused_imports)]
use gdk::{PropMode, WindowTypeHint};
use glib::ObjectExt;
use gtk::{
    traits::GtkWindowExt,
    traits::{ContainerExt, WidgetExt},
    Container, Inhibit, Widget, Window, WindowPosition, WindowType,
};
fn main() {
    let mut app = WebView::new();
    app.set_title("MyWV");
    app.set_window_type(WindowTypeHint::Dock);
    app.set_window_size(1920, 32);

    app.run(true);
}

#[derive(Debug)]
pub enum WinType {
    TopLevel,
    PopUp,
}

impl Default for WinType {
    fn default() -> Self {
        Self::TopLevel
    }
}

pub struct WebView {
    pub title: String,
    pub window_type: WinType,
    window: Window,
}

impl WebView {
    pub fn new() -> Self {
        gtk::init().unwrap();
        Self {
            title: String::new(),
            window_type: WinType::TopLevel,
            window: Window::new(WindowType::Toplevel),
        }
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.window.set_title(&self.title);
    }

    fn set_window_position(&mut self, x: i32, y: i32) {
        self.window.move_(x, y)
    }

    fn set_window_size(&mut self, width: i32, height: i32) {
        self.window.resize(width, height)
    }

    fn set_window_struts(&mut self) {
        let screen = self.window.screen();
        let mut width = screen.clone().unwrap().width();
        let mut monitors: Vec<gdk::Rectangle> = Vec::new();

        for monitor in 0..screen.clone().unwrap().n_monitors() {
            let mg = screen.clone().unwrap().monitor_geometry(monitor);
            // width = mg.width;
            monitors.push(mg);
        }
        let primary_monitor = screen.clone().unwrap().primary_monitor();
        // let curmon = screen
        //     .clone()
        //     .unwrap()
        //     .monitor_at_window(&screen.unwrap().active_window().unwrap());

        let current_monitor_props = monitors[primary_monitor as usize];
        let gdk_window = self.window.window().unwrap();
        let struts = gdk::Atom::intern("_NET_WM_STRUT");
        let type_ = gdk::Atom::intern("CARDINAL");
        let bar_size = 32;
        self.set_window_position(current_monitor_props.x, current_monitor_props.y);

        gdk::property_change(
            &gdk_window,
            &struts,
            &type_,
            32,
            PropMode::Replace,
            gdk::ChangeData::ULongs(&[
                0,        // left
                0,        // right
                bar_size, // top
                0,        // bottom
            ]),
        );
        let struts_partial = gdk::Atom::intern("_NET_WM_STRUT_PARTIAL");
        gdk::property_change(
            &gdk_window,
            &struts_partial,
            &type_,
            32,
            PropMode::Replace,
            gdk::ChangeData::ULongs(&[
                0,                                                  // left
                0,                                                  // right
                bar_size,                                           // top
                0,                                                  // bottom
                0,                                                  // left_start_y
                0,                                                  // left_end_y
                0,                                                  // right_start_y
                0,                                                  // right_end_y
                0,                                                  // top_start_x
                0 + current_monitor_props.width.clone() as u64 - 1, // top_end_x
                0,                                                  // bottom_start_x
                0,                                                  // bottom_end_x
            ]),
        );

        // fork https://github.com/o9000/tint2/blob/master/src/taskbar/taskbar.c
    }

    fn set_window_type(&mut self, window_type: WindowTypeHint) {
        self.window.set_type_hint(window_type)
    }

    fn run(&mut self, strut: bool) {
        self.window.connect_delete_event(move |_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
        self.window.show_all();
        if strut {
            self.set_window_struts();
        }
        gtk::main();
    }
}
