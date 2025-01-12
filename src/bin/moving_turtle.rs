use ros2_client::{ros2, ros2::policy, Context, MessageTypeName, Name, NodeName, NodeOptions};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};

/// Represents a 3D vector with x, y, and z components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Creates a new Vector3 with all components set to 0.0.
    fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

/// Represents a velocity command for a robot, consisting of linear and angular velocities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Twist {
    pub linear: Vector3,
    pub angular: Vector3,
}

impl Twist {
    /// Creates a new Twist with both linear and angular velocities set to zero.
    fn new() -> Self {
        Self {
            linear: Vector3::zero(),
            angular: Vector3::zero(),
        }
    }
}

/// Publishes a Twist message to the specified topic.
///
/// This function publishes a Twist message with different velocities based on the `loop_count`.
///
/// * `publisher`: A reference to the ROS 2 publisher.
/// * `loop_count`: A mutable reference to the loop count.
///
/// Returns a Result that indicates success or an error.
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
    // Create a new ROS 2 context
    let context = Context::new().unwrap();

    // Create a new ROS 2 node
    let mut node = context
        .new_node(
            NodeName::new("/ros2_demo", "moving_turtle").unwrap(),
            NodeOptions::default(),
        )
        .unwrap();

    // Configure QoS policy
    let qos = ros2::QosPolicyBuilder::new()
        .history(policy::History::KeepLast { depth: 10 })
        .reliability(policy::Reliability::Reliable {
            max_blocking_time: ros2::Duration::from_millis(100),
        })
        .durability(policy::Durability::Volatile)
        .build();

    // Create a ROS 2 topic
    let topic = node
        .create_topic(
            &Name::new("/turtle1", "cmd_vel").unwrap(),
            MessageTypeName::new("geometry_msgs", "Twist"),
            &qos,
        )
        .unwrap();

    // Create a ROS 2 publisher
    let publisher = node.create_publisher(&topic, None).unwrap();

    // Initialize loop count and create a timer
    let mut loop_count = 0;
    let mut ticker = interval(Duration::from_millis(10));

    // Publish Twist messages periodically
    loop {
        ticker.tick().await; // Wait for the next tick
        publish_twist(&publisher, &mut loop_count).await?;
    }
}
