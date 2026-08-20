use dotenv::dotenv;
use serenity::prelude::*;
use std::env;

mod event_handler;
mod response_helper;
mod utils;

#[tokio::main]
#[expect(clippy::expect_used, clippy::unwrap_used)]
async fn main() {
    // fetch env vars
    dotenv().ok();
    let token = env::var("TOKEN").expect("env variable `TOKEN` should be set");
    #[expect(unused_variables)]
    let application_id =
        env::var("APPLICATION_ID").expect("env variable `APPLICATION_ID` should be set");
    #[expect(unused_variables)]
    let public_key = env::var("PUBLIC_KEY").expect("env variable `PUBLIC_KEY` should be set");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        // | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(token.clone(), intents)
        .event_handler(event_handler::Handler)
        .await
        .unwrap();

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
