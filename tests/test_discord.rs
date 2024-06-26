mod common;

#[cfg(test)]
mod discord
{
    use busser::integrations::{discord::post::{post_message, try_post}, webhook::Webhook};

    #[tokio::test]
    async fn test_webhook()
    {
        let w = Webhook::new("https://discord.com/api/webhooks/xxx/yyy".to_string());

        assert_eq!(w.get_addr(), "https://discord.com/api/webhooks/xxx/yyy");

        assert!(post_message(&w, &"400".to_string()).await.is_ok());
    }

    #[tokio::test]
    async fn test_err_webhook()
    {
        let w = Webhook::new("not_a_domain".to_string());
        assert!(post_message(&w, &"400".to_string()).await.is_err());
    }

    #[tokio::test]
    async fn test_try_post()
    {
        try_post(None, &"".to_string()).await;
        let w = Webhook::new("not_a_domain".to_string());
        try_post(Some(w), &"".to_string()).await;
    }
}