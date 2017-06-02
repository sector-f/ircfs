extern crate irc;
use irc::client::prelude::Config as IrcConfig;

use std::collections::HashMap;
// use std::Default;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FsConfig {
    pub global: GlobalConfig,
    #[serde(rename = "server")]
    servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    nickname: String,
    nick_password: Option<String>,
    alt_nicks: Option<Vec<String>>,
    username: Option<String>,
    realname: Option<String>,
    owners: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    nickname: Option<String>,
    nick_password: Option<String>,
    alt_nicks: Option<Vec<String>>,
    username: Option<String>,
    realname: Option<String>,
    owners: Option<Vec<String>>,
    server: String,
    port: Option<u16>,
    password: Option<String>,
    use_ssl: Option<bool>,
    encoding: Option<String>,
    channels: Option<Vec<String>>,
    channel_keys: Option<HashMap<String, String>>,
    umodes: Option<String>,
    user_info: Option<String>,
    ping_time: Option<u32>,
    ping_timeout: Option<u32>,
    should_ghost: Option<bool>,
    ghost_sequence: Option<Vec<String>>,
    options: Option<HashMap<String, String>>,
}

macro_rules! extract {
    ($config:ident, $server:ident, $field:ident) => {
        match $config.global.$field {
            Some(ref val) => { Some(val.clone()) },
            None => { $server.$field },
        }
    }
}

pub fn convert_config(config: &FsConfig) -> Vec<IrcConfig> {
    let config = config.clone();
    let mut irc_configs = Vec::new();

    for server in config.servers {
        irc_configs.push(
            IrcConfig {
                nickname: Some(config.global.nickname.clone()),
                nick_password: extract!(config, server, nick_password),
                owners: extract!(config, server, owners),
                alt_nicks: extract!(config, server, alt_nicks),
                username: extract!(config, server, username),
                realname: extract!(config, server, realname),
                server: Some(server.server),
                port: server.port,
                password: server.password,
                use_ssl: server.use_ssl,
                encoding: server.encoding,
                channels: server.channels,
                channel_keys: server.channel_keys,
                umodes: server.umodes,
                user_info: server.user_info,
                ping_time: server.ping_time,
                ping_timeout: server.ping_timeout,
                should_ghost: server.should_ghost,
                ghost_sequence: server.ghost_sequence,
                options: server.options,
                .. Default::default()
            }
        );
    }

    irc_configs
}
