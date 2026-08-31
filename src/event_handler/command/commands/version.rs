use crate::response_helper::ResponseHelper;

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message) {
    ResponseHelper::new(usr, msg)
        .push(format!(
            "`{} {}`",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .say()
        .await;
}
