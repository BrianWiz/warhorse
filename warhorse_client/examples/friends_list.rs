// use tracing_subscriber::FmtSubscriber;
// use warhorse_client::HorseClient;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing::subscriber::set_global_default(FmtSubscriber::default())?;

//     let _client = tokio::task::spawn_blocking(|| {
//         HorseClient::new(
//             "http://localhost:3000",
//             |friends_list| {
//                 println!("Friends list: {:?}", friends_list);
//             },
//         )
//     }).await.expect("Failed to create client");

//     tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
//     println!("Shutting down...");

//     Ok(())
// }

fn main() {
    println!("Hello, world!");
}
