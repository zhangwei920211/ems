use rumqttc::{MqttOptions, Client};

pub struct MqttClient {
    client: Client,
}

impl MqttClient {
    pub fn new(client_id: &str, broker: &str) -> MqttClient {
        let mut settings = MqttOptions::new(client_id, broker);
        settings.set_keep_alive(5);

        let (client, _) = Client::new(settings, 10);
        MqttClient { client }
    }

    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), rumqttc::Error> {
        self.client.publish(topic, payload, 1, false)
    }

    // ...处理 MQTT 客户端功能的附加方法...
}

