mod storage;
use teloxide::prelude::*;

pub mod schema;
mod order;


#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();



    log::info!("Starting...");

    let bot = Bot::new("5714835117:AAEuzNfq54AdZRYC1-GiuzJOkEAeYqFwFsA".to_string());
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // bot.send_dice(msg.chat.id).await?;
        // log::info!("{:?}",msg.text());
        println!("{:?}",msg.text());
        match msg.text() {
            Some(txt) => {
                if txt == "/start" {
                    bot.send_message(msg.chat.id, "Hello!").await?;
                }
            },
            None => {}
        }
        Ok(())
    }).await;
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        assert!(true);
        println!("test")
    }
}