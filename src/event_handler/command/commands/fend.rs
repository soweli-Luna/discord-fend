use std::{char, format, vec};

use fend_core::SpanRef;

use crate::{
    event_handler::interactive_session::InteractiveSession, response_helper::ResponseHelper,
    utils::ansi_color,
};

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, _args: Vec<String>) {
    // let args = args.join(" ").replace(['`'], " ");
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

    if lines.is_empty() {
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
                let mut bot_response = bot_response
                    .reply_to(user_response.clone())
                    .start_typing()
                    .await;

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
        let mut bot_response = ResponseHelper::new(usr, msg).start_typing().await;

        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            tokio::task::spawn_blocking(|| {
                let mut fend_context = fend_core::Context::new();
                fend_context.set_output_mode_terminal();
                fend_context.set_random_u32_fn(random_u32);

                fend_run(lines, &mut fend_context)
            }),
        )
        .await
        .unwrap_or_else(|_| Ok("Operation timed out.".to_string()))
        .unwrap_or_else(|err| format!("Error: {}", err));

        bot_response.push(response).say().await;
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
            result_buf.push_str(&format!("> {}\n", line));
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
