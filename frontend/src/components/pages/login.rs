use yew::prelude::*;
use gloo_utils::{window, document};
use wasm_bindgen::JsCast;
use web_sys::HtmlDocument;

//const URL: &str = std::env!("APP_URL");

#[function_component]
pub fn Login() -> Html {
    /*
    let login = Callback::from(|_: MouseEvent| {
        window()
            .location()
            .set_href("localhost/client-login")
            .ok();
    });*/
    let login_url = "https://baeuerlin.net/client-login";
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
    if let Ok(e) = document.cookie() {
        // TODO: Validate cookie
        if e.is_empty() {
            gloo_console::log!("no cookie");
            window().location().set_href("/login").ok();
        }
        gloo_console::log!("cookie");
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
