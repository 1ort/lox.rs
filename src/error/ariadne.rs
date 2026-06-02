use std::ops::Range;

use crate::error::reporter::ErrorReporter;
use ariadne::{Color, ColorGenerator, Config, Fmt, Label, Report, ReportKind, Source};

pub struct AriadneReporter<'a> {
    source: &'a str,
    source_name: &'a str,
    color: Color,
}

impl<'a> AriadneReporter<'a> {
    pub fn new(source: &'a str, source_name: &'a str) -> Self {
        let mut colors = ColorGenerator::new();
        let color = colors.next();
        AriadneReporter {
            source,
            source_name,
            color,
        }
    }
}

impl<'a> ErrorReporter for AriadneReporter<'a> {
    fn report(&self, err: &super::error::LoxError) {
        let (error_kind, message, span) = match err {
            crate::error::error::LoxError::Syntax { message, token } => {
                ("Syntax error", message, token.span.clone())
            }
            crate::error::error::LoxError::Resolver { message, span } => {
                ("Resolver error", message, span.clone().unwrap_or_default())
            }
            crate::error::error::LoxError::Runtime { message, span } => {
                ("Runtime error", message, span.clone().unwrap_or_default())
            }
        };

        let range: Range<usize> = span.clone().into();

        Report::build(ReportKind::Error, (self.source_name, range.clone()))
            .with_message(error_kind)
            .with_label(
                Label::new((self.source_name, range.clone()))
                    .with_message(message)
                    .with_color(self.color),
            )
            .finish()
            .eprint((self.source_name, Source::from(self.source)));
    }
}

