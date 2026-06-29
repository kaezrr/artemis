use artemis::Application;

#[tokio::main]
async fn main() -> artemis::Result<()> {
    let _ = Application::open("sqlite://data/artemis.db").await?;

    Ok(())
}
