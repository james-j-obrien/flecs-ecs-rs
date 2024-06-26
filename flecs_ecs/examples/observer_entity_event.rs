mod common;

use common::*;

// Entity events are events that are emitted and observed for a specific entity.
// They are a thin wrapper around regular observers, which match against queries
// instead of single entities. While they work similarly under the hood, entity
// events provide a much simpler API.
//
// An entity event only needs two pieces of data:
// - The entity on which to emit the event
// - The event to emit

// An event without payload
#[derive(Component)]
struct Click;

// An event with payload
#[derive(Component)]
struct Resize {
    width: f32,
    height: f32,
}

fn main() {
    let world = World::new();

    let widget = world.new_entity_named(c"MyWidget");
    println!("widget: {:?}", widget);

    // Observe the Click event on the widget entity.
    widget.observe::<Click>(|| {
        println!("clicked!");
    });

    widget.observe_entity::<Click>(|entity| {
        println!("clicked on {:?}", entity.name());
    });

    // Observe the Resize event on the widget entity.
    widget.observe_payload(|payload: &mut Resize| {
        println!(
            "widget resized to {{ {}, {} }}!",
            payload.width, payload.height
        );
    });

    widget.observe_payload_entity(|entity, payload: &mut Resize| {
        println!(
            "{} resized to {{ {}, {} }}!",
            entity.name(),
            payload.width,
            payload.height
        );
    });

    widget.emit::<Click>();
    widget.emit_payload(&mut Resize {
        width: 100.0,
        height: 200.0,
    });

    // Output:
    //  clicked!
    //  clicked on "MyWidget"
    //  widget resized to { 100, 200 }!
    //  MyWidget resized to { 100, 200 }!
}
