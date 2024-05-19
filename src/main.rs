use anyhow::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    no_internal_kbd::Context::initialize()?.start().await
}
