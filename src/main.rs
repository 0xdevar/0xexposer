use std::fs;

use serenity::all::{ChannelId, MessageId, Channel, StageInstanceAction, MessagesIter};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};
use sqlx::query::QueryAs;

const CONFIG_FILE: &'static str = ".config.json";

struct Handler {
    database: sqlx::SqlitePool,
    config: Config 
}

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    channels: Vec<String>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.bot { return; }
        if !self.config.channels.contains(&msg.channel_id.get().to_string()) {return ;}

        sqlx::query("INSERT INTO messages (user_id, message_id, content) VALUES (?,?,?)")
        .bind(msg.author.id.to_string())
            .bind(msg.id.to_string()) // channel_id
            .bind(msg.content) // content
            .execute(&self.database)
            .await
            .expect("Message Save");
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, msg_id: MessageId, _guild_id: Option<serenity::all::GuildId>) {
        if !self.config.channels.contains(&channel_id.get().to_string()) {return ;}
        
        let query : Result<(String, String,), sqlx::Error> = sqlx::query_as("SELECT user_id, content from messages where message_id = ?")
            .bind(msg_id.get().to_string())
            .fetch_one(&self.database)
            .await;

        if let Err(err) = query { 
            println!("error {}", err);
            return
        }

        let message = query.unwrap();
        channel_id.say(&ctx.http, format!("<@{}> has send this : {}", message.0, message.1))
            .await
            .expect("He Not Send Message");
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Create Database
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true)
        )
        .await
        .expect("No Please Database");

    // Create Tables
    sqlx::query(
        r"create table if not exists messages (
          	message_id varchar(64),
          	user_id varchar(64),
            content text,
            PRIMARY KEY (message_id)
        )")
    .execute(&database)
    .await
    .expect("Database can't create table messages");

    // Create Config File
    let config_content = match fs::metadata(CONFIG_FILE) {
        Ok(_) => fs::read_to_string(CONFIG_FILE).expect("No File Here"),
        Err(_) => {
            let config = Config {
                token: "[Your Token]".to_string(),
                channels: vec![
                    "[Channel id]".to_string()
                ]
            };

            fs::write(CONFIG_FILE, serde_json::to_string::<Config>(&config)
                .expect("Error With Parse Config")) // end Parse Config
                .expect("[!] I can't create config file"); // end Write file

            panic!("[!] No Config Here | I Create One Just go change Token & Channel")
        }
    };

    // Read Config File
    let config: Config = serde_json::from_str(&config_content).expect("Can't parse json");

    // Create event handler
    let handler = Handler {
        database,
        config
    };

    let intents = GatewayIntents::all();

    let mut client =
        Client::builder(&handler.config.token, intents).event_handler(handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}