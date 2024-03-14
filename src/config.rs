use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

/// 服务端配置文件
/// v4port: ipv4端口
/// v6port: ipv6端口
/// token: 服务端token
/// money_mode: 单机模式/多机模式
/// sql_mode: sqlite/postgres/mongodb
/// postgres_config: Postgres配置
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub v4port: u16,
    pub v6port: u16,
    pub token: String,
    pub money_mode: String,
    pub sql_mode: String,
    pub postgres_config: PostgresConfig,
}

/// Postgres配置
/// host: 数据库地址
/// port: 数据库端口
/// user: 数据库用户名
/// password: 数据库密码
/// database: 数据库名
#[derive(Clone, Serialize, Deserialize,Default)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

/// 默认配置
/// 默认配置文件路径为config.yml
///  v4port: 2000
/// v6port: 2000
/// token: 随机生成的16位字符串
/// money_mode: single/multi
/// sql_mode: sqlite/postgres/mongodb
/// postgres_config: PostgresConfig默认配置
impl Default for ServerConfig {
    fn default() -> Self {
        let file_path = "config.yml";
        let config = ServerConfig {
            v4port: 2000,
            v6port: 2000,
            token: generate_random(16),
            money_mode: "single/multi".to_string(),
            sql_mode: "sqlite/postgres/mongodb".to_string(),
            postgres_config:PostgresConfig::default(),
        };
        match read_yml(&file_path) {
            Ok(config) => config,
            Err(_err) => {
                let _ = write_config_to_yml(&config, file_path);
                config
            }
        }
    }
}



use rand::Rng;
///随机生成指定长度的密钥
pub fn generate_random(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let characters: Vec<char> = "QWERTYUIOPASDFGHJKLZXCVBNMabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let key: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..characters.len());
            characters[idx]
        })
        .collect();
    key
}

///生成指定长度的数字密钥
pub fn _generate_random_number(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let characters: Vec<char> = "0123456789".chars().collect();
    let key: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..characters.len());
            characters[idx]
        })
        .collect();
    key
}

// 写入到yml文件
pub fn write_config_to_yml(
    config: &ServerConfig,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml_string = serde_yaml::to_string(config)?;
    let mut file = File::create(file_path)?;
    file.write_all(yaml_string.as_bytes())?;
    Ok(())
}

pub fn read_yml(file_path: &str) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: ServerConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}

//初始化日志输出
pub fn init_log() {
    use chrono::Local;

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    let mut builder = env_logger::Builder::from_env(env);
    builder
        .format(|buf, record| {
            let level = { buf.default_level_style(record.level()) };
            writeln!(
                buf,
                "{} {} {} [{}] {}",
                format_args!("{:<5}", level),
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();
}
