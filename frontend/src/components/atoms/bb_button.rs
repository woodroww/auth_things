use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ButtonColor {
    Normal,
    Red,
}

impl Default for ButtonColor {
    fn default() -> Self {
        Self::Normal
    }
}

impl ToString for ButtonColor {
    fn to_string(&self) -> String {
        match self {
            ButtonColor::Normal => "normal",
            ButtonColor::Red => "red",
        }
        .to_owned()
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub label: String,
    pub onclick: Option<Callback<MouseEvent>>,
    pub color: Option<ButtonColor>,
}

#[styled_component(BBButton)]
pub fn bb_button(props: &Props) -> Html {
    let stylesheet = css!(
        r#"
          button {
            font-size: 16px;
            padding: 1px;
            border-radius: 3px;
            border: none;
          }
        "#
    );

    let color = props.color.clone().unwrap_or_default();

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
        <button {onclick} class={color.to_string()}>{&props.label}</button>
      </span>
    }
}

