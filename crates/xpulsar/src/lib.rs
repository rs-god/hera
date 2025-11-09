// import pulsar crate
use pulsar::{
    Authentication, ConsumerBuilder, DeserializeMessage, Error as PulsarError, Payload, Pulsar,
    SerializeMessage, TokioExecutor, producer, producer::ProducerBuilder,
};

use serde::{Deserialize, Serialize};
use serde_json;

pub struct PulsarConf<'a> {
    addr: &'a str,
    token: Option<&'a str>, // token optional param
}

impl<'a> PulsarConf<'a> {
    pub fn new(addr: &'a str) -> Self {
        Self { addr, token: None }
    }

    pub fn with_token(mut self, token: &'a str) -> Self {
        self.token = Some(token);
        self
    }

    // create pulsar builder
    pub fn pulsar_builder(&self) -> pulsar::PulsarBuilder<TokioExecutor> {
        let mut builder = Pulsar::builder(self.addr.to_string(), TokioExecutor);
        if let Some(token) = self.token {
            let authentication = Authentication {
                name: "token".to_string(),
                data: token.to_string().into_bytes(),
            };

            builder = builder.with_auth(authentication);
        }

        builder
    }

    // create client pulsar object
    pub async fn pulsar_obj(
        &self,
        builder: pulsar::PulsarBuilder<TokioExecutor>,
    ) -> Result<Pulsar<TokioExecutor>, PulsarError> {
        builder.build().await
    }

    // create pulsar consumer
    pub fn consumer(&self, pulsar_obj: Pulsar<TokioExecutor>) -> ConsumerBuilder<TokioExecutor> {
        pulsar_obj.consumer()
    }

    // create pulsar producer
    pub fn producer(&self, pulsar_obj: Pulsar<TokioExecutor>) -> ProducerBuilder<TokioExecutor> {
        pulsar_obj.producer()
    }
}

// 定义message消息格式
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub data: String,
}

impl SerializeMessage for Message {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default() // 其他字段采用默认设置
        })
    }
}

// 实现反序列化
impl DeserializeMessage for Message {
    // 执行输出的返回结果
    type Output = Result<Message, serde_json::Error>;
    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    use futures::TryStreamExt;
    use pulsar::{
        Consumer, Error as PulsarError, message::proto::command_subscribe::SubType, producer, proto,
    };

    #[tokio::test]
    async fn message_publish() -> Result<(), PulsarError> {
        let p = super::PulsarConf::new("pulsar://127.0.0.1:6650");
        let builder = p.pulsar_builder();
        let pulsar_obj = p
            .pulsar_obj(builder)
            .await
            .expect("create pulsar obj failed");

        let topic = "my-topic";
        // create producer
        let mut producer = pulsar_obj
            .producer()
            .with_topic(topic)
            .with_name("my_producer")
            .with_options(producer::ProducerOptions {
                schema: Some(proto::Schema {
                    r#type: proto::schema::Type::String as i32,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .build()
            .await?;

        // check producer connection
        producer
            .check_connection()
            .await
            .map(|_| println!("producer connection ok"))?;

        let mut counter: usize = 0;
        loop {
            let s = counter.to_string();

            let msg = Message {
                data: "hello: ".to_string() + &s, // 发送的message内容是 {"data":"hello"}
            };
            println!("sent msg:{:?}", msg);
            // 发送消息
            producer.send_non_blocking(msg).await?;

            counter += 1;
            println!("{} messages", counter);
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            if counter >= 100 {
                break;
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn message_consumer() -> Result<(), PulsarError> {
        // 通过build的方式创建pulsar object
        let p = super::PulsarConf::new("pulsar://127.0.0.1:6650");
        let builder = p.pulsar_builder();
        let pulsar_obj = p
            .pulsar_obj(builder)
            .await
            .expect("create pulsar obj failed");

        let topic = "my-topic";
        // create consumer
        let mut consumer: Consumer<Message, _> = pulsar_obj
            .consumer()
            .with_topic(topic)
            .with_consumer_name("group-2") // 设置消费组名字
            .with_subscription_type(SubType::Exclusive)
            .with_subscription("my_topic test")
            .build()
            .await?;

        println!("consumer has run...");
        let mut counter: usize = 0;
        while let Some(msg) = consumer.try_next().await? {
            println!("metadata:{:?}", msg.message_id());
            println!("id:{:?}", msg.message_id());
            let data = match msg.deserialize() {
                Ok(data) => data,
                Err(err) => {
                    println!("could not deserialize message:{:?}", err);
                    continue;
                }
            };

            // 消费消息逻辑
            println!("got message data:{}", data.data.as_str());

            // 消息ack确认
            consumer.ack(&msg).await?;
            counter += 1;
            println!("got {} messages", counter);
        }

        Ok(())
    }
}
