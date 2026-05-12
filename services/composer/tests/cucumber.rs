mod steps;

use cucumber::World;
use futures::FutureExt;

#[tokio::main]
async fn main() {
    steps::ComposerWorld::cucumber()
        .max_concurrent_scenarios(1) // Run scenarios sequentially to avoid state conflicts with configs and resets
        .before(|_feature, _rule, scenario, world| {
            async move {
                world.cleanup().await;
            }
            .boxed_local()
        })
        .after(|_feature, _rule, scenario, _scenario_finished, optional_world| {
            async move {
                if let Some(world) = optional_world {
                    //world.cleanup().await;
                }
            }
            .boxed_local()
        })
        .run("tests/features")
        .await;
}
