use std::vec;

use fend_core::SpanRef;

use crate::{
    event_handler::interactive_session::InteractiveSession, response_helper::ResponseHelper,
    utils::ansi_color,
};

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, args: Vec<String>) {
    let args = args.join(" ").replace(&['\n', '\r', '`'][..], " ");

    if args.is_empty() {
        // no args, make an interactive session
        let closure = async |session: &mut InteractiveSession| {
            ResponseHelper::new(usr, msg)
                .push("Starting fend REPL. Type `exit` to exit.")
                .say()
                .await;

            loop {
                let expr = match session.get_response().await {
                    Some(expr) => expr,
                    None => {
                        ResponseHelper::new(usr, msg)
                            .push("Session timed out.")
                            .say()
                            .await;
                        break;
                    }
                };

                if expr.content == "exit" || expr.content == "quit" {
                    ResponseHelper::new(usr, msg)
                        .push("Fend exited.")
                        .say()
                        .await;
                    break;
                }

                // let response = fend_run(&expr.content, &mut fend_context);

                let response = tokio::task::spawn_blocking(move || {
                    let mut fend_context = fend_core::Context::new();
                    fend_context.set_output_mode_terminal();
                    fend_context.set_random_u32_fn(random_u32);

                    fend_run(&expr.content, &mut fend_context)
                })
                .await
                .unwrap_or_else(|err| format!("Error: {}", err));

                ResponseHelper::new(usr, msg)
                    .no_reply()
                    .push(response)
                    .say()
                    .await;
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
        let response = tokio::task::spawn_blocking(move || {
            let mut fend_context = fend_core::Context::new();
            fend_context.set_output_mode_terminal();
            fend_context.set_random_u32_fn(random_u32);

            fend_run(&args, &mut fend_context)
        })
        .await
        .unwrap_or_else(|err| format!("Error: {}", err));
        ResponseHelper::new(usr, msg)
            // .no_reply()
            .push(response)
            .say()
            .await;
    }
}

fn fend_run(input: &str, context: &mut fend_core::Context) -> String {
    let result = fend_core::evaluate(input, context);
    match result {
        Ok(result) => render_spans(result.get_main_result_spans()),
        Err(err) => format!(
            "```ansi\n{}\n```",
            ansi_color::format(&err, vec![ansi_color::Style::RedForeground])
        ),
    }
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

    format!("```ansi\n{}\n```", buf).to_string()
}

fn random_u32() -> u32 {
    rand::random()
}
