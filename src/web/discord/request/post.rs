//! Post messages to a discord webhook

use std::collections::HashMap;

use crate::web::discord::request::model::Webhook;

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
/// use pulse::web::discord::request::{model::Webhook, post::post};
/// 
/// pub async fn post_to_discord(){
///     let w = Webhook::new("https://discord.com/api/webhooks/xxx/yyy".to_string());
///     post(w, "this is some plaintext".to_string());
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

pub async fn post(w: Webhook, msg: String) -> Result<String, reqwest::Error>
{

    crate::debug(format!("Posting to Discord {:?}", msg), None);
    let client = reqwest::Client::new();

    let mut map = HashMap::new();
    map.insert("content", &msg);
    
    match client.post(&w.get_addr())
        .json(&map)
        .send()
        .await
    {
        Ok(r) => Ok(format!("OK\nGot response:\n\n{:#?}", r)),
        Err(e) => Err(e)
    }

}