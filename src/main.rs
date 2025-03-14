mod device_configuration;
mod modbus;

use crate::modbus::client::{ModbusClient, ModbusDevice as ClientModbusDevice, ModbusOperation};
use device_configuration::modbus::read_config;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 指定 YAML 配置文件路径
    let file_path = "modbus_config.yaml";
    println!("正在读取配置文件: {}", file_path);

    // 读取和解析 YAML 配置文件
    let config = match read_config(file_path) {
        Ok(cfg) => {
            println!("配置文件加载成功");
            cfg
        }
        Err(e) => {
            eprintln!("无法读取配置文件: {}", e);
            return Err(e);
        }
    };

    // 检查是否有配置的网关设备
    if config.gateways.is_empty() {
        println!("警告: 配置文件中没有定义Modbus设备");
        return Ok(());
    }

    // 遍历所有配置的网关设备
    for gateway in &config.gateways {
        println!("\n处理网关: {}:{}", gateway.ip, gateway.port);

        // 遍历网关中的所有从站ID
        for &slave_id in &gateway.slave_ids {
            println!("连接从站ID: {}", slave_id);

            // 从配置创建客户端设备实例
            let device = ClientModbusDevice {
                ip: gateway.ip.clone(),
                port: gateway.port,
                slave_id,
            };

            // 创建并连接客户端
            let mut client = ModbusClient::new(device.clone());
            match client.connect().await {
                Ok(_) => println!(
                    "成功连接到设备 {}:{} 从站ID {}",
                    gateway.ip, gateway.port, slave_id
                ),
                Err(e) => {
                    println!(
                        "连接失败: {}:{} 从站ID {}, 错误: {:?}",
                        gateway.ip, gateway.port, slave_id, e
                    );
                    continue; // 连接失败，继续处理下一个从站ID
                }
            }

            // 暂停一小段时间确保连接稳定
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // 执行Modbus操作示例
            println!("读取输入寄存器...");
            match client.read_registers(0x04, 0, 4).await {
                Ok(values) => println!("输入寄存器值: {:?}", values),
                Err(e) => println!("读取输入寄存器失败: {:?}", e),
            }

            println!("读取保持寄存器...");
            match client.read_registers(0x03, 0, 4).await {
                Ok(values) => println!("保持寄存器值: {:?}", values),
                Err(e) => println!("读取保持寄存器失败: {:?}", e),
            }

            // 示例写操作 (可选)
            println!("写入线圈...");
            match client.write_registers(0x0F, 0, 4, vec![1, 1, 1, 1]).await {
                Ok(_) => println!("写入线圈成功"),
                Err(e) => println!("写入线圈失败: {:?}", e),
            }

            // 断开连接
            if let Err(e) = client.disconnect().await {
                println!("断开连接失败: {:?}", e);
            } else {
                println!("已断开与设备的连接");
            }
        }
    }

    println!("\n所有设备操作完成");
    Ok(())
}
