use stylist::{css, yew::{styled_component, Global}};
use yew::prelude::*;
use yew_router::prelude::*;

use components::organisms::navbar::Navbar;
use router::{switch, Route};

use contexts::{use_theme, ThemeKind, ThemeProvider};

mod components;
mod contexts;
mod router;

#[styled_component]
fn App() -> Html {

    let theme = use_theme();

    let global_style = css!(
        r#"
			html, body {
                background-color: ${bg};
                color: ${ft_color};
			}
        "#,
        bg = theme.background_color.clone(),
        ft_color = theme.font_color.clone(),
    );

    html! {
        <>
            <Global css={global_style} />
            <BrowserRouter>
                <div>
                    <Navbar />
                </div>
                <div>
                    <Switch<Route> render={switch} />
                </div>
            </BrowserRouter>
    </>
    }
}

#[styled_component]
pub fn Root() -> Html {
    html! {
        <ThemeProvider>
            <App />
        </ThemeProvider>
    }
}

fn main() {
    //yew::Renderer::<App>::new().render();

    //console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::Renderer::<Root>::new().render();
}
