use yew::prelude::*;
use stylist::{css, yew::styled_component};

use crate::router::Route;
use crate::components::atoms::link::BBLink;
use crate::components::atoms::bb_button::BBButton;
use crate::contexts::{use_theme, ThemeKind, ThemeProvider, ThemeContext};

#[styled_component]
pub fn Navbar() -> Html {
    let stylesheet = css!(
        r#"
          section {
            border-bottom: 1px solid antiquewhite;
            padding: 10px 20px;
            display: flex;
            justify-content: space-between;
          }

          .nav-right {
            display: flex;
          }

          .nav-right button {
            margin-left: 10px;
          }
        "#
    );

    let theme = use_theme();
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
            <section>
                <BBLink text="Home" route={Route::Home} />
                <BBLink text="Portfolio" route={Route::Portfolio} />
                <BBButton onclick={switch_theme} label={theme_str}/>
            </section>
        </div>
    }
}
