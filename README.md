# 能源管理系统 (EMS)

## 1. 已完成

### Modbus TCP 协议支持

已实现 Modbus TCP 协议的支持。

### 网关配置

以下是网关的配置示例：

```modbus_config.yaml
gateways:
  - ip: "192.168.1.100"
    port: 502
    slave_ids: [1, 2, 3]
  - ip: "192.168.1.101"
    port: 502
    slave_ids: [4] .

