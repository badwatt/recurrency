//! Binary entry point. All application code lives in the `recurrency_frontend`
//! library so tests and the entry point compile from a single crate.

fn main() {
    recurrency_frontend::launch();
}
