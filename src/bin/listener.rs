//use log::error;
use core::cmp::min;

use mio::{Events, Poll, PollOpt, Ready, Token};
use ros2_client::{Context, MessageTypeName, Name, Node, NodeName, NodeOptions};
use rustdds::{
    policy::{self, Deadline, Lifespan},
    Duration, QosPolicies, QosPolicyBuilder,
};

// Simple demo program.
// Test this against ROS2 "talker" demo node.
// https://github.com/ros2/demos/blob/humble/demo_nodes_py/demo_nodes_py/topics/talker.py

fn main() {
    // Here is a fixed path, so this example must be started from
    // package main directory
    //log4rs::init_file("examples/listener/log4rs.yaml", Default::default()).unwrap();

    let mut node = create_node();
    let topic_qos = create_qos();

    let chatter_topic = node
        .create_topic(
            &Name::new("/", "topic").unwrap(),
            MessageTypeName::new("std_msgs", "String"),
            &topic_qos,
        )
        .unwrap();
    let chatter_subscription = node
        .create_subscription::<String>(&chatter_topic, None)
        .unwrap();

    let poll = Poll::new().unwrap();

    poll.register(
        &chatter_subscription,
        Token(1),
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    let mut events = Events::with_capacity(8);

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                Token(1) => match chatter_subscription.take() {
                    Ok(Some((message, _message_info))) => {
                        let l = message.len();
                        println!("message len={} : {:?}", l, &message[..min(l, 50)]);
                    }
                    Ok(None) => println!("No message?!"),
                    Err(e) => {
                        println!(">>> error with response handling, e: {:?}", e)
                    }
                },
                _ => println!(">>> Unknown poll token {:?}", event.token()),
            } // match
        } // for
    } // loop
} // main

fn create_qos() -> QosPolicies {
    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .history(policy::History::KeepLast { depth: 10 })
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: Duration::from_millis(100),
            })
            .durability(policy::Durability::Volatile)
            .deadline(Deadline(Duration::INFINITE))
            .lifespan(Lifespan {
                duration: Duration::INFINITE,
            })
            .liveliness(policy::Liveliness::Automatic {
                lease_duration: Duration::INFINITE,
            })
            .build()
    };
    service_qos
}

fn create_node() -> Node {
    let context = Context::new().unwrap();
    context
        .new_node(
            NodeName::new("/rustdds", "rustdds_listener").unwrap(),
            NodeOptions::new().enable_rosout(true),
        )
        .unwrap()
}
