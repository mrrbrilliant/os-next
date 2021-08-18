#![allow(deprecated)]
use glib::{ToValue, ToVariant};
use gtk::{
    prelude::GtkWindowExt,
    traits::{ContainerExt, WidgetExt},
    Container, Inhibit, Widget, Window, WindowPosition, WindowType,
};
// #[cfg(feature = "v2_6")]
use webkit2gtk::UserContentManager;
use webkit2gtk::{
    traits::{SettingsExt, UserContentManagerExt, WebContextExt, WebViewExt},
    CacheModel, JavascriptError, JavascriptResult, Settings, WebContext, WebView,
};

use actix_web::{body::Body, dev::Server, rt, web, App, HttpRequest, HttpResponse, HttpServer};
use mime_guess::from_path;
use pem::{EncodeConfig, LineEnding};
use rand_core::OsRng;
use rsa::{
    PaddingScheme, PrivateKeyPemEncoding, PublicKey, PublicKeyPemEncoding, RSAPrivateKey,
    RSAPublicKey,
};
use rust_embed::RustEmbed;
use std::{borrow::Cow, fs::File, io::prelude::*, sync::mpsc, thread};
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate webkit2gtk_sys as ffi;
use gdk::gdk_pixbuf::Pixbuf;
use glib::prelude::*; // or `use gtk::prelude::*;`
use glib::Value;
use std::fs::create_dir_all;

#[derive(RustEmbed)]
#[folder = "gui/build"]
struct Asset;

fn assets(req: HttpRequest) -> HttpResponse {
    let path = if req.path() == "/" {
        "index.html"
    } else {
        // trim leading '/'
        &req.path()[1..]
    };

    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
                Cow::Borrowed(bytes) => bytes.into(),
                Cow::Owned(bytes) => bytes.into(),
            };

            HttpResponse::Ok()
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not found"),
    }
}

fn run_actix(server_tx: mpsc::Sender<Server>, port_tx: mpsc::Sender<u16>) -> std::io::Result<()> {
    let mut server = rt::System::new("SEL");

    server.block_on(async move {
        let server = HttpServer::new(|| App::new().service(web::resource("*").to(assets)))
            .bind("127.3.6.9:3000")?;

        let port = server.addrs().first().unwrap().port();
        port_tx.send(port.clone()).unwrap();

        let server = server.run();
        server_tx.send(server.clone()).unwrap();
        server.await
    })
}

fn main() {
    gtk::init().unwrap();
    let (server_tx, server_rx) = mpsc::channel();
    let (port_tx, port_rx) = mpsc::channel();

    thread::spawn(move || run_actix(server_tx, port_tx).unwrap());

    let port = port_rx.recv().unwrap();
    let server = server_rx.recv().unwrap();

    let window = Window::new(WindowType::Toplevel);
    let context = WebContext::default().unwrap();
    let content = UserContentManager::new();
    let webview = WebView::with_user_content_manager(&content);

    webview
        .context()
        .unwrap()
        .set_web_extensions_initialization_user_data(&"webkit".to_variant());
    webview
        .context()
        .unwrap()
        .set_web_extensions_directory("./cache");

    // context.set_web_extensions_initialization_user_data(&"webkit".to_variant());
    // context.set_web_extensions_directory("./cache");

    // content.register_script_message_handler("external");
    webview
        .user_content_manager()
        .unwrap()
        .register_script_message_handler("sel");
    webview.load_uri(&format!("http://127.3.6.9:{}", &port));

    window.add(&webview);

    window.set_position(WindowPosition::Center);
    let settings = WebViewExt::settings(&webview).unwrap();
    settings.set_enable_developer_extras(true);
    settings.set_enable_html5_local_storage(true);
    settings.set_enable_javascript(true);

    webview
        .user_content_manager()
        .unwrap()
        .connect_script_message_received(Some("sel"), move |ucm, result| {
            let context = result.global_context().unwrap();
            let value = result.value().unwrap().to_string(&context);
            if let Some(val) = value {
                match val.as_ref() {
                    "genkey" => {
                        rsa_keygen();
                        let pubkey = read_pubkey();
                        println!("{}", &pubkey);
                        &webview.run_javascript(
                            &format!("window.localStorage.setItem('pubkey', `{}`);", &pubkey),
                            None::<&gio::Cancellable>,
                            |_result| {
                                println!("Responsed");
                            },
                        );
                        ()
                    }
                    _ => {}
                }
            }
        });

    // content.connect_script_message_received(Some("external"), move |ucm, result| {
    //     let context = result.global_context().unwrap();
    //     let value = result.value().unwrap().to_string(&context);
    // if let Some(val) = value {
    //     match val.as_ref() {
    //         "hello" => {
    //             println!("Hello");
    //             // webview.execute_editing_command(command)
    //         }
    //         _ => {}
    //     }
    // }
    // });

    let screen = gdk::Screen::default().unwrap();
    let rec = screen.monitor_geometry(0);
    let w = rec.width;
    let h = rec.height;
    let app_w = w / 4;
    let app_h = h / 4 * 3;

    window.resize(app_w, app_h);
    window.set_position(WindowPosition::Center);
    window.set_title("SEL");
    #[cfg(debug_assertions)]
    window.set_icon_from_file("gui/build/sel.svg").unwrap();
    #[cfg(not(debug_assertions))]
    window
        .set_icon_from_file("/usr/share/icons/koompi/sel.svg")
        .unwrap();
    // let icon = Pixbuf::from_stream(stream, cancellable)
    // window.set_icon();
    // window.set_icon_from_file();
    window.show_all();

    // webview.run_javascript("alert('Hello');", None::<&gio::Cancellable>, |_result| {});

    // webview.connect_script_message_received();
    // webview.connect_script_message_received();

    window.connect_delete_event(move |_, _| {
        rt::System::new("SEL").block_on(server.stop(true));
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}

pub fn rsa_keygen() {
    let mut rng = OsRng;
    let sel_dir = std::env::home_dir().unwrap().join(".sel");
    if !sel_dir.exists() {
        create_dir_all(&sel_dir).unwrap();
    }
    let bits = 1024;
    let private_key = RSAPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RSAPublicKey::from(&private_key);

    let private_key_string = private_key.to_pem_pkcs8_with_config(EncodeConfig {
        line_ending: LineEnding::CRLF,
    });
    let public_key_string = public_key.to_pem_pkcs8_with_config(EncodeConfig {
        line_ending: LineEnding::CRLF,
    });

    let mut private_key_file = File::create(sel_dir.join("user_private.pem")).unwrap();
    private_key_file
        .write_all(private_key_string.unwrap().as_bytes())
        .unwrap();
    let mut public_key_file = File::create(sel_dir.join("user_public.pem")).unwrap();
    public_key_file
        .write_all(public_key_string.unwrap().as_bytes())
        .unwrap();
}

pub fn read_pubkey() -> String {
    let sel_dir = std::env::home_dir().unwrap().join(".sel");
    let mut data = String::new();
    let mut file = File::open(sel_dir.join("user_public.pem")).unwrap();
    file.read_to_string(&mut data).unwrap();
    let der_encoded =
        data.lines()
            .filter(|line| !line.starts_with("-"))
            .fold(String::new(), |mut data, line| {
                data.push_str(&line);
                data
            });
    der_encoded
}
