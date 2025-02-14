// src/config.rs

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

// 定义 ModbusDevice 结构体
#[derive(Deserialize, Serialize, Debug)]
pub struct ModbusDevice {
    pub ip: String,
    pub port: u16,
    pub slave_ids: Vec<u8>,
}

// 定义 Config 结构体
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub gateways: Vec<ModbusDevice>,
}

// 默认配置
fn default_config() -> Config {
    Config {
        gateways: vec![ModbusDevice {
            ip: "模板".to_string(),
            port: 0,
            slave_ids: vec![],
        }],
    }
}

// 读取并解析 YAML 文件的函数
pub fn read_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);

    // 如果文件不存在，创建默认配置并写入
    if !path.try_exists()? {
        let default = default_config();
        let yaml = serde_yaml::to_string(&default)?; // 将默认配置序列化为 YAML 字符串

        // 使用 OpenOptions 创建文件并写入默认配置
        let mut file = OpenOptions::new()
            .create(true) // 如果文件不存在，则创建
            .write(true) // 以写入模式打开文件
            .truncate(true) // 清空文件内容（如果已存在）
            .open(file_path)?;

        // 写入默认配置
        file.write_all(yaml.as_bytes())?;

        println!("配置文件不存在，已创建默认配置文件: {}", file_path);
        return Ok(default); // 返回默认配置
    }

    // 打开 YAML 文件
    let mut file = File::open(file_path)?;

    // 读取文件内容
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // 解析 YAML 内容为 Config 结构体
    let config: Config = serde_yaml::from_str(&contents)?;

    // 返回解析结果
    Ok(config)
}
