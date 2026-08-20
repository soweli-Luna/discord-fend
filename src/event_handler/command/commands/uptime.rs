use std::eprintln;

use crate::response_helper::ResponseHelper;

pub async fn cmd(usr: &serenity::all::User, msg: &serenity::all::Message, args: Vec<String>) {
    let start_time = crate::START_TIME.get();
    if let Some(start_time) = start_time {
        let now = time::OffsetDateTime::now_utc();
        let uptime = now - *start_time;

        let days = uptime.whole_days();
        let hours = uptime.whole_hours() % 24;
        let minutes = uptime.whole_minutes() % 60;
        let seconds = uptime.whole_seconds() % 60;

        let uptime_str = format!("{}d {}h {}m {}s", days, hours, minutes, seconds);

        ResponseHelper::new(usr, msg).push(uptime_str).say().await;
    } else {
        crate::debug!("Start time not set.");
        ResponseHelper::new(usr, msg)
            .push("Error: Start time not set.")
            .say()
            .await;
    }
}
