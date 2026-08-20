use documented::{DocumentedFieldsOpt, DocumentedVariantsOpt};

#[derive(Debug, Clone, DocumentedFieldsOpt, DocumentedVariantsOpt)]
// #[warn(clippy::missing_docs_in_private_items)]
pub enum Command {
    /// **fend** \[EXPRESSION\]
    ///
    /// ---
    ///
    /// Calculates a [fend](<https://github.com/printfn/fend>) expression,
    /// or starts an interactive fend REPL if no expression is provided
    ///
    /// ---
    ///
    /// [fend](<https://github.com/printfn/fend>) is an arbitrary-precision unit-aware calculator,
    /// see the [manual](<https://printfn.github.io/fend/documentation/>) for more information on how to use it
    ///
    /// ---
    ///
    /// this section should be ignored by the help command
    Fend,
    /// **help** \[COMMANDS\]
    ///
    /// ---
    ///
    /// Lists all commands, or details about a specific command if one or more is provided
    Help,
    /// **uptime**
    ///
    /// ---
    ///
    /// Displays the uptime of the bot
    Uptime,
}
impl Parse for Command {
    fn try_parse(content: &mut Vec<String>) -> Option<Self> {
        if let Some(first) = content.first() {
            let command = match first.as_str() {
                "fend" => Self::Fend,
                "help" => Self::Help,
                "uptime" => Self::Uptime,
                _ => return None,
            };
            content.remove(0);
            return Some(command);
        }
        None
    }
}

impl Command {
    pub async fn execute(
        &self,
        usr: &serenity::all::User,
        msg: &serenity::all::Message,
        args: Vec<String>,
    ) {
        match self {
            Command::Fend => fend::cmd(usr, msg, args).await,
            Command::Help => help::cmd(usr, msg, args).await,
            Command::Uptime => uptime::cmd(usr, msg, args).await,
        }
    }
}

pub trait Parse: Sized {
    /// Returns the parsed value and removes it from the vector
    fn try_parse(content: &mut Vec<String>) -> Option<Self>;
}

mod fend;
mod help;
mod uptime;
