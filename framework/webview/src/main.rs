use gtk::{
    prelude::GtkWindowExt,
    traits::{ContainerExt, WidgetExt},
    Container, Inhibit, Widget, Window, WindowPosition, WindowType,
};

fn main() {
    let mut app = WebView::new();
    app.set_title("MyWV");
    app.set_window_type(WinType::PopUp);
    app.run();
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
    pub window: WinType,
}

impl WebView {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            window: WinType::TopLevel,
        }
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    fn set_window_type(&mut self, window_type: WinType) {
        self.window = window_type;
    }

    fn run(&mut self) {
        gtk::init().unwrap();
        let window: Window = match self.window {
            WinType::TopLevel => gtk::Window::new(gtk::WindowType::Toplevel),
            WinType::PopUp => gtk::Window::new(gtk::WindowType::Popup),
        };

        window.connect_delete_event(move |_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
        window.set_title(&self.title);
        window.show_all();
        gtk::main();
    }
}
