use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::time::{Instant, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::
{
    http::{Request, StatusCode}, 
    response::Response, 
    extract::{State, ConnectInfo},
    middleware::Next
};

pub struct Requests
{
    count: u32,
    last_request_time: Instant,
    timeout: bool
}

impl Requests
{
    pub fn clone(&self) -> Requests
    {
        Requests { count: self.count.clone(), last_request_time: self.last_request_time.clone(), timeout: false }
    }
}

pub struct IpThrottler
{
    requests_from: HashMap<Ipv4Addr, Requests>,
    max_requests_per_second: f64,
    timeout_millis: u128,
    clear_period: Duration,
    last_clear: Instant 
}

impl IpThrottler
{
    pub fn new(max_requests_per_second: f64, timeout_millis: u128, clear_period_seconds: u64) -> IpThrottler
    {
        IpThrottler 
        {
            requests_from: HashMap::new(), 
            max_requests_per_second: max_requests_per_second,
            timeout_millis: timeout_millis,
            clear_period: Duration::from_secs(clear_period_seconds),
            last_clear: Instant::now()
        }
    }

    pub fn check_clear(&mut self)
    {
        if self.last_clear.elapsed() > self.clear_period
        {
            self.requests_from.clear();
            self.last_clear = Instant::now();
        }
    }

    pub fn is_limited(&mut self, addr: SocketAddr) -> bool
    {
        let ip = addr.ip();
        let ipv4: Ipv4Addr;
    
        match ip 
        {
            IpAddr::V4(ip4) => {ipv4 = ip4}
            IpAddr::V6(_ip6) => {return true}
        }
    
        let requests = if self.requests_from.contains_key(&ipv4)
        {
            self.requests_from[&ipv4].clone()
        }
        else 
        {
            self.requests_from.insert(ipv4, Requests {count: 0 as u32, last_request_time: Instant::now(), timeout: false});
            self.requests_from[&ipv4].clone()
        };

        let time = requests.last_request_time.elapsed().as_millis();
        let requests_per_second = requests.count as f64 / (time as f64 / 1000.0);

        if requests.timeout || requests_per_second > self.max_requests_per_second
        {
            if time < self.timeout_millis
            {
                *self.requests_from.get_mut(&ipv4).unwrap() = Requests {count: requests.count, last_request_time: requests.last_request_time, timeout: true};
                return true
            }
            else 
            {
                *self.requests_from.get_mut(&ipv4).unwrap() = Requests {count: 0, last_request_time: Instant::now(), timeout: false};
                return false
            }
        }

        if time < 1000
        {
            *self.requests_from.get_mut(&ipv4).unwrap() = Requests {count: requests.count+1, last_request_time: requests.last_request_time, timeout: false};
        }
        else 
        {
            *self.requests_from.get_mut(&ipv4).unwrap() = Requests {count: 0, last_request_time: Instant::now(), timeout: false};
        }
        return false
    }
}

pub async fn handle_throttle<B>
(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<IpThrottler>>>,
    request: Request<B>,
    next: Next<B>
) -> Result<Response, StatusCode>
{

    {
        let mut throttler = state.lock().await;
        throttler.check_clear();
        if throttler.is_limited(addr)
        {
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        else 
        {
            crate::debug(format!("Allowing: {}", addr), None);
            let response = next.run(request).await;
            Ok(response)
        }
    }
    
}
