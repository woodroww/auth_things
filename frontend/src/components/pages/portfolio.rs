use gloo_console::log;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{store::PoseStore, api::poses::PoseInfo};

#[function_component]
pub fn Portfolio() -> Html {
    let (store, dispatch) = use_store::<PoseStore>();
    let store_clone = store.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let token = store_clone.token.clone();
        match crate::api::poses::get_poses(&token).await {
            Ok(pose_response) => {
                dispatch.reduce_mut(|store| store.poses = pose_response.poses);
            }
            Err(err) => {
                log!("Portfolio() get_poses failed {}", err.to_string());
            },
        }
    });
    log!("poses: {}", store.poses.len());

    /*
    let tasks: Vec<TaskInfo> = use_store::<StoreType>()
        .state()
        .map(|store| store.tasks.clone())
        .unwrap_or_default();
    */
    html! {
        <>
            <h1>{"Amazing Projects"}</h1>
            {pose_data(&store.poses)}
        </>
    }
}

fn pose_data(poses: &Vec<PoseInfo>) -> Html {
    html! {
        poses.into_iter().map(|pose| {
            html!{
                <p>{ format!("id: {} name: {}", pose.id, pose.name) }</p>
            }
        }).collect::<Html>()
    }
}


