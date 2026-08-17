use std::str::FromStr;

use serenity::all::UserId;

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

pub fn parse_mention(msg: String) -> Option<UserId> {
    let digits_str = msg.get(2..msg.len() - 1)?;

    let id = UserId::from_str(digits_str).ok()?;

    Some(id)
}
