use std::collections::{HashMap, VecDeque};
use std::env;

use chatgpt::client::ChatGPT;
use chatgpt::config::{ChatGPTEngine, ModelConfigurationBuilder};
use chatgpt::types::CompletionResponse;
use serenity::{async_trait, Client};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;
use textwrap_macros::dedent;

#[group]
struct General;

struct Handler {
    chatgpt_client: ChatGPT,
    chat_log: RwLock<HashMap<UserId, VecDeque<(String, String)>>>,
}

impl Handler {
    pub fn new() -> Self {
        let api_token = env::var("OPENAI_SK").expect("token not found");
        let client = ChatGPT::new_with_config(
            api_token,
            ModelConfigurationBuilder::default()
                .engine(ChatGPTEngine::Gpt35Turbo_0301)
                .max_tokens(2000)
                .temperature(1.0)
                .presence_penalty(0.6)
                .frequency_penalty(0.7)
                .build()
                .unwrap()
        ).unwrap();

        Self {
            chatgpt_client: client,
            chat_log: Default::default(),
        }
    }
}

const DESCRIPTION: &'static str = dedent!("\
    「くれちき」は、とても明るくてかわいい女の子です。
    たまに奇想天外な返事をすることがありますが、嘘をつかず、面白い話をしてくれます。
    これは、そんな「くれちき」と、人間の会話です。

    -----
");

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id != 1210917178455498792 || msg.author.bot || msg.content.starts_with("!") {
            return;
        }

        let logs = {
            let lock = self.chat_log.read().await;

            match lock.get(&msg.author.id) {
                Some(logs) => logs
                    .iter()
                    .map(|(user_text, ai_text)| format!("{}: {}\nくれちき: {}\n", msg.author.name, user_text, ai_text))
                    .collect::<Box<[_]>>()
                    .join(""),
                None => {
                    drop(lock);
                    self.chat_log
                        .write()
                        .await
                        .insert(msg.author.id, Default::default());
                    DESCRIPTION.into()
                }
            }
        };

        let typing = msg.channel_id.start_typing(&ctx.http);

        let prompt = format!("{logs}You: {}\nくれちき: ", msg.content);
        println!("{prompt}");

        let response: CompletionResponse = self
            .chatgpt_client
            .send_message(prompt)
            .await
            .unwrap();

        if let Err(why) = msg.reply(&ctx.http, response.message().clone().content).await {
            println!("error sending message: {:?}", why);
        }

        {
            let mut logs = self.chat_log.write().await;
            let logs = logs.entry(msg.author.id).or_default();

            logs.push_back((msg.content.clone(), response.message().clone().content));
            // logs.drain(..logs.len().saturating_sub(4));
            println!("{:?}: {:#?}", msg.author.name, logs);
        }

        let _ = typing.stop();
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP);

    //login with a bot token
    let token = dotenv::var("DISCORD_BOT_TOKEN").unwrap();
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler::new())
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}
