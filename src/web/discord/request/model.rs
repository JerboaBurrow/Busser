use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Webhook 
{
    addr: String
}

impl Webhook
{

    pub fn new(url: String) -> Webhook
    {
        Webhook { addr: url }
    }

    pub fn get_addr(self: Webhook) -> String 
    {
        self.addr
    }
}