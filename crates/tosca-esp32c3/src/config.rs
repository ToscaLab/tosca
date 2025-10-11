use alloc::string::String;
use alloc::vec::Vec;

use log::info;

use serde::Deserialize;

/*use toml_parser::decoder::ScalarKind;
use toml_parser::parser::{parse_document, Event, EventKind, ValidateWhitespace};
use toml_parser::ErrorSink as _;
use toml_parser::Span;
use toml_parser::{ParseError, Source};*/

#[derive(Deserialize, Debug)]
struct Device {
    kind: String,
}

#[derive(Deserialize, Debug)]
struct MandatoryRoute {
    path: String,
    hazards: Vec<String>,
    parameters: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Mandatory {
    on: MandatoryRoute,
}

#[derive(Deserialize, Debug)]
struct Parameter {
    kind: String,
    default: f64,
    min: f64,
    max: f64,
}

#[derive(Deserialize, Debug)]
struct Parameters {
    brightness: Parameter,
}

#[derive(Deserialize, Debug)]
struct LocalizedText {
    #[serde(rename = "device.name")]
    device_name: String,
    #[serde(rename = "device.description")]
    device_description: String,
    #[serde(rename = "on.name")]
    on_name: String,
    #[serde(rename = "on.description")]
    on_description: String,
    #[serde(rename = "brightness.name")]
    brightness_name: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    device: Device,
    mandatory: Mandatory,
    parameters: Parameters,
    en: LocalizedText,
    it: LocalizedText,
}

/*fn validate(doc: Source<'_>, events: Vec<Event>, mut errors: Vec<ParseError>) {
    for event in &events {
        if event.kind() == EventKind::SimpleKey {
            let raw = doc.get(event).unwrap();
            let mut value = String::new();
            raw.decode_key(&mut value, &mut errors);
            info!("Parola chiave: {}", value);
        } else if event.kind() == EventKind::Comment {
            let raw = doc.get(event).unwrap();
            raw.decode_comment(&mut errors);
        } else if event.kind() == EventKind::Newline {
            let raw = doc.get(event).unwrap();
            raw.decode_newline(&mut errors);
        } else if event.kind() == EventKind::Scalar {
            let raw = doc.get(event).unwrap();
            let mut value = String::new();
            let kind = raw.decode_scalar(&mut value, &mut errors);
            match kind {
                ScalarKind::String => {
                    info!("Stringa: {}", value);
                }
                ScalarKind::Boolean(v) => {
                    let value = value.parse::<bool>();
                    if value.is_err() {
                        errors.report_error(
                            ParseError::new("failed to parse bool")
                                .with_context(Span::new_unchecked(0, raw.len()))
                                .with_unexpected(Span::new_unchecked(0, 2)),
                        );
                    } else if value != Ok(v) {
                        errors.report_error(
                            ParseError::new("mismatched bool value")
                                .with_context(Span::new_unchecked(0, raw.len()))
                                .with_unexpected(Span::new_unchecked(0, 2)),
                        );
                    }
                }
                ScalarKind::DateTime => {
                    /*let value = value.parse::<toml_datetime::Datetime>();
                    if value.is_err() {
                        errors.report_error(
                            ParseError::new("failed to parse datetime")
                                .with_context(Span::new_unchecked(0, raw.len()))
                                .with_unexpected(Span::new_unchecked(0, 2)),
                        );
                    }*/
                }
                ScalarKind::Float => {
                    let value = value.parse::<f64>();
                    if value.is_err() {
                        errors.report_error(
                            ParseError::new("failed to parse f64")
                                .with_context(Span::new_unchecked(0, raw.len()))
                                .with_unexpected(Span::new_unchecked(0, 2)),
                        );
                    }
                    info!("Float: {}", value.unwrap());
                }
                ScalarKind::Integer(radix) => {
                    let value = i64::from_str_radix(&value, radix.value());
                    if value.is_err() {
                        errors.report_error(
                            ParseError::new("failed to parse i64")
                                .with_context(Span::new_unchecked(0, raw.len()))
                                .with_unexpected(Span::new_unchecked(0, 2)),
                        );
                    }
                }
            }
        }
    }
}

///
pub fn parse_config(toml_str: &str) {
    let source = Source::new(toml_str);

    let tokens = source.lex().into_vec();
    let mut events = Vec::with_capacity(tokens.len());
    let mut errors = Vec::with_capacity(tokens.len());

    let mut receiver = ValidateWhitespace::new(&mut events, source);
    parse_document(&tokens, &mut receiver, &mut errors);

    info!("{:?}", events);

    validate(source, events, errors);
}*/
