use futures::StreamExt;
use ros2_client::{ros2::policy, *};

#[tokio::main]
async fn main() {
    let context = Context::new().unwrap();
    let mut node = context
        .new_node(
            NodeName::new("/rustdds", "rustdds_listener").unwrap(),
            NodeOptions::new().enable_rosout(true),
        )
        .unwrap();

    // QoS 설정에서 Durability를 Volatile로 변경하고, Reliability를 BestEffort로 설정
    let reliable_qos = ros2::QosPolicyBuilder::new()
        .history(policy::History::KeepLast { depth: 10 }) // KeepLast 설정
        .reliability(policy::Reliability::BestEffort) // BestEffort로 설정
        .durability(policy::Durability::Volatile) // Volatile 설정
        .build();

    let chatter_topic = node
        .create_topic(
            &Name::new("/", "topic").unwrap(),
            MessageTypeName::new("std_msgs", "String"),
            &ros2_client::DEFAULT_SUBSCRIPTION_QOS,
        )
        .unwrap();

    let chatter_subscription = node
        .create_subscription::<String>(&chatter_topic, Some(reliable_qos))
        .unwrap();

    let subscription_stream = chatter_subscription
        .async_stream()
        .for_each(|result| async {
            match result {
                Ok((msg, _)) => println!("I heard: {msg}"), // 실시간 메시지만 출력
                Err(e) => eprintln!("Receive request error: {:?}", e),
            }
        });

    rosout!(
        node,
        ros2::LogLevel::Info,
        "wow. very interesting. such topics. much subscribe."
    );

    subscription_stream.await;
}
