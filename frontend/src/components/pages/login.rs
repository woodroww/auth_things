use yew::prelude::*;

const URL: &str = std::env!("URL");

#[function_component]
pub fn Login() -> Html {
    /*
    let login = Callback::from(|_: MouseEvent| {
        window()
            .location()
            .set_href("localhost/client-login")
            .ok();
    });*/
    let login_url = format!("{}/client-login", URL);
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
    html! {
        <>
            <p>{"Yeah boy!"}</p>
        </>
    }
}
