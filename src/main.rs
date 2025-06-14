mod parser_utils;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{
    alpha1, alphanumeric0, alphanumeric1, line_ending, not_line_ending,
};
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::delimited;
use nom::{IResult, Parser};
    
fn main() {
    let path = "cavity/0/U";
    let input = std::fs::read_to_string(path).expect("Failed to read file");
    let (remaining, tokens) = scan_line(
        &input,
    ).unwrap();
    println!("Remaining: {}", remaining);
    println!("Tokens: {:?}", tokens);
}

type Line = Vec<Token>;

#[derive(Debug, PartialEq)]
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
    BoundaryName(String),
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

    BlockComment(String),
    LineComment(String),
    Eof,
}

pub fn scan_lines(input: &str) -> Result<Vec<Line>, nom::Err<nom::error::Error<&str>>> {
    let mut lines: Vec<Line> = Vec::new();
    for line in input.lines() {
        let result = scan_line(line);

        // If result is not OK return error
        if result.is_err() {
            return Err(result.err().unwrap());
        }

        let (remaining, tokens) = result?;

        if !remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::NonEmpty,
            )));
        }
        lines.push(tokens);
    }
    Ok(lines)
}

/// Use nom to parse lines of lox code and return a vector of tokens.
pub fn scan_line(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt(ws_separated!((
        block_comment,
        line_comment,
        keyword,
        int,
        keyword,
        single_char_token
    ))))
    .parse(input)
}

fn line_comment(input: &str) -> IResult<&str, Token> {
    let (remaining, comment) =
        delimited(tag("//"), not_line_ending, many0(line_ending)).parse(input)?;
    Ok((remaining, Token::LineComment(comment.to_string())))
}

fn single_char_token(input: &str) -> IResult<&str, Token> {
    let (remaining, lexeme) = alt((
        tag("("),
        tag(")"),
        tag("{"),
        tag("}"),
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
    let (remaining, comment) = delimited(tag("/*"), is_not("*/"), tag("*/")).parse(input)?;
    Ok((remaining, Token::BlockComment(comment.to_string())))
}

fn float(input: &str) -> IResult<&str, Token> {
    let (remaining, number) = double.parse(input)?;

    Ok((remaining, Token::Float(number)))
}

fn int(input: &str) -> IResult<&str, Token> {
    let (remaining, number) = nom::character::complete::i64.parse(input)?;

    Ok((remaining, Token::Int(number)))
}

fn get_foam_definition(input: Token) -> String {
    let definition = match input {
        Token::FoamFile => "Specifies file metadata including version, format, and class of the OpenFOAM dictionary.",
        Token::ConvertToMeters => "Specifies the scaling factor to convert the mesh units to meters.",
        Token::Blocks => "Defines the list of mesh blocks in blockMesh.",
        Token::Vertices => "Lists the vertex coordinates used to construct mesh blocks.",
        Token::Hex => "Specifies a hexahedral block using a list of vertex indices.",
        Token::SimpleGrading => "Describes the cell expansion ratios for mesh grading inside a block.",
        Token::Boundary => "Defines the boundaries and patches of the mesh with their types and faces.",
        Token::Application => "Specifies the name of the solver or application to be executed.",
        Token::StartFrom => "Indicates how to determine the starting time of the simulation (e.g., 'startTime' or 'latestTime').",
        Token::StartTime => "Specifies the time value to start the simulation from.",
        Token::StopAt => "Determines when the simulation should stop (e.g., 'endTime' or 'writeNow').",
        Token::EndTime => "Specifies the end time value of the simulation.",
        Token::DeltaT => "Defines the time step size used for time integration.",
        Token::WriteControl => "Determines the control strategy for writing output (e.g., 'timeStep', 'runTime').",
        Token::WriteInterval => "Specifies the interval at which results are written to disk.",
        Token::PurgeWrite => "Limits the number of time directories stored by deleting old ones.",
        Token::WriteFormat => "Specifies the format (e.g., ascii, binary) in which data is written.",
        Token::WritePrecision => "Sets the numerical precision of written output.",
        Token::WriteCompression => "Controls whether the output files are compressed (e.g., 'on' or 'off').",
        Token::TimeFormat => "Specifies the format used to write time directories (e.g., 'general' or 'fixed').",
        Token::TimePrecision => "Sets the precision of time values used in directory names.",
        Token::RunTimeModifiable => "Determines if dictionaries can be modified during a running simulation.",
        Token::DdtSchemes => "Defines the schemes for time derivative discretization.",
        Token::GradSchemes => "Specifies the gradient calculation schemes.",
        Token::DivSchemes => "Defines the discretization schemes for divergence terms.",
        Token::LaplacianSchemes => "Specifies the schemes for discretizing Laplacian terms.",
        Token::InterpolationSchemes => "Defines the interpolation schemes for field values at cell faces.",
        Token::SnGradSchemes => "Specifies the schemes used for surface-normal gradient calculations.",
        Token::Solvers => "Defines the linear solvers and their parameters for solving different fields.",
        Token::Dimensions => "Specifies the physical dimensions of a field in SI units using a 7-tuple.",
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
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Alpha,
            )));
        }
    };
    Ok((remaining, token_type))
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_keyword() {
    //     let input = "and";
    //     let result = keyword(input);
    //     assert!(result.is_ok());
    //     let (remaining, token) = result.unwrap();
    //     assert_eq!(remaining, "");
    //     assert_eq!(token, Token::And);
    // }

    #[test]
    fn test_foam_keywords(){
        let input = "hex (0 1 2 3 4 5 6 7) (40 40 1) simpleGrading (1 1 1)";
        let (remaining, token) = keyword(input).unwrap();
        assert_eq!(token, Token::Hex);
    }


    #[test]
    fn test_foam_line(){
        let input = "hex simpleGrading";
        let (remaining, tokens) = scan_line(input).unwrap();
        let expected_tokens = vec![Token::Hex, Token::SimpleGrading];
        assert_eq!(tokens, expected_tokens);
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
        let tokens = scan_line(input);

        println!("{:?}", tokens);
    }

    // #[test]
    // fn test_scan_line_2() {
    //     let input = "var and2 = 10;";
    //     let (remaining, tokens) = scan_line(input).unwrap();

    //     let expected_tokens = vec![
    //         Token::Var,
    //         Token::Identifier("and2".to_string()),
    //         Token::Equal,
    //         Token::Number(10.0),
    //         Token::Semicolon,
    //     ];
    //     assert_eq!(tokens, expected_tokens);
    // }

    // #[test]
    // fn test_scan_line_3() {
    //     let input = "andfunc for;  // This is a comment";
    //     let (remaining, tokens) = scan_line(input).unwrap();

    //     let expected_tokens = vec![
    //         Token::Identifier("andfunc".to_string()),
    //         Token::For,
    //         Token::Semicolon,
    //         Token::LineComment(" This is a comment".to_string()),
    //     ];
    //     assert_eq!(tokens, expected_tokens);
    // }
}