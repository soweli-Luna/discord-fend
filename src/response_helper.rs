use std::time::Duration;

use serenity::{
    Error::Model,
    Result,
    all::{EditMessage, Message, ModelError},
};
use tokio::time::sleep;

use crate::debug;

/// Helper struct for constructing multiple-message responses with convincing typing times
#[derive(Debug)]
pub struct ResponseHelper<'a> {
    response: Vec<String>,
    ctx: &'a serenity::all::Context,
    msg: &'a serenity::all::Message,
    usr: &'a serenity::all::User,
    reply_mode: ReplyMode,
    typing: Option<serenity::all::Typing>,
    latest_message: Option<serenity::all::Message>,
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
            reply_mode: ReplyMode::Reply(msg.clone()),
            typing: None,
            latest_message: None,
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
        self.reply_mode = ReplyMode::Reply(self.msg.clone());
        self
    }

    pub fn ping_reply(mut self) -> Self {
        self.reply_mode = ReplyMode::PingReply(self.msg.clone());
        self
    }

    pub fn reply_to(mut self, msg: Message) -> Self {
        self.reply_mode = ReplyMode::Reply(msg);
        self
    }

    pub fn ping_reply_to(mut self, msg: Message) -> Self {
        self.reply_mode = ReplyMode::PingReply(msg);
        self
    }

    pub fn message_to_edit(mut self, msg: Option<Message>) -> Self {
        if let Some(msg) = msg {
            self.reply_mode = ReplyMode::Edit(msg)
        };
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
        match self.try_say().await {
            Ok(_) => {}
            Err(Model(ModelError::MessageTooLong(_))) => {
                self.response = vec!["Message too long to send.".to_string()];
                self.try_say().await.unwrap_or_else(|err| {
                    eprintln!("Failed to send message: {:?}", err);
                });
            }
            Err(err) => {
                eprintln!("Failed to send message: {:?}", err);
            }
        }
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
            self.start_typing().await;

            sleep(Duration::from_millis((message.len().isqrt() * 50) as u64)).await;
            match &self.reply_mode {
                ReplyMode::NoReply => {
                    self.latest_message = Some(channel.say(&self.ctx.http, message).await?);
                }
                ReplyMode::Reply(msg) => {
                    self.latest_message = Some(msg.reply(&self.ctx.http, message).await?);
                }
                ReplyMode::PingReply(msg) => {
                    self.latest_message = Some(msg.reply_ping(&self.ctx.http, message).await?);
                }
                ReplyMode::Edit(msg) => {
                    msg.clone()
                        .edit(&self.ctx.http, EditMessage::new().content(message))
                        .await?;
                    self.latest_message = Some(msg.clone());
                }
                ReplyMode::Dm => {
                    self.latest_message = Some(channel.say(&self.ctx.http, message).await?);
                }
            };
            // clear reply mode so it only applies to the first message
            if let ReplyMode::Dm = self.reply_mode {
                // Do nothing
            } else {
                self.reply_mode = ReplyMode::NoReply;
            }
        }
        self.stop_typing().await;

        self.response.clear();
        Ok(())
    }

    /// Start typing, failing silently.
    ///
    /// Not typically needed, as [`say`](#method.say) will start typing automatically, but can be useful if you want to start typing
    /// before beginning a time-consuming operation in order to let the user know that something is happening.
    pub async fn start_typing(&mut self) {
        match self.get_channel().await {
            Ok(channel) => {
                if let Some(typing) = self.typing.take() {
                    typing.stop();
                }
                self.typing = Some(channel.start_typing(&self.ctx.http));
            }
            Err(err) => {
                debug!("Failed to start typing: {}", err);
            }
        };
    }

    /// Stop typing, failing silently.
    pub async fn stop_typing(&mut self) {
        if let Some(typing) = self.typing.take() {
            typing.stop();
        }
    }

    pub async fn get_channel(&mut self) -> Result<serenity::all::ChannelId> {
        Ok(match self.reply_mode {
            ReplyMode::Dm => self.usr.create_dm_channel(&self.ctx.http).await?.into(),
            _ => self.msg.channel_id,
        })
    }

    pub fn latest_message(&self) -> Option<&serenity::all::Message> {
        self.latest_message.as_ref()
    }
}

#[derive(Debug, Clone)]
enum ReplyMode {
    NoReply,
    Reply(Message),
    PingReply(Message),
    Edit(Message),
    Dm,
}

impl Drop for ResponseHelper<'_> {
    fn drop(&mut self) {
        if let Some(typing) = self.typing.take() {
            typing.stop();
        }
    }
}
