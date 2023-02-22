use crate::router::Route;
use stylist::{yew::styled_component, StyleSource, css};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct LinkProps {
    pub text: String,
    pub route: Route,
}

#[styled_component(BBLink)]
pub fn bb_link(props: &LinkProps) -> Html {
    let link_style = css!(r#"
        color: antiquewhite;
        text-decoration: none;
        font-size: 16px;
    "#);
    html! {
        <Link<Route> to={props.route.clone()} classes={classes!(link_style)}>
            {props.text.clone()}</Link<Route>>
    }
}
