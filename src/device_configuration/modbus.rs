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

// 读取并解析 YAML 文件的函数
pub fn read_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);

    // 如果文件不存在，创建空配置文件
    if !path.try_exists()? {
        // 创建空的配置结构
        let empty_config = Config {
            gateways: Vec::new(),
        };

        // 序列化为 YAML
        let yaml = serde_yaml::to_string(&empty_config)?;

        // 创建文件并写入空配置
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;

        file.write_all(yaml.as_bytes())?;
        println!("配置文件不存在，已创建空配置文件: {}", file_path);
        return Ok(empty_config);
    }

    // 打开并读取现有配置文件
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // 解析 YAML 内容
    let config: Config = serde_yaml::from_str(&contents)?;

    Ok(config)
}
