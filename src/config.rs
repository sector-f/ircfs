extern crate irc;
use irc::client::prelude::Config;

use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub owners: Option<Vec<String>>,
    pub nickname: Option<String>,
    pub nick_password: Option<String>,
    pub alt_nicks: Option<Vec<String>>,
    pub username: Option<String>,
    pub realname: Option<String>,
    pub server: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
    pub use_ssl: Option<bool>,
    pub cert_path: Option<String>,
    pub encoding: Option<String>,
    pub channels: Option<Vec<String>>,
    pub channel_keys: Option<HashMap<String, String>>,
    pub umodes: Option<String>,
    pub user_info: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>,
    pub ping_time: Option<u32>,
    pub ping_timeout: Option<u32>,
    pub should_ghost: Option<bool>,
    pub ghost_sequence: Option<Vec<String>>,
    pub options: Option<HashMap<String, String>>,
    pub burst_window_length: Option<u32>,
    pub max_messages_in_burst: Option<u32>,
}

pub fn convert_config(config: ServerConfig) -> Config {
    let config = config.clone();

    Config {
        owners: config.owners,
        nickname: config.nickname,
        nick_password: config.nick_password,
        alt_nicks: config.alt_nicks,
        username: config.username,
        realname: config.realname,
        server: config.server,
        port: config.port,
        password: config.password,
        use_ssl: config.use_ssl,
        cert_path: config.cert_path,
        encoding: config.encoding,
        channels: config.channels,
        channel_keys: config.channel_keys,
        umodes: config.umodes,
        user_info: config.user_info,
        version: config.version,
        source: config.source,
        ping_time: config.ping_time,
        ping_timeout: config.ping_timeout,
        should_ghost: config.should_ghost,
        ghost_sequence: config.ghost_sequence,
        options: config.options,
        use_mock_connection: None,
        mock_initial_value: None,
        burst_window_length: config.burst_window_length,
        max_messages_in_burst: config.max_messages_in_burst,
    }
}
