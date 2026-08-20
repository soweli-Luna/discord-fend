use std::str::FromStr;

use serenity::all::UserId;

/// Prints to the standard error, with a newline, and only if `crate::DEBUG == true`.
///
/// Attempts to intelligently select between `eprintln!` behavior and `dbg!` behavior.
///
/// Behaves like `dbg!` when empty or passed an expression,
/// and like `eprintln!` when passed a format string.
///
/// When an expression is passed, like `dbg!`, returns the evaluated expression unchanged.
///
/// `pub crate::DEBUG: bool` must exist.
///
/// See [the formatting documentation in `std::fmt`](../std/fmt/index.html)
/// for details of the macro argument syntax.
///
/// # Panics
///
/// Panics if writing to `io::stderr` fails.
///
/// Writing to non-blocking stderr can cause an error, which will lead
/// this macro to panic.
///
/// # Examples
///
/// ```
/// debug!("Reached Progress Point");   // 'Reached Progress Point'
/// debug!(x)   // '[src/main.rs:2:5] x = 10'
/// debug!()    // '[src/main.rs:3:5]'
/// debug!("Format example. {}", x);   // 'Format example. 10'
/// ```
///
#[macro_export]
#[expect(clippy::crate_in_macro_def)]
macro_rules! debug {
    () => {
        if crate::DEBUG {
            eprint!("{} ", crate::utils::timestamp());
            dbg!();
        }
    };
    ($arg:literal) => {
        if crate::DEBUG {
            crate::print_debug_header!();
            eprintln!($arg);
        }
    };
    ($arg:expr) => {
        match $arg {
            tmp => {
                if crate::DEBUG {
                    eprint!("{} ", crate::utils::timestamp());
                    dbg!($arg);
                }
                tmp
            }
        }
    };
    ($($args:expr),+) => {
        if crate::DEBUG {
            crate::print_debug_header!();
            eprintln!($($args),+);
        }
    };
}

#[macro_export]
macro_rules! print_debug_header {
    () => {
        eprint!("{} ({}:{}): ", $crate::utils::timestamp(), file!(), line!());
    };
}

pub fn timestamp() -> String {
    let now = time::Timestamp::now();
    format!(
        "[{}]",
        now.format(time::macros::format_description!(
            "[hour]:[minute]:[second]:[subsecond digits:3]"
        ))
        .unwrap()
    )
    .to_string()
}

/// Helper module for ansi color codes, as implemented by discord's ansi code block highlighter
pub mod ansi_color {

    #[expect(unused)]
    #[non_exhaustive]
    pub enum Style {
        Reset,
        Bold,
        Underline,
        GreyForeground,
        RedForeground,
        GreenForeground,
        YellowForeground,
        BlueForeground,
        MagentaForeground,
        CyanForeground,
        WhiteForeground,
        FireflyDarkBlueBackground,
        OrangeBackground,
        MarbleBlueBackground,
        TurquoiseGreyBackground,
        GreyBackground,
        IndigoBackground,
        LightGreyBackground,
        WhiteBackground,
    }
    impl Style {
        fn code(&self) -> &'static str {
            match self {
                Style::Reset => "0",
                Style::Bold => "1",
                Style::Underline => "4",
                Style::GreyForeground => "30",
                Style::RedForeground => "31",
                Style::GreenForeground => "32",
                Style::YellowForeground => "33",
                Style::BlueForeground => "34",
                Style::MagentaForeground => "35",
                Style::CyanForeground => "36",
                Style::WhiteForeground => "37",
                Style::FireflyDarkBlueBackground => "40",
                Style::OrangeBackground => "41",
                Style::MarbleBlueBackground => "42",
                Style::TurquoiseGreyBackground => "43",
                Style::GreyBackground => "44",
                Style::IndigoBackground => "45",
                Style::LightGreyBackground => "46",
                Style::WhiteBackground => "47",
            }
        }
    }

    pub fn escape_sequence(parameters: Vec<Style>) -> String {
        format!(
            "\x1b[{}m",
            parameters
                .iter()
                .map(Style::code)
                .collect::<Vec<_>>()
                .join(";")
        )
        .to_string()
    }

    pub fn format(text: &str, parameters: Vec<Style>) -> String {
        format!(
            "{}{}{}",
            escape_sequence(parameters),
            text,
            escape_sequence(vec![Style::Reset])
        )
        .to_string()
    }
}

#[expect(unused)]
pub fn parse_mention(msg: String) -> Option<UserId> {
    let digits_str = msg.get(2..msg.len() - 1)?;

    let id = UserId::from_str(digits_str).ok()?;

    Some(id)
}
