//! IPv4, IPv6, and network address detection.

use crate::regexes::{IPV4, IPV6};

/// Check if an IPv4 address is in a private/internal range
fn is_private_ipv4(octets: &[u8; 4]) -> bool {
	match octets[0] {
		0 => true,
		10 => true,
		127 => true,
		169 if octets[1] == 254 => true,
		172 if octets[1] >= 16 && octets[1] <= 31 => true,
		192 if octets[1] == 168 => true,
		_ => false,
	}
}

/// Detect if a value is an IPv4 address (excluding private ranges).
pub fn is_ipv4(value: &str) -> bool {
	let trimmed = value.trim();
	if !IPV4.is_match(trimmed) {
		return false;
	}

	let parts: Vec<&str> = trimmed.split('.').collect();
	if parts.len() != 4 {
		return false;
	}

	let octets: Result<Vec<u8>, _> = parts.iter().map(|p| p.parse::<u8>()).collect();

	if let Ok(octets_vec) = octets
		&& octets_vec.len() == 4
	{
		let arr: [u8; 4] = [octets_vec[0], octets_vec[1], octets_vec[2], octets_vec[3]];
		return !is_private_ipv4(&arr);
	}

	false
}

/// Check if an IPv6 address is in a private/internal range.
fn is_private_ipv6(value: &str) -> bool {
	let trimmed = value.trim().to_lowercase();

	// Loopback (::1)
	if trimmed == "::1" || trimmed.starts_with("0000:0000:0000:0000:0000:0000:0000:0001") {
		return true;
	}

	// Link-local (fe80::/10)
	if trimmed.starts_with("fe80") {
		return true;
	}

	// Unique local addresses (fc00::/7 and fd00::/8)
	if trimmed.starts_with("fc") || trimmed.starts_with("fd") {
		return true;
	}

	false
}

/// Detect if a value is an IPv6 address (excluding private ranges).
pub fn is_ipv6(value: &str) -> bool {
	let trimmed = value.trim();
	if !IPV6.is_match(trimmed) {
		return false;
	}

	!is_private_ipv6(trimmed)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ipv4_public() {
		assert!(is_ipv4("8.8.8.8"));
		assert!(is_ipv4("1.1.1.1"));
	}

	#[test]
	fn test_ipv4_private() {
		assert!(!is_ipv4("10.0.0.1"));
		assert!(!is_ipv4("172.16.0.1"));
		assert!(!is_ipv4("192.168.0.1"));
		assert!(!is_ipv4("127.0.0.1"));
		assert!(!is_ipv4("169.254.0.1"));
	}

	#[test]
	fn test_ipv6_public() {
		assert!(is_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
	}

	#[test]
	fn test_ipv6_private() {
		assert!(!is_ipv6("::1"));
		assert!(!is_ipv6("fe80::1"));
		assert!(!is_ipv6("fc00::1"));
		assert!(!is_ipv6("fd00::1"));
	}
}
