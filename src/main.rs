use std::env;

use serenity::{async_trait, Client};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id != 1121440749096009850 {
            let message: String = msg.content;
            let result: String;
            if let Err(why) = msg.channel_id.say(&ctx.http, result).await {
                println!("error sending message: {:?}", why);
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
fn main() {
    println!("Hello, world!");
}
