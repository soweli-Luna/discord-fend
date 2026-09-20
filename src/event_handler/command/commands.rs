use documented::{DocumentedFieldsOpt, DocumentedVariantsOpt};

#[derive(Debug, Clone, DocumentedFieldsOpt, DocumentedVariantsOpt, PartialEq)]
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
    /// This command has a number of special features:
    ///
    /// - Interactive REPL sessions
    ///
    /// -# (type `~fend` with no arguments, and the bot will listen to every message you send for a while)
    ///
    /// - Multi-line prompts
    ///
    /// -# (any expression that spans multiple lines will be evaluated as if
    ///   it were a series of prompts, with context retention between prompts)
    ///
    /// - Channel context retention
    ///
    /// -# (try sending `~fend x = 5` and then `~fend x + 10`)
    ///
    /// ---
    ///
    /// this section should be ignored by the help command
    Fend,
    /// **clear-context**
    ///
    /// ---
    ///
    /// Clears the fend context for the current channel
    ///
    /// ---
    ///
    /// This is mostly included in case an important unit or constant gets redefined,
    /// which could remain in the context for a long time depending on usage conditions.
    ClearContext,
    /// **help** \[COMMANDS\]
    ///
    /// ---
    ///
    /// Lists all commands, or details about a specific command if one or more is provided
    Help,
    Uptime, // no docs means it will be hidden from the help command
    Version,
}
impl Parse for Command {
    fn try_parse(content: &mut Vec<String>) -> Option<Self> {
        if let Some(first) = content.first() {
            let command = match first.as_str() {
                "fend" => Self::Fend,
                "clear-context" => Self::ClearContext,
                "help" => Self::Help,
                "uptime" => Self::Uptime,
                "version" => Self::Version,
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
            Command::Fend => fend::cmd(usr, msg, args, None).await,
            Command::ClearContext => fend::clear_context(usr, msg).await,
            Command::Help => help::cmd(usr, msg, args).await,
            Command::Uptime => uptime::cmd(usr, msg, args).await,
            Command::Version => version::cmd(usr, msg).await,
        }
    }
}

pub trait Parse: Sized {
    /// Returns the parsed value and removes it from the vector
    fn try_parse(content: &mut Vec<String>) -> Option<Self>;
}

pub mod fend;
pub mod help;
pub mod uptime;
pub mod version;
