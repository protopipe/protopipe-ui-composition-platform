mod steps;

use cucumber::World;

#[tokio::main]
async fn main() {
    steps::ComposerWorld::run("tests/features").await;
}
