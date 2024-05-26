//! Post messages to a discord webhook

use std::collections::HashMap;

use crate::integrations::webhook::Webhook;

/// Send a simple plaintext string message, msg, to the webhook w
/// 
/// Should not be used to post JSON payloads, msg will be sent to 
/// the webhook wrapped in the content section. It will appear as 
/// plaintext on the server
/// 
/// For example
/// 
/// # Example
/// ```rust
/// 
/// use busser::integrations::{webhook::Webhook, discord::post::post_message};
/// 
/// pub async fn post_to_discord(){
///     let w = Webhook::new("https://discord.com/api/webhooks/xxx/yyy".to_string());
///     post_message(&w, &"this is some plaintext".to_string());
/// }
/// ```
/// 
/// is equivalent to the following POST request
/// 
/// ```not_rust
///  POST /api/webhooks/xxx/yyy HTTP/1.1
///  Host: discord.com
///  Accept: application/json
///  Content-Type:application/json
///  Content-Length: xx
///  {"content": "this is some plaintext"}
/// ``` 

/// Post a text message to a discord Webhook
pub async fn post_message(w: &Webhook, msg: &String) -> Result<String, reqwest::Error>
{
    crate::debug(format!("Posting to Discord {:?}", msg), None);
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("content", &msg);
    
    match client.post(w.clone().get_addr())
        .json(&map)
        .send()
        .await
    {
        Ok(r) => Ok(format!("OK\nGot response:\n\n{:#?}", r)),
        Err(e) => Err(e)
    }
}

/// Attempt to post a message to the discord webhook
pub async fn try_post(webhook: Option<Webhook>, msg: &String)
{
    match webhook
    {
        Some(w) => match post_message(&w, msg).await
            {
                Ok(_s) => (),
                Err(e) => {crate::debug(format!("Error posting to discord\n{}", e), None);}
            },
        None => {crate::debug(format!("Discord webhook is None"), None);}
    }
}