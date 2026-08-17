use std::sync::OnceLock;

use serenity::{
    all::{Context, EventHandler, Message, Ready},
    async_trait,
};

mod command;
pub mod interactive_session;

pub static CTX: OnceLock<Context> = OnceLock::new();

pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, _: Context, msg: Message) {
        if let Some((command, args)) = command::try_find_command(&msg).await {
            if msg.author.bot {
                return;
            }
            command::handle(command, args, msg.clone(), msg.author).await;
            return;
        }
        if let Some(sender) = interactive_session::try_find_session(&msg).await {
            interactive_session::handle(sender, &msg).await;
            return;
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        eprintln!("{} is connected!", ready.user.name);
        #[expect(clippy::expect_used)]
        CTX.set(ctx).expect("CTX should only be set once");
    }
}
