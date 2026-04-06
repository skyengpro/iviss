use axum::http::HeaderMap;
use std::net::SocketAddr;

/// Extract the client IP address from request headers.
///
/// Priority: `X-Forwarded-For` (leftmost) → `X-Real-Ip` → peer address fallback.
pub fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // X-Forwarded-For: client, proxy1, proxy2
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            let first_ip = val.split(',').next().map(|s| s.trim().to_string());
            if let Some(ref ip) = first_ip {
                if !ip.is_empty() {
                    return first_ip;
                }
            }
        }
    }

    // X-Real-Ip: client
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Extract client IP with a ConnectInfo<SocketAddr> fallback for direct connections.
pub fn extract_client_ip_with_peer(
    headers: &HeaderMap,
    peer: Option<SocketAddr>,
) -> Option<String> {
    extract_client_ip(headers).or_else(|| peer.map(|addr| addr.ip().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extracts_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "192.168.1.100, 10.0.0.1".parse().unwrap(),
        );
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.100".into()));
    }

    #[test]
    fn test_extracts_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "10.0.0.5".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("10.0.0.5".into()));
    }

    #[test]
    fn test_forwarded_for_takes_precedence() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        headers.insert("x-real-ip", "5.6.7.8".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("1.2.3.4".into()));
    }

    #[test]
    fn test_returns_none_when_no_headers() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers), None);
    }

    #[test]
    fn test_falls_back_to_peer() {
        let headers = HeaderMap::new();
        let peer: SocketAddr = "172.18.0.1:54321".parse().unwrap();
        assert_eq!(
            extract_client_ip_with_peer(&headers, Some(peer)),
            Some("172.18.0.1".into())
        );
    }
}
