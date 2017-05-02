#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    global: GlobalConfig,
    #[serde(rename = "server")]
    servers: Vec<ServerConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    username: String,
    realname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    username: Option<String>,
    realname: Option<String>,
    address: String,
    port: Option<u16>,
    alias: Option<String>,
    ssl: Option<bool>,
    autoconnect: Option<bool>,
    autojoin: Option<Vec<String>>,
}
