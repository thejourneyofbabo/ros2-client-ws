use async_io::Timer;
use ros2_client::{ros2, ros2::policy, Context, MessageTypeName, Name, NodeName, NodeOptions};

fn main() {
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

    smol::block_on(async {
        loop {
            count += 1;
            let message = format!("count={} {}", count, filler);
            println!("Talking, count = {} len = {}", count, message.len());
            let _ = chatter_publisher.async_publish(message).await;
            Timer::after(std::time::Duration::from_millis(100)).await;
        }
    });
}
