use std::fs;

use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildChannel, MessageId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const CONFIG_FILE: &'static str = ".config.json";

struct Handler {
	database: sqlx::SqlitePool,
	config: Config,
}

#[derive(Serialize, Deserialize)]
struct Config {
	token: String,
	channels: Vec<String>,
}

impl Handler {
	fn is_target_channel(&self, channel: GuildChannel) -> bool {
		for c in &self.config.channels {
			if *c == channel.id.to_string() {
				return true;
			}

			let matched = match channel.parent_id {
				Some(id) => id.to_string() == *c,
				None => false,
			};

			if matched {
				return true;
			}
		}

		return false;
	}
}

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.author.bot {
			return;
		}

		let channel: Option<GuildChannel> = msg.channel(ctx.http).await.unwrap().guild();

		let channel = match channel {
			Some(channel) => channel,
			None => return
		};

		if !self.is_target_channel(channel) {
			println!("insert: message sent in [{}] which is not included in config", msg.channel_id.get());
			return;
		}

		sqlx::query("INSERT INTO messages (user_id, message_id, content) VALUES (?,?,?)")
			.bind(msg.author.id.to_string())
			.bind(msg.id.to_string())
			.bind(msg.content)
			.execute(&self.database)
			.await
			.ok();
	}

	async fn message_delete(&self, ctx: Context, channel_id: ChannelId, msg_id: MessageId, _guild_id: Option<serenity::all::GuildId>) {
		let channel: Option<GuildChannel> = channel_id.to_channel(&ctx.http).await.unwrap().guild();

		let channel = match channel {
			Some(channel) => channel,
			None => return
		};

		if !self.is_target_channel(channel) {
			println!("delete: message sent in [{}] which is not included in config", channel_id);
			return;
		}

		let query: Result<(String, String), sqlx::Error> = sqlx::query_as("SELECT user_id, content from messages where message_id = ?")
			.bind(msg_id.get().to_string())
			.fetch_one(&self.database)
			.await;

		if let Err(err) = query {
			println!("error {}", err);
			return;
		}

		let message = query.unwrap();
		channel_id
			.say(
				&ctx.http,
				format!(
					r#"
				[  🚫 <@{}>  ]: {}
		"#,
					message.0, message.1
				),
			)
			.await
			.ok();
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	// Create Database
	let database = sqlx::sqlite::SqlitePoolOptions::new()
		.connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename("database.sqlite").create_if_missing(true))
		.await
		.expect("No Please Database");

	// Create Tables
	sqlx::query(
		r"create table if not exists messages (
          	message_id varchar(64),
          	user_id varchar(64),
            content text,
            PRIMARY KEY (message_id)
        )",
	)
	.execute(&database)
	.await
	.expect("Database can't create table messages");

	// Create Config File
	let config_content = match fs::metadata(CONFIG_FILE) {
		Ok(_) => fs::read_to_string(CONFIG_FILE).expect("No File Here"),
		Err(_) => {
			let config = Config {
				token: "[Your Token]".to_string(),
				channels: vec!["[Channel id]".to_string()],
			};

			fs::write(CONFIG_FILE, serde_json::to_string::<Config>(&config).expect("Error With Parse Config")) // end Parse Config
				.expect("[!] I can't create config file"); // end Write file

			panic!("[!] No Config Here | I Create One Just go change Token & Channel")
		}
	};

	// Read Config File
	let config: Config = serde_json::from_str(&config_content).expect("Can't parse json");

	// Create event handler
	let handler = Handler { database, config };

	let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::non_privileged();

	let token: String = std::env::var("DISCORD_TOKEN").unwrap_or(handler.config.token.clone());

	let mut client = Client::builder(&token, intents).event_handler(handler).await.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
}
