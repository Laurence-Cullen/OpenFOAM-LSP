use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric1;
use nom::number::complete::double;
use nom::sequence::delimited;
use nom::{IResult, Parser};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

// fn get_token_at_position(line: usize, col: usize) -> Option<Token> {
//     // Implementation goes here
// }

// fn main() {
//     let path = "cavity/0/U";
//     let input = std::fs::read_to_string(path).expect("Failed to read file");
//     let (remaining, tokens) = scan(&input).unwrap();
//     println!("Remaining: {}", remaining);
//     println!("Tokens: {:?}", tokens);
// }

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Token {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // Literals
    Int(i64),
    Float(f64),

    // OpenFOAM keywords
    FoamFile,
    ConvertToMeters,
    Blocks,
    Vertices,
    Hex,
    SimpleGrading,
    Boundary,
    Application,
    StartFrom,
    StartTime,
    StopAt,
    EndTime,
    DeltaT,
    WriteControl,
    WriteInterval,
    PurgeWrite,
    WriteFormat,
    WritePrecision,
    WriteCompression,
    TimeFormat,
    TimePrecision,
    RunTimeModifiable,
    DdtSchemes,
    GradSchemes,
    DivSchemes,
    LaplacianSchemes,
    InterpolationSchemes,
    SnGradSchemes,
    Solvers,
    Dimensions,
    InternalField,
    BoundaryField,
    Type,
    Value,
    Format,
    Ascii,
    Class,
    VolVectorField,
    Object,
    U,
    Uniform,
    MovingWall,
    FixedWalls,
    FrontAndBack,
    FixedValue,
    NoSlip,
    Empty,

    BlockComment,
    LineComment,
    Eof,
}

/// Count how many characters there are per line, inluding new lines
pub fn count_characters_per_line(input: &str) -> Vec<usize> {
    input
        .lines()
        .map(|line| line.len() + 1) // +1 for the newline character
        .collect()
}

pub fn index_from_line_and_col(chars_per_line: Vec<usize>, line: usize, col: usize) -> usize {
    let mut index = 0;

    // Do cumulative sum of characters per line up to the given line
    for &num_chars in chars_per_line.iter().take(line) {
        index += num_chars;
    }

    // Add the column index to the cumulative sum
    index += col;

    index
}

pub fn col_from_index(chars_per_line: Vec<usize>, index: usize) -> usize {
    let mut col = 0;

    let mut cumulative_chars = 0;
    // Loop through
    for &num_chars in chars_per_line.iter() {
        if index >= cumulative_chars && index < cumulative_chars + num_chars {
            col = index - cumulative_chars;
            break;
        }
        cumulative_chars += num_chars;
    }

    col
}

/// Use nom to parse lines of lox code and return a vector of tokens and spans.
pub fn scan(input: &str) -> IResult<&str, (Vec<Token>, Vec<Span>)> {
    let mut tokens = Vec::new();
    let mut spans = Vec::new();
    let mut current_input = input;
    let mut current_index = 0;

    while !current_input.is_empty() {
        // Skip whitespace and track position
        let whitespace_parser = nom::character::complete::multispace0;
        let (after_ws, _) = whitespace_parser(current_input)?;
        current_index += current_input.len() - after_ws.len();
        current_input = after_ws;

        if current_input.is_empty() {
            break;
        }

        let start_index = current_index;

        // Try to parse a token
        let mut token_parser = alt((block_comment, line_comment, keyword, int, single_char_token));

        let parser_result = token_parser.parse(current_input);

        match parser_result {
            Ok((remaining, token)) => {
                let consumed = current_input.len() - remaining.len();
                let end_index = start_index + consumed;

                tokens.push(token);
                spans.push(Span {
                    start: start_index,
                    end: end_index,
                });
                current_input = remaining;
                current_index = end_index;
            }
            Err(e) => return Err(e),
        }
    }

    Ok((current_input, (tokens, spans)))
}

fn line_comment(input: &str) -> IResult<&str, Token> {
    let (remaining, comment) =
        delimited(tag("//"), nom::bytes::complete::take_until("//"), tag("//")).parse(input)?;
    Ok((remaining, Token::LineComment))
}

fn single_char_token(input: &str) -> IResult<&str, Token> {
    let (remaining, lexeme) = alt((
        tag("("),
        tag(")"),
        tag("{"),
        tag("}"),
        tag("["),
        tag("]"),
        tag(","),
        tag("."),
        tag("-"),
        tag("+"),
        tag(";"),
        tag("/"),
        tag("*"),
        tag("="),
    ))
    .parse(input)?;

    let token_type: Token = match lexeme {
        "(" => Token::LeftParen,
        ")" => Token::RightParen,
        "{" => Token::LeftBrace,
        "}" => Token::RightBrace,
        "[" => Token::LeftBracket,
        "]" => Token::RightBracket,
        "," => Token::Comma,
        "." => Token::Dot,
        "-" => Token::Minus,
        "+" => Token::Plus,
        ";" => Token::Semicolon,
        "/" => Token::Slash,
        "*" => Token::Star,
        _ => unreachable!(),
    };

    Ok((remaining, token_type))
}
fn block_comment(input: &str) -> IResult<&str, Token> {
    let (remaining, comment) =
        delimited(tag("/*"), nom::bytes::complete::take_until("*/"), tag("*/")).parse(input)?;
    Ok((remaining, Token::BlockComment))
}

fn float(input: &str) -> IResult<&str, Token> {
    let (remaining, number) = double.parse(input)?;

    Ok((remaining, Token::Float(number)))
}

fn int(input: &str) -> IResult<&str, Token> {
    let (remaining, number) = nom::character::complete::i64.parse(input)?;

    Ok((remaining, Token::Int(number)))
}

pub fn get_foam_definition(input: Token) -> String {
    let definition = match input {
        Token::FoamFile => {
            "Specifies file metadata including version, format, and class of the OpenFOAM dictionary."
        }
        Token::ConvertToMeters => {
            "Specifies the scaling factor to convert the mesh units to meters."
        }
        Token::Blocks => "Defines the list of mesh blocks in blockMesh.",
        Token::Vertices => "Lists the vertex coordinates used to construct mesh blocks.",
        Token::Hex => "Specifies a hexahedral block using a list of vertex indices.",
        Token::SimpleGrading => {
            "Describes the cell expansion ratios for mesh grading inside a block."
        }
        Token::Boundary => {
            "Defines the boundaries and patches of the mesh with their types and faces."
        }
        Token::Application => "Specifies the name of the solver or application to be executed.",
        Token::StartFrom => {
            "Indicates how to determine the starting time of the simulation (e.g., 'startTime' or 'latestTime')."
        }
        Token::StartTime => "Specifies the time value to start the simulation from.",
        Token::StopAt => {
            "Determines when the simulation should stop (e.g., 'endTime' or 'writeNow')."
        }
        Token::EndTime => "Specifies the end time value of the simulation.",
        Token::DeltaT => "Defines the time step size used for time integration.",
        Token::WriteControl => {
            "Determines the control strategy for writing output (e.g., 'timeStep', 'runTime')."
        }
        Token::WriteInterval => "Specifies the interval at which results are written to disk.",
        Token::PurgeWrite => "Limits the number of time directories stored by deleting old ones.",
        Token::WriteFormat => {
            "Specifies the format (e.g., ascii, binary) in which data is written."
        }
        Token::WritePrecision => "Sets the numerical precision of written output.",
        Token::WriteCompression => {
            "Controls whether the output files are compressed (e.g., 'on' or 'off')."
        }
        Token::TimeFormat => {
            "Specifies the format used to write time directories (e.g., 'general' or 'fixed')."
        }
        Token::TimePrecision => "Sets the precision of time values used in directory names.",
        Token::RunTimeModifiable => {
            "Determines if dictionaries can be modified during a running simulation."
        }
        Token::DdtSchemes => "Defines the schemes for time derivative discretization.",
        Token::GradSchemes => "Specifies the gradient calculation schemes.",
        Token::DivSchemes => "Defines the discretization schemes for divergence terms.",
        Token::LaplacianSchemes => "Specifies the schemes for discretizing Laplacian terms.",
        Token::InterpolationSchemes => {
            "Defines the interpolation schemes for field values at cell faces."
        }
        Token::SnGradSchemes => {
            "Specifies the schemes used for surface-normal gradient calculations."
        }
        Token::Solvers => {
            "Defines the linear solvers and their parameters for solving different fields."
        }
        Token::Dimensions => {
            "Specifies the physical dimensions of a field in SI units using a 7-tuple."
        }
        Token::InternalField => "Defines the initial value of the field inside the domain.",
        Token::BoundaryField => "Specifies boundary conditions for a field on each patch.",
        Token::Type => "Specifies the type of a dictionary entry or boundary condition.",
        Token::Value => "Used to assign a value in boundary or internal field specifications.",
        _ => "Unknown OpenFOAM keyword.",
    };
    definition.to_string()
}

/// Return a token from the input string which is a Lox keyword
fn keyword(input: &str) -> IResult<&str, Token> {
    let (remaining, lexeme) = alphanumeric1(input)?;
    let token_type = match lexeme {
        "FoamFile" => Token::FoamFile,
        "convertToMeters" => Token::ConvertToMeters,
        "blocks" => Token::Blocks,
        "vertices" => Token::Vertices,
        "hex" => Token::Hex,
        "simpleGrading" => Token::SimpleGrading,
        "boundary" => Token::Boundary,
        "application" => Token::Application,
        "startFrom" => Token::StartFrom,
        "startTime" => Token::StartTime,
        "stopAt" => Token::StopAt,
        "endTime" => Token::EndTime,
        "deltaT" => Token::DeltaT,
        "writeControl" => Token::WriteControl,
        "writeInterval" => Token::WriteInterval,
        "purgeWrite" => Token::PurgeWrite,
        "writeFormat" => Token::WriteFormat,
        "writePrecision" => Token::WritePrecision,
        "writeCompression" => Token::WriteCompression,
        "timeFormat" => Token::TimeFormat,
        "timePrecision" => Token::TimePrecision,
        "runTimeModifiable" => Token::RunTimeModifiable,
        "ddtSchemes" => Token::DdtSchemes,
        "gradSchemes" => Token::GradSchemes,
        "divSchemes" => Token::DivSchemes,
        "laplacianSchemes" => Token::LaplacianSchemes,
        "interpolationSchemes" => Token::InterpolationSchemes,
        "snGradSchemes" => Token::SnGradSchemes,
        "solvers" => Token::Solvers,
        "dimensions" => Token::Dimensions,
        "internalField" => Token::InternalField,
        "boundaryField" => Token::BoundaryField,
        "type" => Token::Type,
        "value" => Token::Value,
        "format" => Token::Format,
        "ascii" => Token::Ascii,
        "class" => Token::Class,
        "volVectorField" => Token::VolVectorField,
        "object" => Token::Object,
        "U" => Token::U,
        "uniform" => Token::Uniform,
        "movingWall" => Token::MovingWall,
        "fixedValue" => Token::FixedValue,
        "frontAndBack" => Token::FrontAndBack,
        "noSlip" => Token::NoSlip,
        "empty" => Token::Empty,
        "fixedWalls" => Token::FixedWalls,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Alpha,
            )));
        }
    };
    Ok((remaining, token_type))
}

/// Takes a vec of tokens and spans, returns a HashMap of Span -> error string
pub fn get_errors(tokens: &[Token], spans: &[Span]) -> HashMap<Span, String> {
    let mut errors = HashMap::new();

    for (i, (token, span)) in tokens.iter().zip(spans.iter()).enumerate() {
        match token {
            Token::Uniform => {
                // Check that the following tokens are: LeftBrace, Int, Int, Int, RightBrace
                if i + 4 < tokens.len() {
                    if tokens[i + 1] != Token::LeftBrace {
                        errors.insert(
                            *span,
                            format!("Expected {:?}, found {:?}", Token::LeftBrace, tokens[i + 1]),
                        );
                    }
                    if !matches!(tokens[i + 2], Token::Int(_)) {
                        errors.insert(*span, format!("Expected Int, found {:?}", tokens[i + 2]));
                    }
                    if !matches!(tokens[i + 3], Token::Int(_)) {
                        errors.insert(*span, format!("Expected Int, found {:?}", tokens[i + 3]));
                    }
                    if !matches!(tokens[i + 4], Token::Int(_)) {
                        errors.insert(*span, format!("Expected Int, found {:?}", tokens[i + 4]));
                    }
                    if tokens[i + 5] != Token::RightBrace {
                        errors.insert(
                            *span,
                            format!(
                                "Expected {:?}, found {:?}",
                                Token::RightBrace,
                                tokens[i + 5]
                            ),
                        );
                    }
                    if tokens[i + 6] != Token::Semicolon {
                        errors.insert(
                            *span,
                            format!("Expected {:?}, found {:?}", Token::Semicolon, tokens[i + 6]),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    errors
}

pub fn get_inline_hints(tokens: &[Token], spans: &[Span]) -> HashMap<Span, String> {
    let mut hints = HashMap::new();

    for (i, token) in tokens.iter().enumerate() {
        if token == &Token::Dimensions {
            //the next nine tokens should be:
            // LeeftBracket, Int, Int, Int, Int, Int, Int, Int, RightBracket
            // If they match this add the following hints for the Int token spans:
            // kg, meters, seconds, kelvin, moles, amps, candela

            if i + 8 < tokens.len() {
                let unit_labels = vec!["kg", "m", "s", "K", "mol", "A", "cd"];
                let mut all_match = true;

                // Check LeftBracket
                if tokens[i + 1] != Token::LeftBracket {
                    all_match = false;
                }

                // Check the 7 Int tokens and RightBracket
                for j in 0..7 {
                    if !matches!(tokens[i + 2 + j], Token::Int(_)) {
                        all_match = false;
                        break;
                    }
                }

                // Check RightBracket
                if tokens[i + 9] != Token::RightBracket {
                    all_match = false;
                }

                // If all tokens match the expected pattern, add hints for the Int tokens
                if all_match {
                    for j in 0..7 {
                        let token_index = i + 2 + j;
                        hints.insert(spans[token_index].clone(), unit_labels[j].to_string());
                    }
                }
            }
        };
    }
    hints
}

pub fn token_color(token: Token) -> String {
    match token {
        Token::Hex => "#FF0000".to_string(),
        Token::VolVectorField => "#00FF00".to_string(),
        Token::Object => "#0000FF".to_string(),
        Token::U => "#FFFF00".to_string(),
        Token::Uniform => "#FF00FF".to_string(),
        Token::MovingWall => "#00FFFF".to_string(),
        Token::FixedValue => "#800080".to_string(),
        Token::FrontAndBack => "#808080".to_string(),
        Token::NoSlip => "#FFA500".to_string(),
        Token::Empty => "#800000".to_string(),
        Token::FixedWalls => "#008000".to_string(),
        _ => "#FFFFFF".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foam_keywords() {
        let input = "hex (0 1 2 3 4 5 6 7) (40 40 1) simpleGrading (1 1 1)";
        let (remaining, token) = keyword(input).unwrap();
        assert_eq!(token, Token::Hex);
    }

    #[test]
    fn test_invalid_keyword() {
        let input = "invalid";
        let result = keyword(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_comment() {
        let input = "// This is a comment\n";
        let (remaining, comment) = line_comment(input).unwrap();
        assert_eq!(remaining, "");
        assert_eq!(
            comment,
            Token::LineComment(" This is a comment".to_string())
        );
    }

    #[test]
    fn test_scan_line() {
        let input = "var x <= 10;";
        let tokens = scan(input);

        println!("{:?}", tokens);
    }
}
