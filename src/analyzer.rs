use std::path::PathBuf;
use tower_lsp::lsp_types::{Location, Position, Range};

use crate::parser;

pub struct Analyzer {}

impl Analyzer {
    // pub fn new() -> Self {
    //     Self {}
    // }

    pub async fn hover(file: PathBuf, line: usize, col: usize) -> Option<(String)> {
        None
    }
}
