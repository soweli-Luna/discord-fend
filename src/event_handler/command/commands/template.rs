use crate::response_helper::ResponseHelper;

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, args: Vec<String>) {
    ResponseHelper::new(usr, msg).push("Hello!").say().await;
}
