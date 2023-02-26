use std::rc::Rc;

use stylist::{css, yew::{styled_component, Global}};
use yew::prelude::*;
use yew_router::prelude::*;
use components::organisms::navbar::Navbar;
use router::{switch, Route};
use contexts::{use_theme, ThemeKind, ThemeProvider};
use frontend::AppData;

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
                margin-left: 20px;
                padding-top: 20px;
                padding-bottom: 1000px;
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
    let login_url = format!("https://{}:{}/client-login", "matts-imac.local", "3000");
    let app_data = use_memo(|_| {
        AppData { login_url } 
    }, ());

    html! {
        <ContextProvider<Rc<AppData>> context={app_data}>
            <ThemeProvider>
                <App />
            </ThemeProvider>
        </ContextProvider<Rc<AppData>>>
    }
}

fn main() {
    //console_log::init_with_level(Level::Trace).expect("Failed to initialise Log!");
    yew::Renderer::<Root>::new().render();
}



