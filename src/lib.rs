pub mod cli;
mod config;
mod utils;

pub use config::{
    get_body_text, get_header_text, get_status_text, DiffConfig, DiffProfile, LoadConfig,
    ReqConfig, RequestProfile, ResponseProfile,
};
pub use utils::highlight_text;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub querys: Vec<(String, String)>,
    pub bodys: Vec<(String, String)>,
}
