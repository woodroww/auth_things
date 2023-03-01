use gloo_console::log;
use yew::prelude::*;
use yew_router::prelude::*;
use gloo_utils::{window, document};
use wasm_bindgen::JsCast;
use web_sys::HtmlDocument;
use yewdux::prelude::*;
use crate::{BACKEND_BASE_URL, router::Route, store::PoseStore};

#[function_component]
pub fn Login() -> Html {
    let login_url = format!("{}/client-login", BACKEND_BASE_URL);
    gloo_console::log!("the login url: {}", &login_url);
    html! {
        <>
            <h1>{"Login Page"}</h1>
            <a href={login_url}>{"Login"}</a>
        </>
    }
}

#[function_component]
pub fn LoginSuccess() -> Html {

    let document = document().unchecked_into::<HtmlDocument>();
    let navigator = use_navigator().unwrap();
    let (_store, dispatch) = use_store::<PoseStore>();

    // If there's a cookie, assume that we are logged in, else redirect to login page.
    if let Ok(cookie_string) = document.cookie() {
        // TODO: Validate cookie
        if cookie_string.is_empty() {
            gloo_console::log!("no cookie");
            window().location().set_href("/login").ok();
        }
        for raw_cookie in cookie_string.split("; ") {
            gloo_console::log!("cookie: {}", raw_cookie);
            if let Some((key, value)) = raw_cookie.split_once('=') {
                gloo_console::log!("found access_token");
                if key == "access_token" {
                    navigator.push(&Route::Home);
                    dispatch.reduce_mut(|mut store| {
                        store.token = value.to_string();
                    });
                }
            }
        }
        
    } else {
        gloo_console::log!("no cookie");
        window().location().set_href("/login").ok();
    }

    html! {
        <>
            <p>{"Yeah boy!"}</p>
        </>
    }
}
