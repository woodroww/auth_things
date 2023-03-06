use gloo_console::log;
use yew::prelude::*;
use yew_router::prelude::*;
use gloo_utils::{window, document};
use wasm_bindgen::JsCast;
use web_sys::HtmlDocument;
use yewdux::prelude::*;
use crate::{API_BASE_URL, router::Route, store::PoseStore};
use crate::contexts::use_theme;
use stylist::{yew::styled_component, css};

#[function_component]
pub fn Login() -> Html {
    let login_google_url = format!("{}/client-login/google", API_BASE_URL);
    let login_fusion_url = format!("{}/client-login/fusion", API_BASE_URL);
    let login_github_url = format!("{}/client-login/github", API_BASE_URL);

    let theme = use_theme();
    let link_style = css!(r#"
        color: ${link_color};
        text-decoration: none;
        font-size: 16px;
    "#,
        link_color = theme.link_color.clone(),
    );

    html! {
        <>
            <h1>{"Login Page"}</h1>
            <ul>
                <li><a href={login_google_url} class={link_style.clone()}>{"Login Google"}</a></li>
                <li><a href={login_fusion_url} class={link_style.clone()}>{"Login Fusion"}</a></li>
                <li><a href={login_github_url} class={link_style}>{"Login GitHub"}</a></li>
            </ul>
        </>
    }
}

#[function_component]
pub fn LoginSuccess() -> Html {

    let document = document().unchecked_into::<HtmlDocument>();
    let navigator = use_navigator().unwrap();
    let (_store, dispatch) = use_store::<PoseStore>();

    if let Ok(cookie_string) = document.cookie() {
        for raw_cookie in cookie_string.split("; ") {
            //gloo_console::log!("cookie: {}", raw_cookie);
            if let Some((key, value)) = raw_cookie.split_once('=') {
                if key == "access_token" {
                    gloo_console::log!("found access_token");
                    navigator.push(&Route::Home);
                    dispatch.reduce_mut(|mut store| {
                        store.token = value.to_string();
                    });
                    break;
                }
            }
        }
        
    } else {
        gloo_console::log!("no cookie");
        navigator.push(&Route::Login);
        //window().location().set_href("/login").ok();
    }

    html! {
    }
}
