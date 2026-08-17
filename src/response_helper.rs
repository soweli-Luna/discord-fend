use std::time::Duration;

use serenity::Result;
use tokio::time::sleep;

/// Helper struct for constructing multiple-message responses with convincing typing times
#[derive(Debug, Clone)]
pub struct ResponseHelper<'a> {
    response: Vec<String>,
    ctx: &'a serenity::all::Context,
    msg: &'a serenity::all::Message,
    usr: &'a serenity::all::User,
    reply_mode: ReplyMode,
}
impl<'a> ResponseHelper<'a> {
    #![expect(dead_code)]

    pub fn new(usr: &'a serenity::all::User, msg: &'a serenity::all::Message) -> Self {
        #[expect(clippy::expect_used)]
        Self {
            response: Vec::new(),
            ctx: crate::event_handler::CTX
                .get()
                .expect("CTX should be set before the api is used"),
            msg,
            usr,
            reply_mode: ReplyMode::Reply,
        }
    }

    pub fn dm(mut self) -> Self {
        self.reply_mode = ReplyMode::Dm;
        self
    }

    pub fn no_reply(mut self) -> Self {
        self.reply_mode = ReplyMode::NoReply;
        self
    }

    pub fn reply(mut self) -> Self {
        self.reply_mode = ReplyMode::Reply;
        self
    }

    pub fn ping_reply(mut self) -> Self {
        self.reply_mode = ReplyMode::PingReply;
        self
    }

    pub fn push<S>(&mut self, msg: S) -> &mut Self
    where
        S: Into<String>,
    {
        let response = msg.into();
        self.response.push(response);
        self
    }

    /// Send the message, failing silently
    pub async fn say(&mut self) {
        self.try_say().await.unwrap_or_else(|err| {
            eprintln!("Failed to send message: {:?}", err);
        });
    }

    /// Send the message, returning an error upon failure
    pub async fn try_say(&mut self) -> Result<()> {
        // sleep(Duration::from_millis(300)).await;
        let channel = match self.reply_mode {
            ReplyMode::Dm => self.usr.create_dm_channel(&self.ctx.http).await?.into(),
            _ => self.msg.channel_id,
        };
        for message in self.response.clone() {
            sleep(Duration::from_millis(300)).await;
            let typing = channel.start_typing(&self.ctx.http);

            sleep(Duration::from_millis((message.len().isqrt() * 200) as u64)).await;
            typing.stop();
            match self.reply_mode {
                ReplyMode::NoReply => channel.say(&self.ctx.http, message).await?,
                ReplyMode::Reply => self.msg.reply(&self.ctx.http, message).await?,
                ReplyMode::PingReply => self.msg.reply_ping(&self.ctx.http, message).await?,
                ReplyMode::Dm => channel.say(&self.ctx.http, message).await?,
            };
            // clear reply mode so it only applies to the first message
            if self.reply_mode != ReplyMode::Dm {
                self.reply_mode = ReplyMode::NoReply;
            }
        }
        Ok(())
    }

    /// Start typing, failing silently.
    ///
    /// Not typically needed, as [`say`](#method.say) will start typing automatically, but can be useful if you want to start typing
    /// before beginning a time-consuming operation in order to let the user know that something is happening.
    pub async fn start_typing(mut self) -> Self {
        match self.get_channel().await {
            Ok(channel) => {
                channel.start_typing(&self.ctx.http);
            }
            Err(err) => {
                eprintln!("Failed to start typing: {}", err);
            }
        };
        self
    }

    /// Start typing, returning an error upon failure.
    ///
    /// Not typically needed, as [`say`](#method.say) will start typing automatically, but can be useful if you want to start typing
    /// before beginning a time-consuming operation in order to let the user know that something is happening.
    pub async fn try_start_typing(mut self) -> Result<Self> {
        self.get_channel().await?.start_typing(&self.ctx.http);
        Ok(self)
    }

    async fn get_channel(&mut self) -> Result<serenity::all::ChannelId> {
        Ok(match self.reply_mode {
            ReplyMode::Dm => self.usr.create_dm_channel(&self.ctx.http).await?.into(),
            _ => self.msg.channel_id,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ReplyMode {
    NoReply,
    Reply,
    PingReply,
    Dm,
}
