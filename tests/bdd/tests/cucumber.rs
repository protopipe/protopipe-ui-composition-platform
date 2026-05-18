mod steps;

use cucumber::World;
use futures::FutureExt;
use std::env;

#[tokio::main]
async fn main() {
    let features = env::var("CUCUMBER_FEATURES").unwrap_or_else(|_| "features".to_string());

    steps::ComposerWorld::cucumber()
        .max_concurrent_scenarios(1) // Run scenarios sequentially to avoid state conflicts with configs and resets
        .before(|_feature, _rule, _scenario, world| {
            async move {
                world.cleanup().await;
            }
            .boxed_local()
        })
        .after(
            |_feature, _rule, _scenario, _scenario_finished, _optional_world| {
                async move {
                    // Cleanup runs before each scenario so failed scenario state
                    // remains available in logs for diagnosis.
                }
                .boxed_local()
            },
        )
        .run(features)
        .await;
}
