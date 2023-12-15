use std::fs;

use serenity::all::{ChannelId, MessageId, Channel, StageInstanceAction};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};

struct Handler {
    database: sqlx::SqlitePool,
    config: Config 
}

#[derive(Serialize, Deserialize)]
struct Config {
    channels: Vec<String>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{:?}", msg);

        if msg.author.bot { return; }















        

        sqlx::query("INSERT INTO messages (user_id, channel_id, guild_id, content) (? ? ? ? ?)")
            .bind(msg.author.id.get() as i64) // user_id
            .bind(msg.channel_id.get() as i64) // channel_id
            .bind(msg.guild_id.unwrap_or_default().get() as i64) // guild_id
            .bind(msg.content) // content
            .execute(&self.database)
            .await
            .expect("Message Save");
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, msg_id: MessageId, guild_id: Option<serenity::all::GuildId>) {

    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    //let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let token = "MTE2NTY3ODIzMjM3NzQ5OTczOQ.G_vwcP.B1wbG1MXPq6EGoPoAm8BYpnEwaHz79cMy2H5Ws";

    // Create Database
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true)
        )
        .await
        .expect("No Please Database");

    let config_content = match fs::metadata("config.json") {
        Ok(_) => fs::read_to_string("config.json").expect("No File Here"),
        Err(_) => panic!("[!] No Config Here")
    };

    let config: Config = serde_json::from_str(&config_content).expect("Can't parse json");

    // Create event handler
    let handler = Handler {
        database,
        config
    };

    let intents = GatewayIntents::all();

    let mut client =
        Client::builder(&token, intents).event_handler(handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}