# CekSIAK

CekSIAK is a Discord bot that notifies you if scores on your courses have been published on SIAK.

## Setup

1. Clone the repository
2. Create a `.env` file and fill out the following variables (you can copy paste from `.env.example`):

```env
CEKSIAK_SIAK_USERNAME=
CEKSIAK_SIAK_PASSWORD=
CEKSIAK_BOT_TOKEN=
CEKSIAK_CHANNEL_ID=
```

3. If you are using Docker, run `docker-compose up -d` to start the bot. If you don't want to use Docker, run `cargo run` to start the bot.
