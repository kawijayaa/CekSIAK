use log;
use std::env;
use std::sync::Arc;

use ceksiak::SIAKSession;
use serenity::all::EventHandler;
use serenity::all::Ready;
use serenity::all::{Context, CreateMessage};
use serenity::async_trait;
use serenity::builder::CreateEmbed;
use serenity::model::channel::Channel;
use serenity::{self, all::GatewayIntents};
use tokio;

struct Handler;

async fn notify(ctx: &Context, session: Arc<SIAKSession>) {
    log::info!("Notifying...");
    let channel = match ctx
        .http
        .get_channel(
            env::var("CEKSIAK_CHANNEL_ID")
                .unwrap()
                .parse::<u64>()
                .unwrap()
                .into(),
        )
        .await
    {
        Ok(c) => c,
        Err(_) => {
            log::error!("Cannot get channel!");
            return;
        }
    };

    let courses = match session.get_scores().await {
        Some(courses) => courses,
        None => {
            log::error!("Cannot get courses! Logging in again...");
            session
                .login(
                    env::var("CEKSIAK_SIAK_USERNAME").unwrap(),
                    env::var("CEKSIAK_SIAK_PASSWORD").unwrap(),
                )
                .await
                .unwrap();
            notify(ctx, session).await;
            return;
        }
    };

    if !SIAKSession::is_courses_updated(&courses) {
        log::info!("No updates found!");
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
    match channel {
        Channel::Guild(ref guild) => {
            guild
                .send_message(&ctx.http, CreateMessage::default().embed(embed))
                .await
                .unwrap();
        }
        Channel::Private(ref private) => {
            private
                .send_message(&ctx.http, CreateMessage::default().embed(embed))
                .await
                .unwrap();
        }
        _ => {}
    }
    log::info!("Sent notification to {}!", &channel.id());
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        log::info!("Logged in!");
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
    env_logger::init();

    let token = env::var("CEKSIAK_BOT_TOKEN").unwrap();
    let intents = GatewayIntents::all();

    let mut discord = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .unwrap();
    if let Err(e) = discord.start().await {
        log::error!("Client error: {e:?}");
    }
}
