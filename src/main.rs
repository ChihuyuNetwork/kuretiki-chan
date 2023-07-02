use std::ops::Deref;
use std::sync::Arc;

use openai_api_rust::{ApiResult, Auth, OpenAI, openai, Role};
use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::completions::{Completion, CompletionsApi, CompletionsBody};
use openai_api_rust::Role::User;
use serenity::{async_trait, Client};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::StandardFramework;
use serenity::futures::TryFutureExt;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio;

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id == 1121440749096009850 && !msg.author.bot {
            let auth = Auth::from_env().unwrap();
            let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
            let body = ChatBody {
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: None,
                temperature: Some(0_f32),
                top_p: Some(0_f32),
                n: Some(2),
                stream: Some(false),
                stop: None,
                presence_penalty: None,
                frequency_penalty: None,
                logit_bias: None,
                user: None,
                messages: vec![openai_api_rust::Message { role: User, content: msg.content.to_string() }],
            };
            let rs = openai.chat_completion_create(&body);
            let choices = rs.unwrap().choices;
            let message = &choices[0].message.as_ref().unwrap().content;
            let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
            if let Err(why) = msg.reply(&ctx.http, message).await {
                println!("error sending message: {:?}", why);
            }
            typing.stop();
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let freamwork = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    //login with a bot token
    let token = dotenv::var("DISCORD_BOT_TOKEN").unwrap();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(freamwork)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start()
        .await{
        println!("An error occurred while running the client: {:?}", why)
    }
}
