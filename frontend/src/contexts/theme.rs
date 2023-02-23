use std::ops::Deref;

use once_cell::sync::Lazy;
use stylist::yew::styled_component;
use yew::html::ImplicitClone;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ThemeKind {
    Dark,
    Light,
}

impl ImplicitClone for ThemeKind {}

#[derive(Debug, Clone)]
pub(crate) struct Theme {
    pub font_color: String,
    pub background_color: String,
    pub link_color: String,
    pub button_color: String,
}

impl ThemeKind {
    pub fn current(&self) -> &Theme {
        static LIGHT_THEME: Lazy<Theme> = Lazy::new(|| Theme {
            font_color: "rgb(50, 48, 47)".to_string(),
            background_color: "rgb(235, 219, 178)".to_string(),
            link_color: "rgb(50, 48, 47)".to_string(),
            button_color: "rgb(168, 153, 132)".to_string(),
        });

        static DARK_THEME: Lazy<Theme> = Lazy::new(|| Theme {
            font_color: "rgb(181, 118, 20)".to_string(),
            background_color: "rgb(40, 40, 40)".to_string(),
            link_color: "rgb(181, 118, 20)".to_string(),
            button_color: "rgb(215, 153, 33)".to_string(),
        });

        match self {
            ThemeKind::Dark => &DARK_THEME,
            ThemeKind::Light => &LIGHT_THEME,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ThemeContext {
    inner: UseStateHandle<ThemeKind>,
}

impl ThemeContext {
    pub fn new(inner: UseStateHandle<ThemeKind>) -> Self {
        Self { inner }
    }

    pub fn set(&self, kind: ThemeKind) {
        self.inner.set(kind)
    }

    pub fn kind(&self) -> ThemeKind {
        (*self.inner).clone()
    }
}

impl Deref for ThemeContext {
    type Target = Theme;

    fn deref(&self) -> &Self::Target {
        self.inner.current()
    }
}

impl PartialEq for ThemeContext {
    fn eq(&self, rhs: &Self) -> bool {
        *self.inner == *rhs.inner
    }
}

#[derive(Debug, PartialEq, Properties)]
pub(crate) struct ThemeProviderProps {
    pub children: Children,
}

#[styled_component]
pub(crate) fn ThemeProvider(props: &ThemeProviderProps) -> Html {
    let theme_kind = use_state(|| ThemeKind::Dark);

    let theme_ctx = ThemeContext::new(theme_kind);

    html! {
        <ContextProvider<ThemeContext> context={theme_ctx}>
            {props.children.clone()}
        </ContextProvider<ThemeContext>>
    }
}

#[hook]
pub(crate) fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().unwrap()
}
