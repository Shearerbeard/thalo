use api::bank_account_server::BankAccountServer;
use api::BankAccountService;
use eventstore::{Client, ClientSettings};
use futures_util::stream::StreamExt;
use thalo::tests_cfg::bank_account::{BankAccount, BankAccountProjection};
use thalo::{event::EventHandler, event_stream::EventStream};
use thalo_eventstoredb::ESDBEventStore;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tonic::transport::Server;

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("INIT");
    let addr = "[::1]:50051".parse().unwrap();

    // Event stream channel
    let settings = "esdb://localhost:2113?tls=false"
        .parse::<ClientSettings>()?;
    let client = Client::new(settings)?;
    println!("GOT CLIENT");
    let event_store = ESDBEventStore::new(client);
    println!("GOT EVENT_STORE {:?}", event_store);
    let (event_stream, event_stream_rx1) = broadcast::channel(16);

    let bank_account_service = BankAccountService::new(event_stream.clone(), event_store.clone());

    // Projection handler
    {
        let mut broadcast_stream = BroadcastStream::new(event_stream_rx1);
        let bank_account_service2 = bank_account_service.clone();
        let bank_account_projection = BankAccountProjection::default();
        println!("CALLING TABLES, DOES IT EVER RETURN?");
        print_tables(&bank_account_service2.event_store, &bank_account_projection);
        println!("IT DOES!");

        tokio::spawn(async move {
            let mut events =
                EventStream::<BankAccount>::listen_events(&mut broadcast_stream).unwrap();
            while let Some(Ok(event)) = events.next().await {
                bank_account_projection.handle(event).await.unwrap();
                print_tables(&bank_account_service2.event_store, &bank_account_projection);
            }
        });
    }

    // Accept commands through rpc
    Server::builder()
        .add_service(BankAccountServer::new(bank_account_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn print_tables(_event_store: &ESDBEventStore, bank_account_projection: &BankAccountProjection) {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Event Store");
    println!("\nBank Account Projection");
    bank_account_projection.print_bank_accounts();
}
