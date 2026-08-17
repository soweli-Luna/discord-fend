use std::{collections::HashMap, sync::LazyLock, time::Duration};

use serenity::all::{ChannelId, Message};
use tokio::{
    sync::{RwLock, mpsc},
    time::timeout,
};

use crate::event_handler::command::commands::{Command, Parse};

mod commands;

#[derive(Clone)]
struct CommandInfo {
    command: Command,
    args: Vec<String>,
    msg: Message,
}

type CommandInfoSender = mpsc::Sender<CommandInfo>;
type ChannelMap = HashMap<ChannelId, CommandInfoSender>;
static CHANNEL_THREADS_MAP: LazyLock<RwLock<ChannelMap>> = LazyLock::new(Default::default);

const COMMAND_PREFIX: &str = "~";

/// Tries to find a command from a message, taking the prefix into account.
/// Checks the user profile to see what prefix types they use.
/// Returns `None` if no command was found, otherwise returns `(command, args)`
pub async fn try_find_command(msg: &serenity::all::Message) -> Option<(Command, Vec<String>)> {
    // !!! this function runs for EVERY message received, so it should finish as quickly as possible !!!

    // for now we'll only listen to `!`
    if let Some(content) = msg.content.strip_prefix(COMMAND_PREFIX) {
        // only if prefix is not followed by whitespace
        if content.strip_prefix(char::is_whitespace).is_none() {
            let mut args: Vec<String> = content.split_whitespace().map(String::from).collect();
            if let Some(command) = Command::try_parse(&mut args) {
                return Some((command, args));
            }
        }
    }

    None
}

pub async fn handle(
    command: Command,
    args: Vec<String>,
    msg: serenity::all::Message,
    usr: serenity::all::User,
) {
    let mut threads_map_guard = CHANNEL_THREADS_MAP.write().await;

    // Add hashmap entry if there is none, and spawn mpsc consumer thread
    if let std::collections::hash_map::Entry::Vacant(entry) =
        threads_map_guard.entry(msg.channel_id)
    {
        let (sender, receiver) = mpsc::channel::<CommandInfo>(10);
        entry.insert(sender);

        // Command handler (mpsc consumer)

        tokio::spawn(async move {
            let channel_id = msg.channel_id;
            let mut receiver = receiver;
            while let Ok(Some(cmd_info)) = timeout(Duration::from_secs(30), receiver.recv()).await {
                let usr = usr.clone();
                tokio::spawn(async move {
                    cmd_info
                        .command
                        .execute(&usr, &cmd_info.msg, cmd_info.args)
                        .await;
                });
            }

            CHANNEL_THREADS_MAP.write().await.remove(&channel_id);
        });
    }
    // Send command
    #[expect(unused_must_use, clippy::expect_used)]
    threads_map_guard
        .get(&msg.channel_id)
        .expect("Should not have experienced a cosmic bit flip")
        .send(CommandInfo { command, args, msg })
        .await
        .map_err(|err| eprintln!("Error sending command to thread: {}", err));
}
