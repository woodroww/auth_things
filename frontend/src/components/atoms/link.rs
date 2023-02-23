use crate::router::Route;
use crate::contexts::use_theme;
use stylist::{yew::styled_component, css};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct LinkProps {
    pub text: String,
    pub route: Route,
}

#[styled_component(BBLink)]
pub fn bb_link(props: &LinkProps) -> Html {

    let theme = use_theme();
    let link_style = css!(r#"
        color: ${link_color};
        text-decoration: none;
        font-size: 18px;
    "#,
        link_color = theme.link_color.clone(),
    );
    html! {
        <Link<Route> to={props.route.clone()} classes={classes!(link_style)}>
            {props.text.clone()}</Link<Route>>
    }
}
