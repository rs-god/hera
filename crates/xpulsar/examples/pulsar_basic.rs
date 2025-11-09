use futures::TryStreamExt;
use pulsar::{
    Consumer, Error as PulsarError, message::proto::command_subscribe::SubType, producer, proto,
};
use xpulsar;
use xpulsar::Message;

#[tokio::main]
async fn main() -> Result<(), PulsarError> {
    message_publish().await
    // message_consumer().await
}

#[tokio::test]
async fn test_message_consumer() -> Result<(), PulsarError> {
    message_consumer().await
}

async fn message_publish() -> Result<(), PulsarError> {
    let p = xpulsar::PulsarConf::new("pulsar://127.0.0.1:6650");
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
async fn test_consumer() -> Result<(), PulsarError> {
    message_consumer().await
}

async fn message_consumer() -> Result<(), PulsarError> {
    // 通过build的方式创建pulsar object
    let p = xpulsar::PulsarConf::new("pulsar://127.0.0.1:6650");
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
        // println!("metadata:{:?}", msg.message_id());
        // println!("id:{:?}", msg.message_id());
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
