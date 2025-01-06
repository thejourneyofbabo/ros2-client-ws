use ros2_client::{ros2, ros2::policy, Context, MessageTypeName, Name, NodeName, NodeOptions};
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    let context = Context::new().unwrap();
    let mut node = context
        .new_node(
            NodeName::new("/rustdds", "talker").unwrap(),
            NodeOptions::default(),
        )
        .unwrap();

    let reliable_qos = ros2::QosPolicyBuilder::new()
        .history(policy::History::KeepLast { depth: 10 })
        .reliability(policy::Reliability::Reliable {
            max_blocking_time: ros2::Duration::from_millis(100),
        })
        .durability(policy::Durability::TransientLocal)
        .build();

    let chatter_topic = node
        .create_topic(
            &Name::new("/", "topic").unwrap(),
            MessageTypeName::new("std_msgs", "String"),
            &reliable_qos,
        )
        .unwrap();

    let chatter_publisher = node
        .create_publisher::<String>(&chatter_topic, None)
        .unwrap();
    let mut count = 0;

    let filler: String =
        "All work and no play makes ROS a dull boy. All play and no work makes RTPS a mere toy. "
            .repeat(2);

    // Create an interval for a periodic 100ms tick
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await; // Wait for the next tick
        count += 1;
        let message = format!("count={} {}", count, filler);
        println!("Talking, count = {} len = {}", count, message.len());
        if let Err(e) = chatter_publisher.async_publish(message).await {
            eprintln!("Failed to publish message: {:?}", e);
        }
    }
}
