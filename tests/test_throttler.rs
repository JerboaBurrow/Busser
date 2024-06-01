mod common;

#[cfg(test)]
mod test_throttle
{
    use std::net::{Ipv4Addr, SocketAddr};

    use busser::server::throttle::{IpThrottler, Request};
    use openssl::sha::sha512;

    #[test]
    pub fn test_request()
    {
        let r1 = Request::new
        (
            Ipv4Addr::new(127, 0, 0, 0),
            "/index.html"
        );

        let r2 = Request::new
        (
            Ipv4Addr::new(127, 0, 0, 0),
            "/page.html"
        );

        let r3 = Request::new
        (
            Ipv4Addr::new(127, 1, 1, 1),
            "/index.html"
        );

        assert_ne!(r1, r2);
        assert_ne!(r1, r3);
        assert_ne!(r2, r3);

        assert_eq!(r1.hash(), sha512(&["/index.html".as_bytes(), &Ipv4Addr::new(127, 0, 0, 0).octets()].concat()));
        assert_eq!(r2.hash(), sha512(&["/page.html".as_bytes(), &Ipv4Addr::new(127, 0, 0, 0).octets()].concat()));
        assert_eq!(r3.hash(), sha512(&["/index.html".as_bytes(), &Ipv4Addr::new(127, 1, 1, 1).octets()].concat()));
    }

    #[test]
    pub fn test_throttler()
    {
        let mut throttle = IpThrottler::new(1e-9, 5000, 3600);
        let ip = Ipv4Addr::new(127, 0, 0, 0);
        let path = "/index.html";
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), false);
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), true);
        throttle.check_clear();
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), true);

        let mut throttle = IpThrottler::new(1e-9, 5000, 0);
        let ip = Ipv4Addr::new(127, 0, 0, 0);
        let path = "/index.html";
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), false);
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), true);
        throttle.check_clear();
        assert_eq!(throttle.is_limited(SocketAddr::new(std::net::IpAddr::V4(ip), 80), path), false);
    }
}