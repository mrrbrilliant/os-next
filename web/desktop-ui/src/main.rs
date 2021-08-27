#[allow(unsafe_op_in_unsafe_fn, unsafe_code)]
use futures::executor::block_on;
use js_sys::JsString;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

fn main() {
    block_on(async_main());
}

async fn async_main() {
    sycamore::render(|| unsafe {
        template! {
            p { "Hello world!" }
            button (on:click=|_| {
                spawn_local(
                    async move {
                        let data = r#"
                        {
                            "id": 1,
                            "data": {
                                ping: "pong"
                            }
                        }
                        "#;
                        let val = get_from_js(&data).await.unwrap();
                        alert(val.as_string().unwrap().as_ref())
                    }
                );
        }) {
                "Alert"
            }
        }
    });
}

async unsafe fn get_from_js(data: &str) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::resolve(myPostMessage(data).as_ref());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}

#[wasm_bindgen]
extern "C" {
    fn myPostMessage(data: &str) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
