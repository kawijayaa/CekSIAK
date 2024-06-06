use std::env;
use std::sync::Arc;

use ceksiak::SIAKSession;
use serenity::all::EventHandler;
use serenity::all::Ready;
use serenity::all::{Context, CreateMessage};
use serenity::async_trait;
use serenity::builder::CreateEmbed;
use serenity::{self, all::GatewayIntents};
use tokio;

struct Handler;

async fn notify(ctx: &Context, session: Arc<SIAKSession>) {
    println!("[CekSIAK v2] Notifying...");
    let channel = ctx
        .http
        .get_channel(
            env::var("CEKSIAK_CHANNEL_ID")
                .unwrap()
                .parse::<u64>()
                .unwrap()
                .into(),
        )
        .await
        .unwrap()
        .guild()
        .unwrap();
    let courses = session.get_scores().await.unwrap();

    if !SIAKSession::is_courses_updated(&courses) {
        println!("[CekSIAK v2] No updates found!");
        return;
    }
    SIAKSession::save_courses(&courses);

    let embed = CreateEmbed::new().title("CekSIAK v2 🦀").description(
        courses
            .iter()
            .map(|c| {
                format!(
                    "- {} ({}) - {}",
                    c.name_indonesian, c.name_english, c.status
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
    );
    channel
        .send_message(&ctx.http, CreateMessage::default().embed(embed))
        .await
        .unwrap();
    println!("[CekSIAK v2] Notified!");
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        println!("[CekSIAK v2] Logged in!");
        let session = Arc::new(SIAKSession::new());
        session
            .login(
                env::var("CEKSIAK_SIAK_USERNAME").unwrap(),
                env::var("CEKSIAK_SIAK_PASSWORD").unwrap(),
            )
            .await
            .unwrap();

        let forever = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 30));
            loop {
                interval.tick().await;
                notify(&ctx, session.clone()).await;
            }
        });

        let _ = forever.await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("CEKSIAK_BOT_TOKEN").unwrap();
    let intents = GatewayIntents::all();

    let mut discord = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .unwrap();
    if let Err(why) = discord.start().await {
        println!("Client error: {why:?}");
    }
}
