use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::pages::home::Home;
use crate::components::pages::portfolio::Portfolio;
use crate::components::pages::login::LoginSuccess;
use crate::components::pages::login::Login;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/portfolio")]
    Portfolio,
    #[at("/login-success")]
    LoginSuccess,
    #[at("/login")]
    Login,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
        Route::Portfolio => html! { <Portfolio /> },
        Route::Login => html! { <Login /> },
        Route::LoginSuccess => html! { <LoginSuccess /> },
    }
}
