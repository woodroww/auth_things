use stylist::{css, yew::styled_component};
use yew::prelude::*;

use crate::components::atoms::bb_button::BBButton;
use crate::components::atoms::link::BBLink;
use crate::contexts::{use_theme, ThemeKind};
use crate::router::Route;

#[styled_component]
pub fn Navbar() -> Html {
    let theme = use_theme();
    let stylesheet = css!(
        r#"
          .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            position: fixed;
            top: 0px;
            left: 0px;
            right: 0px;
            height: 50px;
            background-color: ${bg};
            color: ${ft_color};
            border-bottom-width: 1px;
            border-bottom-style: groove;
            border-color: antiquewhite;
            z-index: 100;
            padding-left: 20px;
            padding-right: 20px;
          }

          .nav-right {
            display: flex;
          }

          .nav-right button {
            margin-left: 10px;
          }
        "#,
        bg = theme.background_color.clone(),
        ft_color = theme.font_color.clone(),
    );

    let theme_str = match theme.kind() {
        ThemeKind::Light => "Dark Theme",
        ThemeKind::Dark => "Light Theme",
    };
    let other_theme = match theme.kind() {
        ThemeKind::Light => ThemeKind::Dark,
        ThemeKind::Dark => ThemeKind::Light,
    };
    let switch_theme = Callback::from(move |_| theme.set(other_theme.clone()));

    // <button class={style} onclick={switch_theme} id="yew-sample-button">{"Switch to "}{theme_str}</button>

    html! {
        <div class={stylesheet}>
            <nav class="header">
                <BBLink text="Home" route={Route::Home} />
                <BBLink text="Portfolio" route={Route::Portfolio} />
                <BBLink text="Login" route={Route::Login} />
                <BBButton onclick={switch_theme} label={theme_str} />
            </nav>
        </div>
    }
}
