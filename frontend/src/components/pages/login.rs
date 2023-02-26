use yew::prelude::*;
use gloo_utils::{window, document};
use wasm_bindgen::JsCast;
use web_sys::HtmlDocument;

const APP_ENVIRONMENT: &str = std::env!("APP_ENVIRONMENT");

#[function_component]
pub fn Login() -> Html {
    /*
    let login = Callback::from(|_: MouseEvent| {
        window()
            .location()
            .set_href("localhost/client-login")
            .ok();
    });*/

    let login_url = match APP_ENVIRONMENT {
        "production" => "https://baeuerlin.net/client-login",
        "imac" => "http://matts-imac.local:3000/client-login",
        "aquiles" => "http://aquiles.local:3000/client-login",
        _ => {
            gloo_console::log!("APP_ENVIRONMENT invalid");
            ""
        }
    };
    gloo_console::log!("the login url: {}", login_url);
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

    // If there's a cookie, assume that we are logged in, else redirect to login page.
    if let Ok(cookie_string) = document.cookie() {
        // TODO: Validate cookie
        if cookie_string.is_empty() {
            gloo_console::log!("no cookie");
            window().location().set_href("/login").ok();
        }
        for raw_cookie in cookie_string.split("; ") {
            gloo_console::log!("cookie: {}", raw_cookie);
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
