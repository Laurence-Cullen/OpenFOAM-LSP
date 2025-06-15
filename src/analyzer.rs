use std::path::PathBuf;
use tower_lsp::lsp_types::{Location, Position, Range};

use crate::parser;

pub struct Analyzer {}

impl Analyzer {
    // pub fn new() -> Self {
    //     Self {}
    // }

    pub async fn hover(file: PathBuf, line: usize, col: usize) -> Option<(String, Location)> {
        let file_content = std::fs::read_to_string(&file).ok()?;
        let Ok((_, (tokens, spans))) = parser::scan(&file_content) else {
            return None;
        };

        let chars_per_line = parser::count_characters_per_line(&file_content);
        let index = parser::index_from_line_and_col(chars_per_line.clone(), line, col);

        let mut span_index = 0;
        let mut width = 0;
        let mut start_col = 0;

        // iterate through spans until index sits between start and end
        for (i, span) in spans.iter().enumerate() {
            if span.start <= index && index < span.end {
                span_index = i;
                width = span.end - span.start;
                start_col = parser::col_from_index(chars_per_line.clone(), span.start);
                break;
            }
        }

        let hover_text = parser::get_foam_definition(tokens[span_index]);
        let range = Range {
            start: Position {
                line: line as u32,
                character: start_col as u32,
            },
            end: Position {
                line: line as u32,
                character: start_col as u32 + width as u32,
            },
        };
        let location = Location::new(tower_lsp::lsp_types::Url::from_file_path(file).ok()?, range);
        Some((hover_text, location))
    }
}
