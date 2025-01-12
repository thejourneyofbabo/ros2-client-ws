use ros2_client::{
    ros2, ros2::policy, Context, MessageTypeName, Name, NodeName, NodeOptions, Publisher,
};
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

/// Structure for controlling the movement of a turtle using a ROS 2 publisher.
struct MovingTurtle {
    publisher: Publisher<Twist>,
    loop_count: i32,
}

impl MovingTurtle {
    /// Creates a new MovingTurtle instance.
    fn new(node: &mut ros2_client::Node) -> Self {
        let qos = ros2::QosPolicyBuilder::new()
            .history(policy::History::KeepLast { depth: 10 })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: ros2::Duration::from_millis(100),
            })
            .durability(policy::Durability::Volatile)
            .build();

        let topic = node
            .create_topic(
                &Name::new("/turtle1", "cmd_vel").unwrap(),
                MessageTypeName::new("geometry_msgs", "Twist"),
                &qos,
            )
            .expect("Failed to create topic");

        let publisher = node
            .create_publisher(&topic, None)
            .expect("Failed to create publisher");

        MovingTurtle {
            publisher,
            loop_count: 0,
        }
    }

    /// Updates the movement command based on the loop count and publishes it.
    async fn timer_callback(&mut self) {
        let mut msg = Twist::new();

        // Determine movement pattern based on the loop count
        (msg.linear.x, msg.angular.z) = match self.loop_count {
            0..=99 => (2.0, 0.0),    // Move forward
            100..=199 => (0.0, 1.0), // Rotate
            _ => {
                self.loop_count = 0; // Reset loop count
                (0.0, 0.0) // Stop
            }
        };

        self.loop_count += 1;

        // Publish the message
        if let Err(e) = self.publisher.async_publish(msg).await {
            eprintln!("Failed to publish message: {:?}", e);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a ROS 2 context
    let context = Context::new()?;

    // Create a new ROS 2 node
    let mut node = context
        .new_node(
            NodeName::new("/ros2_demo", "moving_turtle").unwrap(),
            NodeOptions::default(),
        )
        .unwrap();

    let mut moving_turtle = MovingTurtle::new(&mut node);

    // Set up a timer interval for callbacks
    let mut ticker = interval(Duration::from_millis(10));

    // Main loop to handle the node and callbacks
    loop {
        ticker.tick().await;
        moving_turtle.timer_callback().await;
    }
}
