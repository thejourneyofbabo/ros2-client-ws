use ros2_client::{ros2, ros2::policy, Context, MessageTypeName, Name, NodeName, NodeOptions};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Twist {
    pub linear: Vector3,
    pub angular: Vector3,
}

impl Vector3 {
    fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Twist {
    fn new() -> Self {
        Self {
            linear: Vector3::zero(),
            angular: Vector3::zero(),
        }
    }
}

async fn publish_twist(
    publisher: &ros2_client::Publisher<Twist>,
    loop_count: &mut i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut message = Twist::new();

    match loop_count {
        0..=99 => {
            // Move forward
            message.linear.x = 2.0;
        }
        100..=199 => {
            // Rotate
            message.angular.z = 1.0;
        }
        _ => {
            // Reset counter
            *loop_count = 0;
        }
    }

    if let Err(e) = publisher.async_publish(message).await {
        eprintln!("Failed to publish message: {:?}", e);
        return Err(e.into());
    }

    *loop_count += 1;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::new().unwrap();
    let mut node = context
        .new_node(
            NodeName::new("/ros2_demo", "moving_turtle").unwrap(),
            NodeOptions::default(),
        )
        .unwrap();

    // QoS 설정
    let qos = ros2::QosPolicyBuilder::new()
        .history(policy::History::KeepLast { depth: 10 })
        .reliability(policy::Reliability::Reliable {
            max_blocking_time: ros2::Duration::from_millis(100),
        })
        .durability(policy::Durability::Volatile)
        .build();

    // 토픽 및 게시자 생성
    let topic = node
        .create_topic(
            &Name::new("/turtle1", "cmd_vel").unwrap(),
            MessageTypeName::new("geometry_msgs", "Twist"),
            &qos,
        )
        .unwrap();
    let publisher = node.create_publisher(&topic, None).unwrap();

    let mut loop_count = 0;
    let mut ticker = interval(Duration::from_millis(10));

    loop {
        ticker.tick().await; // Wait for the next tick
        publish_twist(&publisher, &mut loop_count).await?;
    }
}
