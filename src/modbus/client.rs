use std::error::Error;
use std::time::Duration;
use tokio_modbus::client::tcp;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

// Modbus设备参数结构体
#[derive(Debug, Clone)]
pub struct ModbusDevice {
    /// Modbus设备的IP地址
    pub ip: String,
    /// Modbus设备的端口号（默认502）
    pub port: u16,
    /// 从站ID（范围1-247）
    pub slave_id: u8,
}

// Modbus操作trait
#[async_trait::async_trait]
pub trait ModbusOperation {
    /// 从Modbus设备读取寄存器
    ///
    /// # 参数说明
    /// * `function_code` - 功能码:
    ///   * 0x01: 读取线圈状态（读写线圈）
    ///   * 0x02: 读取离散输入状态（只读线圈）
    ///   * 0x03: 读取保持寄存器（读写寄存器）
    ///   * 0x04: 读取输入寄存器（只读寄存器）
    /// * `address` - 起始地址（0-65535）
    /// * `quantity` - 读取数量（线圈1-2000，寄存器1-125）
    ///
    /// # 返回值
    /// * `Ok(Vec<u16>)` - 返回读取到的数据
    /// * `Err` - 返回错误信息
    async fn read_registers(
        &mut self,
        function_code: u8,
        address: u16,
        quantity: u16,
    ) -> Result<Vec<u16>, Box<dyn Error>>;

    /// 向Modbus设备写入寄存器
    ///
    /// # 参数说明
    /// * `function_code` - 功能码:
    ///   * 0x05: 写入单个线圈
    ///   * 0x06: 写入单个寄存器
    ///   * 0x0F: 写入多个线圈（最多1968个）
    ///   * 0x10: 写入多个寄存器（最多123个）
    /// * `address` - 起始地址（0-65535）
    /// * `quantity` - 写入数量
    /// * `values` - 要写入的数据:
    ///   * 线圈: 0表示OFF，非0表示ON
    ///   * 寄存器: 16位无符号整数（0-65535）
    ///
    /// # 返回值
    /// * `Ok(())` - 写入成功
    /// * `Err` - 返回错误信息
    async fn write_registers(
        &mut self,
        function_code: u8,
        address: u16,
        quantity: u16,
        values: Vec<u16>,
    ) -> Result<(), Box<dyn Error>>;

    /// 断开与Modbus设备的连接
    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>>;
}

// Modbus客户端结构体
pub struct ModbusClient {
    device: ModbusDevice,
    ctx: Option<Context>,
}

impl ModbusClient {
    /// 创建新的Modbus客户端实例
    ///
    /// # 参数说明
    /// * `device` - Modbus设备配置:
    ///   * ip: 设备IP地址（例如："192.168.1.100"）
    ///   * port: 端口号（默认502）
    ///   * slave_id: 从站ID（范围1-247）
    pub fn new(device: ModbusDevice) -> Self {
        ModbusClient { device, ctx: None }
    }

    /// 连接到Modbus服务器
    ///
    /// # 说明
    /// * 连接超时时间为5秒
    /// * 连接成功后才能执行读写操作
    ///
    /// # 返回值
    /// * `Ok(())` - 连接成功
    /// * `Err` - 连接失败，返回错误信息
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let socket_addr = format!("{}:{}", self.device.ip, self.device.port).parse()?;
        let slave = Slave(self.device.slave_id);

        println!("尝试连接到Modbus服务器 {}...", socket_addr);

        match tokio::time::timeout(
            Duration::from_secs(5),
            tcp::connect_slave(socket_addr, slave),
        )
        .await
        {
            Ok(result) => match result {
                Ok(ctx) => {
                    println!("成功连接到服务器");
                    self.ctx = Some(ctx);
                    Ok(())
                }
                Err(e) => {
                    println!("连接失败: {:?}", e);
                    Err(e.into())
                }
            },
            Err(_) => {
                println!("连接尝试超时");
                Err("Connection timeout".into())
            }
        }
    }
}

#[async_trait::async_trait]
impl ModbusOperation for ModbusClient {
    async fn read_registers(
        &mut self,
        function_code: u8,
        address: u16,
        quantity: u16,
    ) -> Result<Vec<u16>, Box<dyn Error>> {
        let ctx = self.ctx.as_mut().ok_or("客户端未连接")?;

        let result = match function_code {
            //OXO1 读取线圈
            0x01 => {
                let coils =
                    tokio::time::timeout(Duration::from_secs(5), ctx.read_coils(address, quantity))
                        .await?
                        .expect("读取线圈失败");

                // 展平嵌套的 Vec<Vec<bool>> 并映射为 Vec<u16>
                let coils_vec: Vec<u16> = coils
                    .into_iter() // 获取 Vec<Vec<bool>> 的所有权
                    .flatten() // 展平为 Vec<bool>
                    .map(|b| if b { 1 } else { 0 }) // 将每个 bool 映射为 u16
                    .collect();
                Ok(coils_vec)
            }
            //OXO2 读取输入寄存器
            0x02 => {
                let discrete_inputs = tokio::time::timeout(
                    Duration::from_secs(1),
                    ctx.read_discrete_inputs(address, quantity),
                )
                .await?
                .expect("读取离散输入失败");

                // 展平嵌套的 Vec<Vec<bool>> 并映射为 Vec<u16>
                let discrete_inputs_vec: Vec<u16> = discrete_inputs
                    .into_iter() // 获取 Vec<Vec<bool>> 的所有权
                    .flatten() // 展平为 Vec<bool>
                    .map(|b| if b { 1 } else { 0 }) // 将每个 bool 映射为 u16
                    .collect();
                Ok(discrete_inputs_vec)
            }
            //OXO3 读取保持寄存器
            0x03 => tokio::time::timeout(
                Duration::from_secs(5),
                ctx.read_holding_registers(address, quantity),
            )
            .await?
            .expect("读取保持寄存器失败"),
            //OXO4 读取输入寄存器
            0x04 => tokio::time::timeout(
                Duration::from_secs(5),
                ctx.read_input_registers(address, quantity),
            )
            .await?
            .expect("读取输入寄存器失败"),
            _ => return Err("不支持的功能码".into()),
        };

        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                println!("读取失败: {:?}", e);
                Err(e.into())
            }
        }
    }

    async fn write_registers(
        &mut self,
        function_code: u8,
        address: u16,
        quantity: u16,
        values: Vec<u16>,
    ) -> Result<(), Box<dyn Error>> {
        let ctx = self.ctx.as_mut().ok_or("客户端未连接")?;

        let result = match function_code {
            //OXO5 写入单个线圈
            0x05 => {
                if quantity != 1 {
                    return Err("功能码0x05仅支持写入单个线圈".into());
                }

                let coil = values[0] >= 1;
                tokio::time::timeout(Duration::from_secs(5), ctx.write_single_coil(address, coil))
                    .await
            }
            //0x0F 写入多个线圈
            0x0f => {
                if values.len() != quantity as usize {
                    return Err("值的长度是与数量不匹配".into());
                }
                let coils: Vec<bool> = values.into_iter().map(|v| v >= 1).collect();
                tokio::time::timeout(
                    Duration::from_secs(5),
                    ctx.write_multiple_coils(address, &coils),
                )
                .await
            }
            //OXO6 写入单个寄存器
            0x06 => {
                if quantity != 1 {
                    return Err("功能码0x06仅支持写入单个寄存器".into());
                }
                tokio::time::timeout(
                    Duration::from_secs(5),
                    ctx.write_single_register(address, values[0]),
                )
                .await
            }
            //OXO10 写入多个寄存器
            0x10 => {
                if values.len() != quantity as usize {
                    return Err("值的长度与数量不匹配".into());
                }
                tokio::time::timeout(
                    Duration::from_secs(5),
                    ctx.write_multiple_registers(address, &values),
                )
                .await
            }
            _ => return Err("不支持的功能码".into()),
        };

        match result {
            Ok(response) => match response {
                Ok(_) => Ok(()),
                Err(e) => {
                    println!("写入失败: {:?}", e);
                    Err(e.into())
                }
            },
            Err(_) => {
                println!("写入超时");
                Err("写入超时".into())
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(ctx) = self.ctx.as_mut() {
            if let Err(e) = ctx.disconnect().await {
                println!("断开连接失败: {:?}", e);
                return Err(e.into());
            }
            println!("连接已关闭");
        }
        Ok(())
    }
}
