use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::pages::home::Home;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/portfolio")]
    Portfolio,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
        Route::Portfolio => html! { <h1>{ "portfolio" }</h1> },
    }
}
