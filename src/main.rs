use std::time::Duration;
use twitch_eventsub::*;

fn get_env(key: &str) -> Result<String, String> {
  std::env::var(key).map_err(|_| format!("please set {key} environment variable"))
}

fn main() {
    let keys = TwitchKeys::from_secrets_env().unwrap();
    let redirect_url = get_env("TWITCH_REDIRECT_URL").unwrap();

    let twitch = TwitchEventSubApi::builder(keys)
        .set_redirect_url(redirect_url)
        .is_run_remotely()
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
        .add_subscriptions(vec![
            Subscription::ChatMessage,
        ]);

    let mut api = {
        match twitch.build() {
            Ok(api) => api,
            Err(EventSubError::TokenMissingScope) => {
                panic!("Reauthorisation of token is required for the token to have all the requested subscriptions.");
            }
            Err(EventSubError::NoSubscriptionsRequested) => {
                panic!("No subscriptions passed into builder!");
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    };

    println!("Authenticated sucessfully!");

    loop {
        let responses = api.receive_all_messages(Some(Duration::from_millis(1)));
        for response in responses {
            match response {
                ResponseType::Event(event) => {
                    match event {
                        Event::ChatMessage(message_data) => {
                            let message = message_data.message.text;
                            let username = message_data.chatter.name;
                            println!("{} said: {}", username, message);

                            if message.contains("!discord") {
                                let _ = api
                                    .send_chat_message(format!("Join the discord: https://discord.gg/eDzdYAT3sX !"))
                                    .unwrap();
                            }
                        }
                        _ => {
                        }
                    }
                }
                ResponseType::Close => println!("Twitch requested socket close."),
                _ => {}
            }
        }
    }
}
