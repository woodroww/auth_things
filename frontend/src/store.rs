use yewdux::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::poses::PoseInfo;

//pub type StoreType = PersistentStore<PoseStore>;
//pub type StoreDispatch = Dispatch<StoreType>;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")] // can also be "session"
pub struct PoseStore {
    pub username: String,
    pub token: String,
    pub poses: Vec<PoseInfo>,
}
