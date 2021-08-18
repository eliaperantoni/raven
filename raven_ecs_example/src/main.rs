use serde::{Serialize, Deserialize};
use raven_ecs::{Component, World, Query};

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

    let serialized = serde_json::to_string_pretty(&w).unwrap();
    println!("{}", serialized);

    let w = serde_json::from_str::<World>(&serialized).unwrap();

    for (_, (c,)) in <(MyComponent,)>::query_shallow(&w) {
        println!("s => {}, i => {}", c.s, c.i);
    }
}
