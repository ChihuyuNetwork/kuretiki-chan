use std::collections::{HashMap, VecDeque};
use std::env;

use openai_api::api::{CompletionArgs, CompletionArgsBuilder, Engine};
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
    openai_client: openai_api::Client,
    args_builder: CompletionArgsBuilder,
    chat_log: RwLock<HashMap<UserId, VecDeque<(String, String)>>>,
}

impl Handler {
    pub fn new() -> Self {
        let api_token = env::var("OPENAI_SK").expect("token not found");
        let client = openai_api::Client::new(&api_token);

        let mut builder = CompletionArgs::builder();
        builder
            .engine(Engine::Davinci)
            .max_tokens(100)
            .temperature(0.0)
            .top_p(1.0)
            .stop(vec!["You:".into()])
            .presence_penalty(0.6)
            .frequency_penalty(0.7);

        Self {
            openai_client: client,
            args_builder: builder,
            chat_log: Default::default(),
        }
    }
}

const DESCRIPTION: &'static str = dedent!("\
    「くれちき」は、とても明るい人です。
    嘘をつかず、面白い話をしてくれます。
    これは、そんな「くれちき」と、人間の会話です。

    -----
");

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id != 1121440749096009850 || msg.author.bot {
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
                    "".into()
                }
            }
        };

        let typing = msg.channel_id.start_typing(&ctx.http).unwrap();

        let prompt = format!("{DESCRIPTION}{logs}You: {}\nくれちき: ", msg.content);
        println!("{prompt}");
        let args = self
            .args_builder
            .clone()
            .prompt(prompt)
            .build()
            .expect("failed to build completion args ::::((((:(");

        let completion = self
            .openai_client
            .complete_prompt_sync(args)
            .expect("copmpletion falied");

        if let Err(why) = msg.reply(&ctx.http, &completion.choices[0].text).await {
            println!("error sending message: {:?}", why);
        }

        {
            let mut logs = self.chat_log.write().await;
            let logs = logs.entry(msg.author.id).or_default();

            logs.push_back((msg.content.clone(), completion.choices[0].text.clone()));
            // logs.drain(..logs.len().saturating_sub(4));
            println!("{:?}: {:#?}", msg.author, logs);
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
        .configure(|c| c.prefix("~"))
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
