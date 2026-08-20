use core::time;
use std::{collections::HashMap, fmt::Display, str::FromStr, sync::LazyLock};

use serenity::all::{ChannelId, Message, UserId};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, Receiver, Sender},
    },
    time::timeout,
};

use crate::response_helper::ResponseHelper;

type InteractiveSender = mpsc::Sender<Message>;
type InteractiveMap = HashMap<(UserId, ChannelId), InteractiveSender>;
pub static INTERACTIVE_SESSION_MAP: LazyLock<RwLock<InteractiveMap>> =
    LazyLock::new(Default::default);

pub async fn try_find_session(msg: &Message) -> Option<InteractiveSender> {
    let session_map_guard = INTERACTIVE_SESSION_MAP.read().await;
    session_map_guard
        .get(&(msg.author.id, msg.channel_id))
        .cloned()
}

pub async fn handle(sender: Sender<Message>, msg: &Message) {
    if let Err(_err) = sender.send(msg.clone()).await {
        INTERACTIVE_SESSION_MAP
            .write()
            .await
            .remove(&(msg.author.id, msg.channel_id));
    }
}

pub struct InteractiveSession {
    receiver: Receiver<Message>,
    pub msg: Message,
}

impl InteractiveSession {
    pub async fn register(
        msg: Message,
        closure: impl AsyncFnOnce(&mut InteractiveSession),
    ) -> Option<()> {
        let mut session_map_guard = INTERACTIVE_SESSION_MAP.write().await;

        // Add hashmap entry if there is none, and spawn mpsc consumer thread
        // if there is already an entry, return an error, we dont want to overwrite it
        // or to have multiple sessions running at once for a single user+channel
        if let std::collections::hash_map::Entry::Vacant(entry) =
            session_map_guard.entry((msg.author.id, msg.channel_id))
        {
            let (sender, receiver) = mpsc::channel::<Message>(10);
            entry.insert(sender);

            let mut session = InteractiveSession {
                receiver,
                msg: msg.clone(),
            };
            drop(session_map_guard);

            closure(&mut session).await;

            let mut session_map_guard = INTERACTIVE_SESSION_MAP.write().await;
            session_map_guard.remove(&(msg.author.id, msg.channel_id));
        } else {
            return None;
        }

        Some(())
    }

    #[allow(dead_code)]
    pub async fn get_response(&mut self) -> Option<Message> {
        if let Ok(Some(msg)) = timeout(time::Duration::from_secs(60), self.receiver.recv()).await {
            return Some(msg);
        }
        None
    }

    #[expect(unused)]
    pub async fn parse_response<T>(&mut self, prompt: &str) -> Option<T>
    where
        T: InteractiveFill + Display,
    {
        if !prompt.is_empty() {
            ResponseHelper::new(&self.msg.author, &self.msg)
                .no_reply()
                .push(prompt)
                .say()
                .await;
        }
        loop {
            let result = T::interactive_fill(self).await;
            match result {
                Some(Ok(result)) => break Some(result),
                Some(Err(err)) => {
                    ResponseHelper::new(&self.msg.author, &self.msg)
                        .no_reply()
                        .push(err)
                        .say()
                        .await;
                }
                None => break None,
            }
        }
    }
}

/// A trait to simplify filling any type using an interactive session where
/// the user can be prompted for each unit of information one at a time.
pub trait InteractiveFill
where
    Self: Sized,
{
    async fn interactive_fill(session: &mut InteractiveSession) -> Option<Result<Self, String>>;
}

impl<T> InteractiveFill for T
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    async fn interactive_fill(session: &mut InteractiveSession) -> Option<Result<Self, String>> {
        if let Some(msg) = session.get_response().await {
            Some(msg.content.parse::<T>().map_err(|err| err.to_string()))
        } else {
            None
        }
    }
}
