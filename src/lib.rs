pub mod cli;
mod config;
mod req;
mod utils;

pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use req::RequestProfile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub querys: Vec<(String, String)>,
    pub bodys: Vec<(String, String)>,
}
