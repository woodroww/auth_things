mod components;
mod router;
mod contexts;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AppData {
    pub login_url: String,
}

