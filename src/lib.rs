#![warn(missing_docs)]

//! # rusty-sonos
//! A Rust library that allows you to discover and interact with Sonos speakers
//!
//! Connecting to a Sonos speaker:
//! ```rust
//! let ip_addr = Ipv4Addr::from_str("192.168.1.0").unwrap();
//!
//! let speaker = Speaker::new(ip_addr).await.unwrap();
//! ```
//!
//! Discovering speakers on the current network:
//! ```rust
//! // search for 2 seconds, with a read timeout of 5 seconds
//! let devices = discover_devices(2, 5).await.unwrap();
//!
//! for device in devices {
//!    println!("{}, {}", device.friendly_name, device.room_name)
//! }
//! ```
//!
//! Get information about a speaker at a certain IP address
//! ```rust
//! let ip_addr = Ipv4Addr::from_str("192.168.1.0").unwrap();
//!
//! let info = get_speaker_info(ip_addr).await.unwrap();
//! ```

pub mod discovery;
pub mod errors;
pub mod responses;
mod services;
pub mod speaker;
mod xml;
