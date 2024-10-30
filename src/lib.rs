// src/lib.rs

pub mod adb;
pub mod constants;
pub mod enums;
pub mod utils;
pub mod models;

pub use self::adb::client::Client;
pub use self::enums::device_transport::DeviceTransport;
pub use self::enums::pull_result::PullResult;
pub use self::enums::push_result::PushResult;

pub use self::adb::app_installation;
pub use self::adb::debugging;
pub use self::adb::file_transfer;
pub use self::adb::io;
pub use self::adb::network;
pub use self::adb::protocol;
pub use self::adb::scripting;
pub use self::adb::security;
pub use self::adb::shell;

pub use self::adb::app_installation::{install, uninstall};
pub use self::adb::file_transfer::{push, pull};

pub use self::utils::strip_adb_prefix;

pub use self::models::remote_dir_entry::RemoteDirEntry;
pub use self::models::remote_metadata::RemoteMetadata;
pub use self::models::stat_data::StatData;
