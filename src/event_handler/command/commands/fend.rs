use std::{char, format, sync::LazyLock, vec};

use circular_buffer::FixedCircularBuffer;
use fend_core::SpanRef;
use serenity::all::ChannelId;
use tokio::sync::RwLock;

use crate::{
    event_handler::{EDITABLE_COMMANDS, interactive_session::InteractiveSession},
    response_helper::ResponseHelper,
    utils::ansi_color,
};

/// The maximum number of fend contexts to keep in memory
const MAX_FEND_CTX: usize = 128;
/// A buffer of fend contexts associated to channels, so that users can continue their calculations
type FendCtxBuf = FixedCircularBuffer<(ChannelId, fend_core::Context), MAX_FEND_CTX>;
/// A buffer of fend contexts associated to channels, so that users can continue their calculations
static FEND_CTX_BUF: LazyLock<RwLock<FendCtxBuf>> = LazyLock::new(Default::default);

pub async fn cmd(
    usr: &serenity::all::User,
    msg: &serenity::all::Message,
    _args: Vec<String>,
    message_to_edit: Option<&serenity::all::Message>,
) {
    let args = msg
        .content
        .strip_prefix("~fend")
        .unwrap_or("")
        .trim()
        .replace(['`'], " ");

    let lines = args
        .lines()
        .map(String::from)
        .filter(|l| !l.trim_matches(char::is_whitespace).is_empty())
        .collect::<Vec<_>>();

    if lines.is_empty() && message_to_edit.is_none() {
        // no args, make an interactive session
        let closure = async |session: &mut InteractiveSession| {
            let mut bot_response = ResponseHelper::new(usr, msg);

            bot_response
                .push("Starting fend REPL. Type `exit` to exit.")
                .say()
                .await;

            let mut fend_context = fend_core::Context::new();
            fend_context.set_output_mode_terminal();
            fend_context.set_random_u32_fn(random_u32);

            loop {
                let mut bot_response = ResponseHelper::new(usr, msg);
                let user_response = match session.get_response().await {
                    Some(expr) => expr,
                    None => {
                        bot_response.push("Session timed out.").say().await;
                        break;
                    }
                };

                if user_response.content == "exit" || user_response.content == "quit" {
                    bot_response.push("Fend exited.").say().await;
                    break;
                }

                // set to reply to the latest message,
                // and start typing right away so the user knows we're working on it
                let mut bot_response = bot_response.reply_to(user_response.clone());
                bot_response.start_typing().await;

                let mut passed_fend_context = fend_context.clone();
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    tokio::task::spawn_blocking(move || {
                        let lines = user_response
                            .content
                            .replace(['`'], " ")
                            .lines()
                            .map(String::from)
                            .collect::<Vec<_>>();

                        (
                            fend_run(lines, &mut passed_fend_context),
                            passed_fend_context,
                        )
                    }),
                )
                .await;

                let (response, returned_fend_context) = match result {
                    Ok(Ok((response, fend_context))) => (response, fend_context),
                    Ok(Err(err)) => {
                        bot_response.push(format!("Error: {}", err)).say().await;
                        continue;
                    }
                    Err(_) => {
                        bot_response.push("Operation timed out.").say().await;
                        break;
                    }
                };

                fend_context = returned_fend_context;

                bot_response.push(response).say().await;
            }
        };

        if (InteractiveSession::register(msg.clone(), closure).await).is_none() {
            ResponseHelper::new(usr, msg)
                .push("You already have an active interactive session.")
                .say()
                .await;
        }
    } else {
        // args given, just evaluate the expression

        // start typing right away so the user knows we're working on it
        let mut bot_response = ResponseHelper::new(usr, msg);
        bot_response.start_typing().await;

        if let Some(message_to_edit) = message_to_edit {
            bot_response = bot_response.message_to_edit(Some(message_to_edit.clone()));
        }

        let handle = FEND_CTX_BUF.read().await;
        let ctx_entry = handle
            .iter()
            .find(|(channel_id, _ctx)| *channel_id == msg.channel_id)
            .cloned();
        drop(handle);

        // get the context for this channel, or create a new one if it doesn't exist
        let mut fend_context = match ctx_entry {
            Some((_, ctx)) => ctx.clone(),
            None => {
                let mut fend_context = fend_core::Context::new();
                fend_context.set_output_mode_terminal();
                fend_context.set_random_u32_fn(random_u32);
                fend_context
            }
        };

        let (response, fend_context) = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            tokio::task::spawn_blocking(move || {
                (fend_run(lines, &mut fend_context), Some(fend_context))
            }),
        )
        .await
        .unwrap_or_else(|_| Ok(("Operation timed out.".to_string(), None)))
        .unwrap_or_else(|err| (format!("Error: {}", err), None));

        // store the updated context in the buffer
        let mut handle = FEND_CTX_BUF.write().await;

        // we need to re-index in case another thread has pushed to the buffer
        let ctx_entry_idx = handle
            .iter()
            .position(|(channel_id, _ctx)| *channel_id == msg.channel_id);

        // only update the context if we actually got a new one
        if let Some(fend_context) = fend_context {
            if let Some(idx) = ctx_entry_idx {
                match handle.nth_front_mut(idx) {
                    Some((_, ctx)) => *ctx = fend_context,
                    None => {
                        // this shouldnt be possible as the buffer never shrinks,
                        // but just in case, we handle it by pushing the context to the front of the buffer
                        handle.push_front((msg.channel_id, fend_context));
                    }
                }
            } else {
                // this would indicate that the entry fell out of the buffer while we were processing,
                // which is unlikely but possible
                handle.push_front((msg.channel_id, fend_context));
            }
        }

        drop(handle);

        bot_response.push(response).say().await;

        if let Some(my_msg) = bot_response.latest_message() {
            let mut handle = EDITABLE_COMMANDS.write().await;
            if let Some(index) = handle.iter().position(|pair| pair.1.id == my_msg.id) {
                handle.remove(index);
            }
            handle.push_front((msg.clone(), my_msg.clone()));
        }
    }
}

/// Run a series of lines in the given context, returning the output as a string
///
/// Will block
fn fend_run(lines: Vec<String>, context: &mut fend_core::Context) -> String {
    let mut result_buf = String::new();

    let multiline = lines.len() > 1;

    for line in lines {
        if multiline {
            result_buf.push_str(&format!("> {}\n", line.trim()));
        }

        let result = match fend_core::evaluate(&line, context) {
            Ok(result) => render_spans(result.get_main_result_spans()),
            Err(err) => ansi_color::format(&err, vec![ansi_color::Style::RedForeground]),
        };

        result_buf.push_str(&format!("{result}\n"));
    }

    format!("```ansi\n{}\n```", result_buf)
}

fn render_spans<'a, T: Iterator<Item = SpanRef<'a>>>(spans: T) -> String {
    let mut buf = String::new();
    for span in spans {
        let style = match span.kind() {
            fend_core::SpanKind::Number => {
                vec![ansi_color::Style::WhiteForeground, ansi_color::Style::Bold]
            }
            fend_core::SpanKind::BuiltInFunction => vec![ansi_color::Style::YellowForeground],
            fend_core::SpanKind::Keyword => vec![ansi_color::Style::MagentaForeground],
            fend_core::SpanKind::String => {
                vec![ansi_color::Style::GreenForeground, ansi_color::Style::Bold]
            }
            fend_core::SpanKind::Date => {
                vec![ansi_color::Style::CyanForeground, ansi_color::Style::Bold]
            }
            fend_core::SpanKind::Whitespace => vec![ansi_color::Style::WhiteForeground],
            fend_core::SpanKind::Ident => vec![ansi_color::Style::BlueForeground],
            fend_core::SpanKind::Boolean => {
                vec![ansi_color::Style::RedForeground, ansi_color::Style::Bold]
            }
            _ => vec![ansi_color::Style::WhiteForeground],
        };

        buf.push_str(&ansi_color::format(span.string(), style));
    }

    // format!("```ansi\n{}\n```", buf).to_string()
    buf
}

fn random_u32() -> u32 {
    rand::random()
}

pub async fn clear_context(usr: &serenity::all::User, msg: &serenity::all::Message) {
    let mut handle = FEND_CTX_BUF.write().await;
    let ctx_entry_idx = handle
        .iter()
        .position(|(channel_id, _ctx)| *channel_id == msg.channel_id);

    if let Some(idx) = ctx_entry_idx {
        handle.remove(idx);
        let mut bot_response = ResponseHelper::new(usr, msg);
        bot_response.push("Context cleared").say().await;
    } else {
        let mut bot_response = ResponseHelper::new(usr, msg);
        bot_response.push("No context to clear").say().await;
    }
}
