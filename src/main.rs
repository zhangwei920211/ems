mod modbus;
use crate::modbus::client::{ModbusClient, ModbusDevice, ModbusOperation};
mod device_configuration;
use device_configuration::modbus::read_config;
#[tokio::main]
async fn main() {
    //YAML
    // 指定 YAML 文件路径
    let file_path = "modbus_config.yaml";

    // 调用 read_config 函数来读取和解析 YAML 文件
    match read_config(file_path) {
        Ok(config) => {
            // 输出解析结果
            for gateway in config.gateways {
                println!("Gateway IP: {}, Port: {}", gateway.ip, gateway.port);
                println!("Slave IDs: {:?}", gateway.slave_ids);
            }
        }
        Err(e) => {
            // 错误处理
            eprintln!("错误: {}", e);
        }
    }
    //YAML

    // 创建设备配置

    let device = ModbusDevice {
        ip: "192.168.0.80".to_string(), // Modbus设备的IP地址
        port: 10123,                    // 默认端口号502
        slave_id: 1,                    // 从站ID（1-247）
    };
    println!("设备: {:?}", device);

    // 创建并连接客户端
    let mut client = ModbusClient::new(device);
    if let Err(e) = client.connect().await {
        println!("连接失败: {:?}", e);
        return;
    }

    // 等待0.1秒后验证
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 读取输入寄存器 (功能码: 0x04, 起始地址: 0, 读取数量: 4)
    println!("验证输入寄存器状态...");
    match client.read_registers(0x04, 0, 4).await {
        Ok(values) => println!("输入寄存器状态: {:?}", values),
        Err(e) => println!("读取输入寄存器失败: {:?}", e),
    }

    println!("写入线圈寄存器");
    match client.write_registers(0x0F, 0, 4, vec![1, 0, 0, 1]).await {
        Ok(_) => println!("写入线圈寄存器成功"),
        Err(e) => println!("写入线圈寄存器失败{}", e),
    }
    // 读取保持寄存器示例 (功能码0x03)
    // address=0: 起始地址
    // quantity=4: 读取4个寄存器
    println!("验证保持寄存器状态...");

    match client.read_registers(0x03, 0, 4).await {
        Ok(values) => println!("保持寄存器状态: {:?}", values),
        Err(e) => println!("读取保持寄存器失败: {:?}", e),
    }

    // 断开连接
    if let Err(e) = client.disconnect().await {
        println!("断开连接失败: {:?}", e);
    } else {
        println!("连接已关闭");
    }
}
