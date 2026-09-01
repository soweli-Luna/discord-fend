use std::{
    eprintln,
    sync::{LazyLock, OnceLock},
};

use circular_buffer::FixedCircularBuffer;
use serenity::{
    all::{Context, EventHandler, Message, MessageUpdateEvent, Ready},
    async_trait,
};
use tokio::sync::RwLock;

use crate::{debug, event_handler::command::commands};

mod command;
pub mod interactive_session;

pub static CTX: OnceLock<Context> = OnceLock::new();

const MAX_EDITABLE_COMMANDS: usize = 1024;
/// holds (user_message, bot_message) pairs for the last MAX_EDITABLE_COMMANDS commands that were sent by users
type EditableCommandsBuf = FixedCircularBuffer<(Message, Message), MAX_EDITABLE_COMMANDS>;
pub static EDITABLE_COMMANDS: LazyLock<RwLock<Box<EditableCommandsBuf>>> =
    LazyLock::new(|| RwLock::new(EditableCommandsBuf::boxed()));

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, _: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if let Some((command, args)) = command::try_find_command(&msg.content).await {
            command::handle(command, args, msg.clone(), msg.author).await;
            return;
        }
        if let Some(sender) = interactive_session::try_find_session(&msg).await {
            interactive_session::handle(sender, &msg).await;
            return;
        }
    }

    // Set a handler to be called on the `message_update` event. This is called when a message is
    // edited.
    //
    // We hook into this event to handle updates to editable commands
    async fn message_update(
        &self,
        ctx: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let msg_id = event.id;
        if let Some(user) = event.author {
            if user.bot {
                return;
            }
        } else {
            return;
        }
        if let Some(content) = event.content
            && let Some((command, args)) = command::try_find_command(&content).await
            && command == commands::Command::Fend
            && !args.is_empty()
        {
            let message_pair = {
                EDITABLE_COMMANDS
                    .read()
                    .await
                    .iter()
                    .find(|pair| pair.0.id == msg_id)
                    .cloned()
            };

            debug!("message update: {:?} -> {:?}", old, new);

            if let Some((user_msg, bot_msg)) = message_pair {
                match event.channel_id.message(ctx.http, user_msg.id).await {
                    Ok(new_msg) => {
                        commands::fend::cmd(&new_msg.author, &new_msg, args, Some(&bot_msg)).await;
                    }
                    Err(err) => debug!("Failed to update message: {}", err),
                }
            }

            // bring the message pair back to the front of the buffer
            // { ----------- nope this happens inside fend::cmd now
            //     let mut handle = EDITABLE_COMMANDS.write().await;
            //     if let Some(index) = handle.iter().position(|pair| pair.0.id == msg_id)
            //         && let Some(pair) = handle.remove(index)
            //     {
            //         handle.push_front(pair);
            //     }
            // }
        };
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);
        CTX.set(ctx)
            .unwrap_or_else(|_| eprintln!("Received additional `ready` event"))
    }
}
