use actix_web::{body::Body, dev::Server, rt, web, App, HttpRequest, HttpResponse, HttpServer};
use gio::Cancellable;
use gtk::{
    prelude::GtkWindowExt,
    traits::{ContainerExt, WidgetExt},
    Inhibit, Window, WindowType,
};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fs::File, io::prelude::*, sync::mpsc, thread};
use webkit2gtk::traits::{SettingsExt, UserContentManagerExt, WebViewExt};
use webkit2gtk::{self, LoadEvent, UserContentManager, WebContext, WebView, WebViewExtManual};

#[derive(RustEmbed)]
#[folder = "./dist"]
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
            .bind("127.3.6.9:3001")?;

        let port = server.addrs().first().unwrap().port();
        port_tx.send(port.clone()).unwrap();

        let server = server.run();
        server_tx.send(server.clone()).unwrap();
        server.await
    })
}

#[derive(Deserialize)]
struct Request {
    id: usize,
    data: serde_json::Value,
}

#[derive(Serialize)]
struct Response {
    id: usize,
    data: serde_json::Value,
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
    let ucm = UserContentManager::new();
    let webview = WebView::new_with_context_and_user_content_manager(&context, &ucm);
    webview.load_uri(&format!("http://127.3.6.9:{}", &port));
    window.add(&webview);

    let settings = WebViewExt::settings(&webview).unwrap();
    settings.set_enable_developer_extras(true);

    // On page load inject javascript to handle request-response communication
    webview.connect_load_changed(|wv, event| {
        if event != LoadEvent::Finished {
            return;
        }

        let cancellable: Option<&Cancellable> = None;
        wv.run_javascript(
            "
            window._requests = {};
            window._request_id = 0;
            window.myPostMessage = function (data) {
                return new Promise((resolve, reject) => {
                    window._request_id += 1;
                    window._requests[window._request_id] = [resolve, reject];
                    let request = JSON.stringify({id: window._request_id, data: data});
                    // console.log('Request', request);
                    window.webkit.messageHandlers.external.postMessage(request);
                });
            };
            window._myPostMessageResponse = function(response) {
                // console.log('Response', response);
                let [resolve, reject] = window._requests[response.id];
                resolve(response.data);
                delete window._requests[response.id];
            };
            ",
            cancellable,
            |result| {
                println!("inject internal js code result {:?}", result);
            },
        );
    });

    // window.set_decorated(false);
    window.show_all();

    webview
        .user_content_manager()
        .unwrap()
        .register_script_message_handler("external");

    webview
        .user_content_manager()
        .unwrap()
        .connect_script_message_received(Some("external"), move |_ucm, jsr| {
            let ctx = jsr.global_context().unwrap();
            let val = jsr.value().unwrap();
            let request = val.to_string(&ctx).unwrap();
            println!("Request {}", request);

            let request: Request = serde_json::from_str(&request).unwrap();

            // Some app logic
            // TODO

            // genereate response
            let response = serde_json::to_string(&Response {
                id: request.id,
                data: serde_json::Value::String(format!("pong")),
            })
            .unwrap();

            println!("Response {}", response);

            let cancellable: Option<&Cancellable> = None;
            webview.run_javascript(
                &format!("window._myPostMessageResponse({});", response),
                cancellable,
                |result| {
                    println!("myPostMessageResponse result {:?}", result);
                },
            );
        });

    window.connect_delete_event(move |_, _| {
        rt::System::new("DESKTOP").block_on(server.stop(false));
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
