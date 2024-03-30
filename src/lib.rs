#![warn(missing_docs)]

//! # rusty-sonos
//! A Rust library that allows you to discover and interact with Sonos speakers
//!
//! Connecting to a Sonos speaker:
//! ```rust,no_run
//! # tokio_test::block_on(async {
//! # use std::net::Ipv4Addr;
//! # use rusty_sonos::speaker::Speaker;
//! # use std::str::FromStr;
//! let ip_addr = Ipv4Addr::from_str("192.168.1.0").unwrap();
//!
//! let speaker = Speaker::new(ip_addr).await.unwrap();
//! # })
//! ```
//!
//! Discovering speakers on the current network:
//! ```rust,no_run
//! # tokio_test::block_on(async {
//! # use rusty_sonos::discovery::discover_devices;
//! # use std::time::Duration;
//! // search for 2 seconds, with a read timeout of 5 seconds
//! let devices = discover_devices(Duration::from_secs(2), Duration::from_secs(5)).await.unwrap();
//!
//! for device in devices {
//!    println!("{}, {}", device.friendly_name(), device.room_name())
//! }
//! # })
//! ```
//!
//! Get information about a speaker at a certain IP address
//! ```rust,no_run
//! # tokio_test::block_on(async {
//! # use std::net::Ipv4Addr;
//! # use rusty_sonos::discovery::get_speaker_info;
//! # use std::str::FromStr;
//! let ip_addr = Ipv4Addr::from_str("192.168.1.0").unwrap();
//!
//! let info = get_speaker_info(ip_addr).await.unwrap();
//! # })
//! ```

pub mod discovery;
pub mod errors;
pub mod responses;
mod services;
pub mod speaker;
mod xml;
