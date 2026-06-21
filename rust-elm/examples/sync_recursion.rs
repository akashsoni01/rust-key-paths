//! **Sync recursion** — no runtime; a driver calls `update` synchronously in a loop.
//!
//! Compare with [`recursion`]: async effect chain where each page schedules the next fetch.
//!
//! ```bash
//! cargo run -p rust-elm --example sync_recursion --release
//! ```

const PAGES: &[&str] = &["alpha", "beta", "gamma", "delta"];

#[derive(Default, Debug, PartialEq, Eq)]
struct App {
    items: Vec<String>,
    pages_loaded: u32,
    done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    Start,
    Step(u32),
}

fn init() -> App {
    App::default()
}

fn update(app: &mut App, action: Action) {
    match action {
        Action::Start => {
            app.items.clear();
            app.pages_loaded = 0;
            app.done = false;
        }
        Action::Step(page) => {
            if (page as usize) < PAGES.len() {
                app.items.push(PAGES[page as usize].to_string());
                app.pages_loaded = page + 1;
            }
            app.done = (page as usize + 1) >= PAGES.len();
        }
    }
}

/// Tail-call style pagination without effects: the driver loops `Step(n)`.
fn drive_sync_pagination() -> App {
    let mut app = init();
    update(&mut app, Action::Start);
    for page in 0..PAGES.len() {
        update(&mut app, Action::Step(page as u32));
    }
    app
}

/// Pure functional recursion on a tree (functional core, no actions).
#[derive(Debug)]
enum Node {
    Leaf(u32),
    Branch(Vec<Node>),
}

fn sum_tree(node: &Node) -> u32 {
    match node {
        Node::Leaf(n) => *n,
        Node::Branch(children) => children.iter().map(sum_tree).sum(),
    }
}

fn main() {
    let tree = Node::Branch(vec![
        Node::Leaf(1),
        Node::Branch(vec![Node::Leaf(2), Node::Leaf(3)]),
    ]);
    assert_eq!(sum_tree(&tree), 6);
    println!("sum_tree: {}", sum_tree(&tree));

    let app = drive_sync_pagination();
    println!("pages loaded: {}", app.pages_loaded);
    println!("items: {:?}", app.items);
    assert!(app.done);
    assert_eq!(app.items, ["alpha", "beta", "gamma", "delta"]);
    assert_eq!(app.pages_loaded, PAGES.len() as u32);

    println!("sync_recursion example OK");
}
