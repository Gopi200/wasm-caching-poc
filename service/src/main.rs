use std::{cell::Cell, rc::Rc};

use shared::{Action, Direction};

#[wstd::main]
async fn main() -> wstd::io::Result<()> {
    let counter: Rc<Cell<i32>> = Rc::new(Cell::default());
    wash_service_helpers::run_tcp_server(8080, async move |action: Action| {
        process_message(&counter, action).await
    })
    .await
}

async fn process_message(counter: &Cell<i32>, action: Action) -> i32 {
    match action {
        Action::Get => get_counter_value(counter),
        Action::Update(direction) => update_counter_value(counter, direction),
    }
}

fn get_counter_value(counter: &Cell<i32>) -> i32 {
    counter.get()
}

fn update_counter_value(counter: &Cell<i32>, direction: Direction) -> i32 {
    let offset = match direction {
        Direction::Increment => 1,
        Direction::Decrement => -1,
    };

    let counter_value = counter.get() + offset;
    counter.set(counter_value);
    counter_value
}
