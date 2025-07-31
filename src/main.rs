use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{interval, Duration};

struct Handler {
    // shared flag: true = stop requested
    stop: Arc<AtomicBool>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        match msg.content.as_str() {
            "!ping" => {
                if let Err(e) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error: {e:?}");
                }
            }

            "!count" => {
                // reset flag in case a previous run left it set
                self.stop.store(false, Ordering::Relaxed);

                let http = ctx.http.clone();
                let channel = msg.channel_id;
                let stop = self.stop.clone();

                tokio::spawn(async move {
                    let mut n = 0;
                    let mut ticker = interval(Duration::from_secs(1));
                    loop {
                        n+=1;
                        if stop.load(Ordering::Relaxed) {
                            let _ = channel.say(&http, "Counting stopped.").await;
                             if let Err(e) = channel.say(&http, n.to_string()).await {
                            println!("Error: {e:?}");
                            break;
                        }
                        break;
                        }
                        ticker.tick().await;
                        if let Err(e) = channel.say(&http, n.to_string()).await {
                            println!("Error: {e:?}");
                            break;
                        }
                    }

                });
            }

            "!stop" => {
                self.stop.store(true, Ordering::Relaxed);
            }

            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("token missing");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = Handler {
        stop: Arc::new(AtomicBool::new(false)),
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        println!("Client error: {e:?}");
    }
}
