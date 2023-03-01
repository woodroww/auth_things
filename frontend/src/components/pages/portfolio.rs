use gloo_console::log;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::store::PoseStore;

#[function_component]
pub fn Portfolio() -> Html {
    let (store, dispatch) = use_store::<PoseStore>();
    wasm_bindgen_futures::spawn_local(async move {
        let pose_response = crate::api::poses::get_poses("jam").await.unwrap();
        dispatch.reduce_mut(|store| store.poses = pose_response.poses);
    });
    log!("poses: {}", store.poses.len());
    html! {
        <>
            <h1>{"Amazing Projects"}</h1>
        </>
    }
}
