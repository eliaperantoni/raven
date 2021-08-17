use raven_ecs::{Component, world::World};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Component)]
struct MyComponent {
    s: String,
    i: i32,
}

fn main() {
    let mut w = World::default();

    let e = w.create();
    w.attach(e, MyComponent {
        s: "Hello, World!".to_string(),
        i: 256,
    });
}
