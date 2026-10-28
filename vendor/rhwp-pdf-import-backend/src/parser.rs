//! PDF object parser.
//!
//! This module provides parsing of PDF objects by combining tokens from the lexer
//! into complete objects (arrays, dictionaries, indirect references, etc.).
//!
//! # Architecture
//!
//! The parser uses a recursive descent approach:
//! 1. Read token from lexer
//! 2. Based on token type, decide how to parse
//! 3. For composite types (arrays, dicts), recursively parse contents
//!
//! # Error Handling
//!
//! All parsing functions return `IResult` from nom. Parse errors contain
//! descriptive messages about what went wrong and where.

use crate::error::{Error, Result};
use crate::import_limits::ImportBudget;
use crate::lexer::{token, token_lenient, Token};
use crate::object::{Object, ObjectRef};
use nom::IResult;
use std::collections::HashMap;
use std::sync::Arc;

/// Decode escape sequences in PDF literal strings.
///
/// PDF literal strings (enclosed in parentheses) support escape sequences
/// per ISO 32000-1:2008, Section 7.3.4.2:
///
/// - `\n` → Line Feed (0x0A)
/// - `\r` → Carriage Return (0x0D)
/// - `\t` → Horizontal Tab (0x09)
/// - `\b` → Backspace (0x08)
/// - `\f` → Form Feed (0x0C)
/// - `\(` → Left Parenthesis
/// - `\)` → Right Parenthesis
/// - `\\` → Backslash
/// - `\ddd` → Character with octal code (1-3 digits)
/// - `\<newline>` → Line continuation (ignored)
///
/// # Examples
///
/// ```
/// # use pdf_oxide::parser::decode_literal_string_escapes;
/// let input = b"Section \\247 71.01";
/// let decoded = decode_literal_string_escapes(input);
/// assert_eq!(decoded, b"Section \xa7 71.01"); // \247 = § (section sign)
/// ```
pub fn decode_literal_string_escapes(raw: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(raw.len());
    let mut i = 0;

    while i < raw.len() {
        if raw[i] == b'\\' && i + 1 < raw.len() {
            match raw[i + 1] {
                // Single character escapes
                b'n' => {
                    result.push(b'\n');
                    i += 2;
                },
                b'r' => {
                    result.push(b'\r');
                    i += 2;
                },
                b't' => {
                    result.push(b'\t');
                    i += 2;
                },
                b'b' => {
                    result.push(8); // Backspace
                    i += 2;
                },
                b'f' => {
                    result.push(12); // Form feed
                    i += 2;
                },
                b'(' => {
                    result.push(b'(');
                    i += 2;
                },
                b')' => {
                    result.push(b')');
                    i += 2;
                },
                b'\\' => {
                    result.push(b'\\');
                    i += 2;
                },
                // Line continuation: \<newline> is ignored
                b'\n' => {
                    i += 2; // Skip backslash and newline
                },
                b'\r' => {
                    // Handle \r or \r\n
                    i += 2;
                    if i < raw.len() && raw[i] == b'\n' {
                        i += 1;
                    }
                },
                // Octal escape: \ddd (1-3 octal digits)
                c if c.is_ascii_digit() && c < b'8' => {
                    let start = i + 1;
                    let mut octal_value = 0u32;
                    let mut octal_len = 0;

                    // Read up to 3 octal digits
                    for j in 0..3 {
                        if start + j < raw.len() {
                            let digit = raw[start + j];
                            if (b'0'..b'8').contains(&digit) {
                                octal_value = octal_value * 8 + (digit - b'0') as u32;
                                octal_len += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if octal_len > 0 {
                        // Octal value must fit in a byte
                        result.push((octal_value & 0xFF) as u8);
                        i += 1 + octal_len; // Skip backslash + digits
                    } else {
                        // Not a valid octal, keep backslash as-is
                        result.push(b'\\');
                        i += 1;
                    }
                },
                // Unknown escape: keep backslash literal (PDF spec allows this)
                _ => {
                    result.push(b'\\');
                    i += 1;
                },
            }
        } else {
            // Regular character
            result.push(raw[i]);
            i += 1;
        }
    }

    result
}

/// Parse a PDF object from input bytes.
///
/// This is the main entry point for parsing PDF objects. It handles all
/// PDF object types:
/// - Primitives: null, boolean, integer, real, string, name
/// - Composites: array, dictionary
/// - References: indirect object references (10 0 R)
///
/// # Example
///
/// ```
/// use pdf_oxide::parser::parse_object;
///
/// let input = b"[ 1 2 /Name ]";
/// let (remaining, obj) = parse_object(input).unwrap();
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Input is not a valid PDF object
/// - Nested structures are malformed (unclosed arrays/dicts)
/// - Hex strings contain invalid hex digits
pub fn parse_object(input: &[u8]) -> IResult<&[u8], Object> {
    parse_object_at(input, 0, None, true)
}

/// Parse one object while retaining its request context on stream bytes.
pub fn parse_object_with_budget<'a>(
    input: &'a [u8],
    budget: Option<&Arc<ImportBudget>>,
) -> IResult<&'a [u8], Object> {
    parse_object_at(input, 0, budget, true)
}

/// Resolve a top-level stream's indirect Length before slicing binary bytes.
/// ISO 32000-1 7.3.8.2 permits an indirect length. A checksum byte equal to CR
/// followed by the delimiter LF must not be mistaken for a CRLF delimiter.
pub(crate) fn parse_indirect_with_budget(
    input: &[u8],
    budget: &Arc<ImportBudget>,
    resolve: impl Fn(ObjectRef) -> Result<Object>,
) -> Result<Object> {
    parse_indirect_prefix_with_budget(input, budget, resolve).map(|(_, object)| object)
}

pub(crate) fn parse_indirect_prefix_with_budget<'a>(
    input: &'a [u8],
    budget: &Arc<ImportBudget>,
    resolve: impl Fn(ObjectRef) -> Result<Object>,
) -> Result<(&'a [u8], Object)> {
    let parse_error = || Error::Decode("invalid bounded indirect object".into());
    let (remaining, tok) = token(input).map_err(|_| parse_error())?;
    if !matches!(tok, Token::DictStart) {
        let parsed = parse_object_with_budget(input, Some(budget));
        budget.check()?;
        return parsed.map_err(|_| parse_error());
    }
    budget.charge_object()?;
    let parsed = parse_dictionary(remaining, 0, Some(budget), true);
    budget.check()?;
    let (remaining, object) = parsed.map_err(|_| parse_error())?;
    let Ok((stream_input, Token::StreamStart)) = token(remaining) else {
        return Ok((remaining, object));
    };
    let Object::Dictionary(dict) = object else {
        unreachable!()
    };
    let mut length = dict.get("Length").cloned();
    for _ in 0..budget.limits.reference_depth {
        let Some(Object::Reference(reference)) = length.as_ref() else {
            break;
        };
        length = Some(resolve(*reference)?);
    }
    let length = match length {
        Some(Object::Reference(_)) => return Err(budget.reject("stream Length reference depth")),
        Some(Object::Integer(n)) => Some(usize::try_from(n).map_err(|_| parse_error())?),
        None | Some(Object::Null) => None,
        _ => return Err(parse_error()),
    };
    let parsed = parse_stream_data(stream_input, &dict, Some(budget), length);
    budget.check()?;
    let (remaining, data) = parsed.map_err(|_| parse_error())?;
    Ok((
        remaining,
        Object::Stream {
            dict,
            data: crate::import_limits::StreamData::bounded(data, Some(budget)),
        },
    ))
}

pub(crate) fn parse_content_operand<'a>(
    input: &'a [u8],
    budget: Option<&Arc<ImportBudget>>,
) -> IResult<&'a [u8], Object> {
    parse_object_at(input, 0, budget, false)
}

fn budget_error(input: &[u8]) -> nom::Err<nom::error::Error<&[u8]>> {
    nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge))
}

// Keep the depth guard outside the large object-parser frame.
fn parse_object_at<'a>(
    input: &'a [u8],
    depth: usize,
    budget: Option<&Arc<ImportBudget>>,
    container_values: bool,
) -> IResult<&'a [u8], Object> {
    if let Some(budget) = budget {
        budget.depth(depth).map_err(|_| budget_error(input))?;
        if container_values {
            budget.charge_object().map_err(|_| budget_error(input))?;
        }
    }
    parse_object_body(input, depth, budget, container_values)
}

fn parse_object_body<'a>(
    input: &'a [u8],
    depth: usize,
    budget: Option<&Arc<ImportBudget>>,
    container_values: bool,
) -> IResult<&'a [u8], Object> {
    let (input, tok) = token(input)?;

    match tok {
        Token::Null => Ok((input, Object::Null)),
        Token::True => Ok((input, Object::Boolean(true))),
        Token::False => Ok((input, Object::Boolean(false))),

        Token::Integer(i) => {
            // Could be a plain integer OR the start of an indirect reference (obj_num gen R)
            // Try to parse as reference first

            // Look ahead for generation number
            if let Ok((input2, Token::Integer(gen))) = token(input) {
                // Look ahead for 'R' token
                if let Ok((input3, Token::R)) = token(input2) {
                    // Successfully parsed indirect reference
                    return Ok((input3, Object::Reference(ObjectRef::new(i as u32, gen as u16))));
                }
            }

            // Not a reference, just a plain integer
            Ok((input, Object::Integer(i)))
        },

        Token::Real(r) => Ok((input, Object::Real(r))),

        Token::LiteralString(bytes) => {
            if let Some(budget) = budget {
                if bytes.len() > budget.limits.string_bytes {
                    budget.reject("literal string bytes");
                    return Err(budget_error(input));
                }
            }
            // Decode escape sequences per ISO 32000-1:2008, Section 7.3.4.2
            let decoded = decode_literal_string_escapes(bytes);
            Ok((input, Object::String(decoded)))
        },

        Token::HexString(hex_bytes) => {
            if let Some(budget) = budget {
                if hex_bytes.len() / 2 > budget.limits.string_bytes {
                    budget.reject("hex string bytes");
                    return Err(budget_error(input));
                }
            }
            // Decode hex string to bytes
            match decode_hex(hex_bytes) {
                Ok(decoded) => Ok((input, Object::String(decoded))),
                Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Fail,
                ))),
            }
        },

        Token::Name(name) => Ok((input, Object::Name(name))),

        Token::ArrayStart => parse_array(input, depth, budget, container_values),

        Token::DictStart => {
            // Parse dictionary, then check if it's followed by a stream
            let (remaining, dict_obj) = parse_dictionary(input, depth, budget, container_values)?;

            // Check if next token is 'stream' keyword
            if let Ok((stream_input, Token::StreamStart)) = token(remaining) {
                // This is a stream object
                // Extract the dictionary
                let dict = match dict_obj {
                    Object::Dictionary(d) => d,
                    _ => {
                        // parse_dictionary guarantees Dictionary return type
                        // This should never happen, but handle gracefully if it does
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    },
                };

                // Parse the stream data
                let (final_input, stream_data) =
                    parse_stream_data(stream_input, &dict, budget, None)?;

                return Ok((
                    final_input,
                    Object::Stream {
                        dict,
                        data: crate::import_limits::StreamData::bounded(stream_data, budget),
                    },
                ));
            }

            // Not a stream, just return the dictionary
            Ok((remaining, dict_obj))
        },

        _ => {
            // Unexpected token
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        },
    }
}

/// Parse stream data after the `stream` keyword.
///
/// Stream data starts after a newline following `stream` and ends with `endstream`.
/// The Length entry in the dictionary tells us how many bytes to read.
///
/// PDF Spec: ISO 32000-1:2008, Section 7.3.8.1 - Stream Objects
/// The keyword stream must be followed by either a CRLF or LF sequence, but not CR alone.
fn parse_stream_data<'a>(
    input: &'a [u8],
    dict: &HashMap<String, Object>,
    budget: Option<&Arc<ImportBudget>>,
    resolved_length: Option<usize>,
) -> IResult<&'a [u8], Vec<u8>> {
    // SPEC COMPLIANCE: PDF Spec ISO 32000-1:2008, Section 7.3.8.1 states that
    // the 'stream' keyword must be followed by either CRLF or LF, but NOT CR alone.
    //
    // We accept CR alone in lenient mode for compatibility with malformed PDFs,
    // but log a warning. In strict mode, this should be an error (requires ParserOptions).

    let input = if input.starts_with(b"\r\n") {
        // CRLF - correct per PDF spec
        &input[2..]
    } else if input.starts_with(b"\n") {
        // LF - correct per PDF spec
        &input[1..]
    } else if input.starts_with(b"\r") {
        // CR alone - SPEC VIOLATION
        // PDF Spec ISO 32000-1:2008, Section 7.3.8.1 requires CRLF or LF, not CR alone
        let msg = "SPEC VIOLATION: Stream keyword followed by CR alone (should be CRLF or LF). \
            Accepting in lenient mode for compatibility. \
            PDF Spec: ISO 32000-1:2008, Section 7.3.8.1";
        log::warn!("{}", msg);
        // push into the process-wide structured sink
        // so callers can retrieve via `flatten_warnings()` instead of
        // parsing stderr from `log::warn`.
        crate::extractors::warnings::push_global_warning(crate::extractors::warnings::Warning {
            category: crate::extractors::warnings::WarningCategory::SpecViolation,
            page: None,
            message: msg.to_string(),
            spec_section: Some("7.3.8.1"),
        });
        &input[1..]
    } else {
        // No newline after 'stream' - SPEC VIOLATION
        let msg = "SPEC VIOLATION: No newline after stream keyword (should be CRLF or LF). \
            PDF Spec: ISO 32000-1:2008, Section 7.3.8.1";
        log::warn!("{}", msg);
        crate::extractors::warnings::push_global_warning(crate::extractors::warnings::Warning {
            category: crate::extractors::warnings::WarningCategory::SpecViolation,
            page: None,
            message: msg.to_string(),
            spec_section: Some("7.3.8.1"),
        });
        input
    };

    // Get the length from the dictionary
    if let Some(length) = resolved_length.or_else(|| {
        dict.get("Length")
            .and_then(Object::as_integer)
            .and_then(|n| usize::try_from(n).ok())
    }) {
        {
            if length <= input.len() {
                // Read exactly 'length' bytes
                let stream_data =
                    copy_stream(&input[..length], budget).map_err(|_| budget_error(input))?;
                let remaining = &input[length..];

                // Skip whitespace and expect 'endstream'
                let ws_result: nom::IResult<&[u8], &[u8]> =
                    nom::bytes::complete::take_while(|c: u8| c.is_ascii_whitespace())(remaining);
                if let Ok((remaining, _)) = ws_result {
                    if let Ok((remaining, Token::StreamEnd)) = token(remaining) {
                        return Ok((remaining, stream_data));
                    }
                }
            }

            // /Length was wrong — fall back to scanning for 'endstream'
            let msg =
                format!("Stream /Length {} is incorrect, falling back to endstream scan", length);
            log::warn!("{}", msg);
            crate::extractors::warnings::push_global_warning(
                crate::extractors::warnings::Warning {
                    category: crate::extractors::warnings::WarningCategory::SpecViolation,
                    page: None,
                    message: msg,
                    spec_section: Some("7.3.8.2"),
                },
            );
            if let Some(pos) = find_endstream(input) {
                // Strip trailing CR/LF per PDF spec Section 7.3.8.1
                let mut end = pos;
                if end > 0 && input[end - 1] == b'\n' {
                    end -= 1;
                }
                if end > 0 && input[end - 1] == b'\r' {
                    end -= 1;
                }
                let stream_data =
                    copy_stream(&input[..end], budget).map_err(|_| budget_error(input))?;
                let remaining = &input[pos..];
                let (remaining, _) = token(remaining)?;
                return Ok((remaining, stream_data));
            }
        }
    }

    // SPEC DEVIATION (LOW PRIORITY): PDF Spec ISO 32000-1:2008, Section 7.3.8.1
    // requires stream dictionaries to have a /Length entry. This fallback scans for
    // 'endstream' keyword when /Length is missing or invalid.
    //
    // Rationale: Many malformed PDFs in the wild lack correct /Length values.
    // This heuristic makes parsing more robust at the cost of spec compliance.
    //
    // Proper implementation: Only use this fallback in lenient mode (ParserOptions).
    // In strict mode, should fail with error when /Length is missing/invalid.
    //
    // If no Length or invalid Length, scan for 'endstream' keyword
    // This is less reliable but acts as fallback
    if let Some(pos) = find_endstream(input) {
        // Strip trailing CR/LF per PDF spec Section 7.3.8.1
        let mut end = pos;
        if end > 0 && input[end - 1] == b'\n' {
            end -= 1;
        }
        if end > 0 && input[end - 1] == b'\r' {
            end -= 1;
        }
        let stream_data = copy_stream(&input[..end], budget).map_err(|_| budget_error(input))?;
        let remaining = &input[pos..];

        // Skip 'endstream' keyword
        let (remaining, _) = token(remaining)?;

        return Ok((remaining, stream_data));
    }

    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)))
}

/// Find the position of 'endstream' keyword in input.
fn find_endstream(input: &[u8]) -> Option<usize> {
    let keyword = b"endstream";
    input
        .windows(keyword.len())
        .position(|window| window == keyword)
}

/// Parse a PDF array: `[ obj1 obj2 ... objN ]`
///
/// Arrays can contain any PDF objects, including nested arrays and dictionaries.
/// Empty arrays are valid: `[]`
///
/// # Example
///
/// ```
/// use pdf_oxide::parser::parse_object;
///
/// let input = b"[ 1 2 /Name (string) [ 3 4 ] ]";
/// let (_, obj) = parse_object(input).unwrap();
/// assert!(obj.as_array().is_some());
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Array is not properly closed with `]`
/// - Array contains malformed objects
fn parse_array<'a>(
    input: &'a [u8],
    depth: usize,
    budget: Option<&Arc<ImportBudget>>,
    container_values: bool,
) -> IResult<&'a [u8], Object> {
    let mut objects = Vec::new();
    let mut remaining = input;

    loop {
        // Try to get next token
        let token_result = token(remaining);

        match token_result {
            Ok((inp, tok)) => {
                // Check for array end
                if tok == Token::ArrayEnd {
                    return Ok((inp, Object::Array(objects)));
                }

                // Otherwise, we need to parse this as an object
                // Put the token back by re-parsing from remaining
                if let Some(budget) = budget {
                    budget
                        .charge_entries(1)
                        .map_err(|_| budget_error(remaining))?;
                    objects.try_reserve(1).map_err(|_| {
                        budget.reject("array allocation");
                        budget_error(remaining)
                    })?;
                }
                match parse_object_at(remaining, depth + 1, budget, container_values) {
                    Ok((inp, obj)) => {
                        objects.push(obj);
                        remaining = inp;
                    },
                    Err(e) => {
                        // If we can't parse an object, check if it's EOF
                        if remaining.is_empty() {
                            // Unclosed array, but return what we have
                            return Ok((remaining, Object::Array(objects)));
                        }
                        return Err(e);
                    },
                }
            },
            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) if remaining.is_empty() => {
                // Hit EOF before closing array - return what we have
                return Ok((remaining, Object::Array(objects)));
            },
            Err(e) => return Err(e),
        }
    }
}

/// Parse a PDF dictionary: `<< /Key1 value1 /Key2 value2 ... >>`
///
/// Dictionary keys must be names (starting with /). Values can be any PDF object.
/// Empty dictionaries are valid: `<< >>`
///
/// # Example
///
/// ```
/// use pdf_oxide::parser::parse_object;
///
/// let input = b"<< /Type /Page /Count 3 >>";
/// let (_, obj) = parse_object(input).unwrap();
/// assert!(obj.as_dict().is_some());
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Dictionary is not properly closed with `>>`
/// - A key is not a name
/// - Values are malformed
fn parse_dictionary<'a>(
    input: &'a [u8],
    depth: usize,
    budget: Option<&Arc<ImportBudget>>,
    container_values: bool,
) -> IResult<&'a [u8], Object> {
    let mut dict = HashMap::new();
    let mut remaining = input;

    loop {
        // Try to get next token
        let token_result = token(remaining);

        match token_result {
            Ok((inp, tok)) => {
                // Check for dictionary end
                if tok == Token::DictEnd {
                    return Ok((inp, Object::Dictionary(dict)));
                }

                // Otherwise, expect a name as key
                match tok {
                    Token::Name(key) => {
                        if let Some(budget) = budget {
                            budget.charge_entries(1).map_err(|_| budget_error(inp))?;
                            dict.try_reserve(1).map_err(|_| {
                                budget.reject("dictionary allocation");
                                budget_error(inp)
                            })?;
                        }
                        // Parse the value; fall back to lenient token for bare words
                        // (e.g. `OBJR` without a `/` prefix in malformed PDFs)
                        let value_result =
                            parse_object_at(inp, depth + 1, budget, container_values).or_else(
                                |error| {
                                    if budget.is_some_and(|b| b.check().is_err()) {
                                        return Err(error);
                                    }
                                    token_lenient(inp).and_then(|(inp2, tok)| {
                                        if let Token::Name(name) = tok {
                                            Ok((inp2, Object::Name(name)))
                                        } else {
                                            Err(nom::Err::Error(nom::error::Error::new(
                                                inp,
                                                nom::error::ErrorKind::Alt,
                                            )))
                                        }
                                    })
                                },
                            );
                        match value_result {
                            Ok((inp, value)) => {
                                dict.insert(key, value);
                                remaining = inp;
                            },
                            Err(e) => {
                                // If we can't parse the value, check if it's EOF
                                if inp.is_empty() {
                                    // Incomplete dictionary, return what we have
                                    return Ok((inp, Object::Dictionary(dict)));
                                }
                                return Err(e);
                            },
                        }
                    },
                    _ => {
                        // Invalid dictionary - key must be a name
                        // But if we hit EOF, return what we have
                        if remaining.is_empty() {
                            return Ok((remaining, Object::Dictionary(dict)));
                        }
                        return Err(nom::Err::Error(nom::error::Error::new(
                            remaining,
                            nom::error::ErrorKind::Tag,
                        )));
                    },
                }
            },
            Err(nom::Err::Incomplete(_)) | Err(nom::Err::Error(_)) if remaining.is_empty() => {
                // Hit EOF before closing dictionary - return what we have
                return Ok((remaining, Object::Dictionary(dict)));
            },
            Err(e) => return Err(e),
        }
    }
}

fn copy_stream(data: &[u8], budget: Option<&Arc<ImportBudget>>) -> Result<Vec<u8>> {
    if let Some(budget) = budget {
        let mut output = budget.output();
        output.append(data)?;
        Ok(output.finish())
    } else {
        Ok(data.to_vec())
    }
}

/// Decode a hex string to bytes.
///
/// PDF hex strings contain pairs of hexadecimal digits representing bytes.
/// Whitespace is ignored. If there's an odd number of hex digits, the last
/// digit is padded with 0.
///
/// # Example
///
/// ```
/// use pdf_oxide::parser::decode_hex;
///
/// let decoded = decode_hex(b"48656C6C6F").unwrap();
/// assert_eq!(decoded, b"Hello");
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Input contains non-hex, non-whitespace characters
/// - Hex digit parsing fails
pub fn decode_hex(hex_bytes: &[u8]) -> Result<Vec<u8>> {
    // Filter out whitespace
    let hex_str: Vec<u8> = hex_bytes
        .iter()
        .filter(|&&c| !c.is_ascii_whitespace())
        .copied()
        .collect();

    // Handle empty hex string
    if hex_str.is_empty() {
        return Ok(Vec::new());
    }

    // Per PDF spec ISO 32000-1:2008 §7.3.4.3: non-hex characters are treated as 0
    fn hex_val(b: u8) -> u8 {
        match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => 0, // Non-hex chars treated as 0 per spec
        }
    }

    let mut result = Vec::with_capacity(hex_str.len() / 2 + 1);

    // Process pairs of hex digits
    for chunk in hex_str.chunks(2) {
        match chunk.len() {
            2 => {
                let byte = (hex_val(chunk[0]) << 4) | hex_val(chunk[1]);
                result.push(byte);
            },
            1 => {
                // Odd number of hex digits - pad last digit with 0
                let byte = hex_val(chunk[0]) << 4;
                result.push(byte);
            },
            _ => {
                // chunks(2) guarantees max 2 elements, this should never execute
                return Err(Error::ParseError {
                    offset: 0,
                    reason: "Invalid hex string chunk size".to_string(),
                });
            },
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Primitive Type Tests
    // ========================================================================

    #[test]
    fn test_parse_null() {
        let input = b"null";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Null);
    }

    #[test]
    fn test_parse_boolean_true() {
        let input = b"true";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Boolean(true));
    }

    #[test]
    fn test_parse_boolean_false() {
        let input = b"false";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Boolean(false));
    }

    #[test]
    fn test_parse_integer() {
        let input = b"42";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Integer(42));
    }

    #[test]
    fn test_parse_negative_integer() {
        let input = b"-123";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Integer(-123));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_real() {
        let input = b"3.14";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Real(3.14));
    }

    #[test]
    fn test_parse_name() {
        let input = b"/Type";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Name("Type".to_string()));
    }

    #[test]
    fn test_parse_literal_string() {
        let input = b"(Hello World)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Hello World".to_vec()));
    }

    #[test]
    fn test_parse_empty_literal_string() {
        let input = b"()";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"".to_vec()));
    }

    // ========================================================================
    // Escape Sequence Tests (ISO 32000-1:2008, Section 7.3.4.2)
    // ========================================================================

    #[test]
    fn test_escape_sequence_newline() {
        let input = b"(Line1\\nLine2)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Line1\nLine2".to_vec()));
    }

    #[test]
    fn test_escape_sequence_carriage_return() {
        let input = b"(Line1\\rLine2)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Line1\rLine2".to_vec()));
    }

    #[test]
    fn test_escape_sequence_tab() {
        let input = b"(Col1\\tCol2)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Col1\tCol2".to_vec()));
    }

    #[test]
    fn test_escape_sequence_backspace() {
        let input = b"(Text\\bmore)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Text\x08more".to_vec()));
    }

    #[test]
    fn test_escape_sequence_form_feed() {
        let input = b"(Page1\\fPage2)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Page1\x0CPage2".to_vec()));
    }

    #[test]
    fn test_escape_sequence_parentheses() {
        let input = b"(Open \\( Close \\))";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Open ( Close )".to_vec()));
    }

    #[test]
    fn test_escape_sequence_backslash() {
        let input = b"(Path\\\\to\\\\file)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Path\\to\\file".to_vec()));
    }

    #[test]
    fn test_escape_sequence_octal_three_digits() {
        // \247 = octal 247 = decimal 167 = 0xA7 = § (section sign)
        let input = b"(Section \\247)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Section \xa7".to_vec()));
    }

    #[test]
    fn test_escape_sequence_octal_two_digits() {
        // \53 = octal 53 = decimal 43 = 0x2B = '+'
        let input = b"(Plus \\53)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Plus +".to_vec()));
    }

    #[test]
    fn test_escape_sequence_octal_one_digit() {
        // \7 = octal 7 = decimal 7 = bell character
        let input = b"(Bell \\7)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Bell \x07".to_vec()));
    }

    #[test]
    fn test_escape_sequence_octal_stops_at_non_octal() {
        // \128 = \12 (octal 12 = 10) + '8' (literal)
        let input = b"(Value \\128)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        // \12 = octal 12 = decimal 10 = newline
        assert_eq!(obj, Object::String(b"Value \n8".to_vec()));
    }

    #[test]
    fn test_escape_sequence_real_pdf_case() {
        // This is the actual case from XYUJKKMUXDLLC6JTCXEWHK5ZMNSTPHF6.pdf
        // \247 = § (section sign), \261 = ± (plus-minus)
        let input = b"(\\247 71.01\\26115 Temporary certificate.)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        // \247 = 0xA7 = §, \261 = 0xB1 = ±
        assert_eq!(obj, Object::String(b"\xa7 71.01\xb115 Temporary certificate.".to_vec()));
    }

    #[test]
    fn test_escape_sequence_line_continuation() {
        // \<newline> is ignored (line continuation)
        let input = b"(This is a long \\\nstring)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"This is a long string".to_vec()));
    }

    #[test]
    fn test_escape_sequence_mixed() {
        let input = b"(Tab:\\tNewline:\\nOctal:\\53)";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Tab:\tNewline:\nOctal:+".to_vec()));
    }

    #[test]
    fn test_decode_literal_string_escapes_directly() {
        // Test the decoder function directly
        assert_eq!(decode_literal_string_escapes(b"Hello"), b"Hello");
        assert_eq!(decode_literal_string_escapes(b"\\n"), b"\n");
        assert_eq!(decode_literal_string_escapes(b"\\247"), b"\xa7");
        assert_eq!(decode_literal_string_escapes(b"\\(\\)"), b"()");
        assert_eq!(decode_literal_string_escapes(b"\\\\"), b"\\");
    }

    // ========================================================================
    // Hex String Tests
    // ========================================================================

    #[test]
    fn test_parse_hex_string() {
        let input = b"<48656C6C6F>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Hello".to_vec()));
    }

    #[test]
    fn test_parse_hex_string_with_whitespace() {
        let input = b"<48 65 6C 6C 6F>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"Hello".to_vec()));
    }

    #[test]
    fn test_parse_empty_hex_string() {
        let input = b"<>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::String(b"".to_vec()));
    }

    #[test]
    fn test_parse_hex_string_odd_length() {
        // Odd number of hex digits - last digit padded with 0
        let input = b"<ABC>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        // ABC -> AB C0 -> 171, 192
        assert_eq!(obj, Object::String(vec![0xAB, 0xC0]));
    }

    #[test]
    fn test_decode_hex() {
        let result = decode_hex(b"48656C6C6F").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_with_whitespace() {
        let result = decode_hex(b"48 65 6C 6C 6F").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_empty() {
        let result = decode_hex(b"").unwrap();
        assert_eq!(result, b"");
    }

    #[test]
    fn test_decode_hex_odd_length() {
        let result = decode_hex(b"ABC").unwrap();
        // ABC -> AB C0
        assert_eq!(result, vec![0xAB, 0xC0]);
    }

    // ========================================================================
    // Indirect Reference Tests
    // ========================================================================

    #[test]
    fn test_parse_indirect_reference() {
        let input = b"10 0 R";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Reference(ObjectRef::new(10, 0)));
    }

    #[test]
    fn test_parse_indirect_reference_with_generation() {
        let input = b"42 5 R";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Reference(ObjectRef::new(42, 5)));
    }

    #[test]
    fn test_parse_integer_not_reference() {
        // Just "10" without "0 R" should parse as integer
        let input = b"10";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Integer(10));
    }

    // ========================================================================
    // Array Tests
    // ========================================================================

    #[test]
    fn test_parse_empty_array() {
        let input = b"[]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Array(vec![]));
    }

    #[test]
    fn test_parse_array_with_integers() {
        let input = b"[ 1 2 3 ]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(
            obj,
            Object::Array(vec![Object::Integer(1), Object::Integer(2), Object::Integer(3),])
        );
    }

    #[test]
    fn test_parse_array_mixed_types() {
        let input = b"[ 1 /Name (string) true ]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(
            obj,
            Object::Array(vec![
                Object::Integer(1),
                Object::Name("Name".to_string()),
                Object::String(b"string".to_vec()),
                Object::Boolean(true),
            ])
        );
    }

    #[test]
    fn test_parse_nested_arrays() {
        let input = b"[ 1 [ 2 3 ] 4 ]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(
            obj,
            Object::Array(vec![
                Object::Integer(1),
                Object::Array(vec![Object::Integer(2), Object::Integer(3)]),
                Object::Integer(4),
            ])
        );
    }

    #[test]
    fn test_parse_array_with_references() {
        let input = b"[ 10 0 R 20 0 R ]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(
            obj,
            Object::Array(vec![
                Object::Reference(ObjectRef::new(10, 0)),
                Object::Reference(ObjectRef::new(20, 0)),
            ])
        );
    }

    // ========================================================================
    // Dictionary Tests
    // ========================================================================

    #[test]
    fn test_parse_empty_dictionary() {
        let input = b"<<>>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Dictionary(HashMap::new()));
    }

    #[test]
    fn test_parse_dictionary_single_entry() {
        let input = b"<< /Type /Page >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.len(), 1);
        assert_eq!(dict.get("Type").unwrap().as_name(), Some("Page"));
    }

    #[test]
    fn test_parse_dictionary_multiple_entries() {
        let input = b"<< /Type /Page /Count 3 /Title (My Page) >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.len(), 3);
        assert_eq!(dict.get("Type").unwrap().as_name(), Some("Page"));
        assert_eq!(dict.get("Count").unwrap().as_integer(), Some(3));
        assert_eq!(dict.get("Title").unwrap().as_string(), Some(&b"My Page"[..]));
    }

    #[test]
    fn test_parse_dictionary_with_array() {
        let input = b"<< /MediaBox [ 0 0 612 792 ] >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.len(), 1);
        let media_box = dict.get("MediaBox").unwrap().as_array().unwrap();
        assert_eq!(media_box.len(), 4);
    }

    #[test]
    fn test_parse_nested_dictionaries() {
        let input = b"<< /Outer << /Inner /Value >> >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        let inner = dict.get("Outer").unwrap().as_dict().unwrap();
        assert_eq!(inner.get("Inner").unwrap().as_name(), Some("Value"));
    }

    #[test]
    fn test_parse_dictionary_with_reference() {
        let input = b"<< /Pages 2 0 R >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get("Pages").unwrap().as_reference(), Some(ObjectRef::new(2, 0)));
    }

    // ========================================================================
    // Complex Nested Structure Tests
    // ========================================================================

    #[test]
    fn test_parse_complex_nested_structure() {
        let input = b"<< /Type /Catalog /Pages [ 1 0 R 2 0 R ] /Metadata << /Author (John) >> >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);

        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get("Type").unwrap().as_name(), Some("Catalog"));

        let pages = dict.get("Pages").unwrap().as_array().unwrap();
        assert_eq!(pages.len(), 2);

        let metadata = dict.get("Metadata").unwrap().as_dict().unwrap();
        assert_eq!(metadata.get("Author").unwrap().as_string(), Some(&b"John"[..]));
    }

    // ========================================================================
    // Error Cases
    // ========================================================================

    #[test]
    fn test_parse_unclosed_array() {
        // Lenient parsing: unclosed arrays return what they have
        let input = b"[ 1 2 3";
        let result = parse_object(input);
        assert!(result.is_ok());
        let (_, obj) = result.unwrap();
        let arr = obj.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_integer(), Some(1));
        assert_eq!(arr[1].as_integer(), Some(2));
        assert_eq!(arr[2].as_integer(), Some(3));
    }

    #[test]
    fn test_parse_unclosed_dictionary() {
        // Lenient parsing: unclosed dictionaries return what they have
        let input = b"<< /Type /Page";
        let result = parse_object(input);
        assert!(result.is_ok());
        let (_, obj) = result.unwrap();
        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get("Type").and_then(|o| o.as_name()), Some("Page"));
    }

    #[test]
    fn test_parse_dictionary_missing_value() {
        let input = b"<< /Type >>";
        let result = parse_object(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dictionary_non_name_key() {
        let input = b"<< 123 /Value >>";
        let result = parse_object(input);
        assert!(result.is_err());
    }

    // ========================================================================
    // Whitespace Handling Tests
    // ========================================================================

    #[test]
    fn test_parse_with_leading_whitespace() {
        let input = b"  \n\t  42";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        assert_eq!(obj, Object::Integer(42));
    }

    #[test]
    fn test_parse_array_with_extra_whitespace() {
        let input = b"[  1   2    3  ]";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        let arr = obj.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_parse_dictionary_with_extra_whitespace() {
        let input = b"<<  /Type   /Page  >>";
        let (remaining, obj) = parse_object(input).unwrap();
        assert_eq!(remaining, &b""[..]);
        let dict = obj.as_dict().unwrap();
        assert_eq!(dict.get("Type").unwrap().as_name(), Some("Page"));
    }
}
