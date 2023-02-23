use yew::prelude::*;
use crate::contexts::use_theme;
use stylist::{yew::styled_component, css};

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    pub onclick: Option<Callback<MouseEvent>>,
}

#[styled_component]
pub fn BBButton(props: &Props) -> Html {
    let theme = use_theme();
    let stylesheet = css!(
        r#"
          button {
            font-size: 16px;
            padding: 1px;
            border-radius: 3px;
            border: none;
            background-color: ${button_color};
          }
        "#,
        button_color = theme.button_color.clone(),
    );

    let onclick = {
        let props_onclick = props.onclick.clone();
        Callback::from(move |event: MouseEvent| {
            if let Some(props_onclick) = props_onclick.clone() {
                props_onclick.emit(event);
            }
        })
    };

    html! {
      <span class={stylesheet}>
        <button {onclick} class={"button"}>{&props.label}</button>
      </span>
    }
}

