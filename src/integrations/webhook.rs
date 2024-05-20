use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Webhook 
{
    addr: String
}

/// Wraps a webhook url
impl Webhook
{

    pub fn new(url: String) -> Webhook
    {
        Webhook { addr: url }
    }

    pub fn get_addr(&self) -> String 
    {
        self.addr.clone()
    }
}