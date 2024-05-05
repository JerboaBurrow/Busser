mod common;

#[cfg(test)]
mod discord
{
    use busser::web::discord::request::{model::Webhook, post::post};


    #[tokio::test]
    async fn test_webhook()
    {
        let w = Webhook::new("https://discord.com/api/webhooks/xxx/yyy".to_string());

        assert_eq!(w.get_addr(), "https://discord.com/api/webhooks/xxx/yyy");

        assert!(post(&w, "400".to_string()).await.is_ok());
    }
}