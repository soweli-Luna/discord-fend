use documented::{DocumentedFieldsOpt, DocumentedVariantsOpt};

use crate::response_helper::ResponseHelper;

use super::Parse;

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, args: Vec<String>) {
    let mut parsed_commands = Vec::new();
    let mut unparsed_args = Vec::new();
    for arg in &args {
        let arg = arg.clone();
        if let Some(command) = super::Command::try_parse(&mut vec![arg.clone()]) {
            parsed_commands.push(command);
        } else {
            unparsed_args.push(arg);
        }
    }

    let mut response = ResponseHelper::new(usr, msg);

    if !unparsed_args.is_empty() {
        if unparsed_args.len() == 1 {
            response.push(format!("Unknown command: {}", unparsed_args[0]));
        } else {
            response.push(format!("Unknown commands: {}", unparsed_args.join(", ")));
        }
    }

    if parsed_commands.is_empty() {
        let command_docs = super::Command::FIELD_DOCS;
        let command_docs_formatted = command_docs
            .iter()
            .filter_map(|doc| *doc)
            .map(doc_comment_format)
            .collect::<Vec<_>>()
            .join("");

        response.push(command_docs_formatted);
    } else {
        for command in parsed_commands {
            let command_doc = command.get_variant_docs();
            if let Some(doc) = command_doc {
                let command_doc_format = doc_comment_format_full(doc);
                response.push(command_doc_format);
            };
        }
    }

    response.say().await;
}

struct DocCommentParsed {
    usage: String,
    description: String,
    detail: String,
}

fn doc_comment_parse(doc: &str) -> DocCommentParsed {
    let doc = doc.replace("\n", " ").replace("\n\n", "\n");
    let mut sections = doc.split("---");

    DocCommentParsed {
        usage: sections.next().unwrap_or_default().trim().to_string(),
        description: sections.next().unwrap_or_default().trim().to_string(),
        detail: sections.next().unwrap_or_default().trim().to_string(),
    }
}

fn doc_comment_format(doc: &str) -> String {
    let doc = doc_comment_parse(doc);
    let mut formatted = String::new();

    if !doc.usage.is_empty() {
        formatted.push_str(&format!("{}\n", doc.usage));

        if !doc.description.is_empty() {
            formatted.push_str(&format!("-# {}\n", doc.description));
        }
    }
    formatted
}

fn doc_comment_format_full(doc: &str) -> String {
    let doc = doc_comment_parse(doc);
    let mut formatted = String::new();

    if !doc.usage.is_empty() {
        formatted.push_str(&format!("{}\n", doc.usage));

        if !doc.description.is_empty() {
            formatted.push_str(&format!("{}\n", doc.description));
        }
        if !doc.detail.is_empty() {
            formatted.push_str(&format!("{}\n", doc.detail));
        }
    }
    formatted
}
