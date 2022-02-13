use thalo::{
    aggregate::Aggregate, event::AggregateEventEnvelope, event_store::EventStore,
    tests_cfg::bank_account::BankAccount,
};
use thalo_eventstoredb::ESDBEventStore;
use tokio::sync::broadcast::Sender;
use tonic::{Request, Status};

tonic::include_proto!("bank_account");

#[derive(Clone, Debug)]
pub struct BankAccountService {
    pub(crate) event_store: ESDBEventStore,
    event_stream: Sender<AggregateEventEnvelope<BankAccount>>,
}

impl BankAccountService {
    #[allow(unused)]
    pub fn new(event_stream: Sender<AggregateEventEnvelope<BankAccount>>, event_store: ESDBEventStore) -> Self {
        BankAccountService {
            event_store,
            event_stream,
        }
    }
}

#[tonic::async_trait]
impl bank_account_server::BankAccount for BankAccountService {
    async fn open_account(
        &self,
        request: Request<OpenAccountCommand>,
    ) -> Result<tonic::Response<Response>, Status> {
        let command = request.into_inner();
        println!("OPEN ACCOUNT COMMAND {:?}", command);

        let exists = self
            .event_store
            .load_aggregate_sequence::<BankAccount>(&command.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))
            .is_some();

        println!("OPEN ACCOUNT AGG SEQUENCE {:?}", aggregate_sequence);

        if exists {
            println!("ACCOUNT Thinks it exists - Err!");
            return Err(Status::already_exists("account already opened"));
        }

        let (bank_account, event) = BankAccount::open_account(command.id, command.initial_balance)?;
        println!("GOT NEW BANK ACCOUNT Event to dispatch! {:?}", event);
        let events = &[event];

        let event_ids = self
            .event_store
            .save_events::<BankAccount>(bank_account.id(), events)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        println!("OPEN ACCOUNT REQUEST finished save_events without err - {:?}", event_ids);

        broadcast_events(&self.event_store, &self.event_stream, &event_ids).await?;

        let events_json = serde_json::to_string(events)
            .map_err(|_| Status::internal("failed to serialize events"))?;

        Ok(tonic::Response::new(Response {
            events: events_json,
        }))
    }

    async fn deposit_funds(
        &self,
        request: Request<DepositFundsCommand>,
    ) -> Result<tonic::Response<Response>, Status> {
        let command = request.into_inner();

        let bank_account: BankAccount = self
            .event_store
            .load_aggregate(command.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("account does not exist"))?;

        let event = bank_account.deposit_funds(command.amount)?;
        let events = &[event];

        let event_ids = self
            .event_store
            .save_events::<BankAccount>(bank_account.id(), events)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        broadcast_events(&self.event_store, &self.event_stream, &event_ids).await?;

        let events_json = serde_json::to_string(events)
            .map_err(|_| Status::internal("failed to serialize events"))?;

        Ok(tonic::Response::new(Response {
            events: events_json,
        }))
    }

    async fn withdraw_funds(
        &self,
        request: Request<WithdrawFundsCommand>,
    ) -> Result<tonic::Response<Response>, Status> {
        let command = request.into_inner();

        let bank_account: BankAccount = self
            .event_store
            .load_aggregate(command.id)
            .await
            .map_err(|err| Status::internal(err.to_string()))?
            .ok_or_else(|| Status::not_found("account does not exist"))?;

        let event = bank_account.withdraw_funds(command.amount)?;
        let events = &[event];

        let event_ids = self
            .event_store
            .save_events::<BankAccount>(bank_account.id(), events)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        broadcast_events(&self.event_store, &self.event_stream, &event_ids).await?;

        let events_json = serde_json::to_string(events)
            .map_err(|_| Status::internal("failed to serialize events"))?;

        Ok(tonic::Response::new(Response {
            events: events_json,
        }))
    }
}

async fn broadcast_events(
    event_store: &ESDBEventStore,
    event_stream: &Sender<AggregateEventEnvelope<BankAccount>>,
    event_ids: &[usize],
) -> Result<(), Status> {
    println!("LOADING EVENT ENVELOPES FOR BROADCAST");
    let event_envelopes = event_store
        .load_events_by_id::<BankAccount>(event_ids)
        .await
        .map_err(|err| Status::internal(err.to_string()))?;
    println!("LOADING EVENT RESULT {:?}", event_envelopes);

    for event_envelope in event_envelopes {
        event_stream
            .send(event_envelope)
            .map_err(|err| Status::internal(err.to_string()))?;
    }

    Ok(())
}
