use std::{vec, fmt::Debug};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eventstore::{
    AppendToStreamOptions, Client, EventData, ExpectedRevision, ReadStreamOptions,
    StreamPosition,
};
use futures::TryFutureExt;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thalo::{
    aggregate::{Aggregate, TypeId},
    event::{AggregateEventEnvelope, EventType},
    event_store::EventStore,
};
use uuid::Uuid;

use crate::Error;

#[derive(Serialize, Deserialize, Debug)]
struct ESDBEventPayload {
    created_at: DateTime<Utc>,
    aggregate_type: String,
    aggregate_id: String,
    event_data: serde_json::Value,
}

impl ESDBEventPayload {
    fn event_envelope<A>(&self, id: usize) -> Result<AggregateEventEnvelope<A>, Error>
    where
        A: Aggregate,
        <A as Aggregate>::Event: DeserializeOwned,
    {
        Ok(AggregateEventEnvelope::<A> {
            id,
            created_at: self.created_at.into(),
            aggregate_type: self.aggregate_type.to_string(),
            aggregate_id: self.aggregate_id.clone(),
            sequence: id.clone(),
            event: serde_json::from_value(self.event_data.clone())
                .map_err(Error::DeserializeEvent)?,
        })
    }
}

#[derive(Clone)]
pub struct ESDBEventStore {
    pub client: Client,
}

impl ESDBEventStore {
    pub fn new(client: Client) -> Self {
        ESDBEventStore { client }
    }

    fn stream_id<A>(&self, id: Option<&<A as Aggregate>::ID>) -> String
    where
        A: Aggregate,
    {
        let mut stream = <A as TypeId>::type_id().to_owned();

        if let Some(id_str) = id {
            stream.push_str(&String::from("-"));
            stream.push_str(&id_str.to_string());
        }

        stream
    }
}

#[async_trait]
impl EventStore for ESDBEventStore {
    type Error = Error;

    async fn load_events<A>(
        &self,
        id: Option<&<A as Aggregate>::ID>,
    ) -> Result<Vec<AggregateEventEnvelope<A>>, Self::Error>
    where
        A: Aggregate,
        <A as Aggregate>::Event: DeserializeOwned,
    {
        let options = ReadStreamOptions::default()
            .position(StreamPosition::Start)
            .forwards();

        let mut result = self
            .client
            .read_stream(self.stream_id::<A>(id), &options)
            .await
            .map_err(Error::ReadStreamError)?;

        let mut rv: Vec<AggregateEventEnvelope<A>> = vec![];

        while let Some(event) = result.next().await? {
            let event_data = event.get_original_event();

            // TODO: - can we try event eventlope ids as uuid in addition to usize?
            // let uuid = event_data.id.clone();
            let event_payload = event_data
                .as_json::<ESDBEventPayload>()
                .map_err(Error::DeserializeEvent)?
                .event_envelope::<A>(event_data.revision as usize)?;

            rv.push(event_payload);
        }

        Ok(rv)
    }

    async fn load_events_by_id<A>(
        &self,
        ids: &[usize],
    ) -> Result<Vec<AggregateEventEnvelope<A>>, Self::Error>
    where
        A: Aggregate,
        <A as Aggregate>::Event: DeserializeOwned,
    {
        let options = ReadStreamOptions::default()
            .position(StreamPosition::Start)
            .forwards();

        let mut result = self
            .client
            .read_stream(self.stream_id::<A>(None), &options)
            .await
            .map_err(Error::ReadStreamError)?;

        let mut rv: Vec<AggregateEventEnvelope<A>> = vec![];

        while let Some(event) = result.next().await? {
            let event_data = event.get_original_event();

            // TODO: - can we try event eventlope ids as uuid in addition to usize?
            // let uuid = event_data.id.clone();
            let sequence = usize::try_from(event_data.revision).unwrap();
            if ids.contains(&sequence) {
                let event_payload = event_data
                    .as_json::<ESDBEventPayload>()
                    .map_err(Error::DeserializeEvent)?
                    .event_envelope::<A>(sequence)?;

                rv.push(event_payload);
            }
        }

        Ok(rv)
    }

    async fn load_aggregate_sequence<A>(
        &self,
        id: &<A as Aggregate>::ID,
    ) -> Result<Option<usize>, Self::Error>
    where
        A: Aggregate,
    {
        let options = ReadStreamOptions::default()
            .position(StreamPosition::End)
            .max_count(1);

        println!("CALLED load_aggregate_sequence with type {:?} and id {:?}", <A as TypeId>::type_id().to_string(), id.to_string());
        let mut stream = self
            .client
            .read_stream(self.stream_id::<A>(Some(id)), &options)
            .await
            .map_err(Error::ReadStreamError)?;
        println!("RESOLVED load_aggregate_sequence with Result {:?}", stream);

        while let Some(event) = stream.next().await? {
            let event_data = event.get_original_event();
            return Ok(Some(event_data.revision as usize));
        }

        Ok(None)
    }

    async fn save_events<A>(
        &self,
        id: &<A as Aggregate>::ID,
        events: &[<A as Aggregate>::Event],
    ) -> Result<Vec<usize>, Self::Error>
    where
        A: Aggregate,
        <A as Aggregate>::Event: serde::Serialize,
    {

        print!("CALLED SAVE_EVENTS {:?}", id.to_string());
        if events.is_empty() {
            return Ok(vec![]);
        }

        let revision = self.load_aggregate_sequence::<A>(id).await?.unwrap_or(0);
        print!("SAVE_EVENTS REVISION {:?}", revision);
        let mut payload: Vec<EventData> = vec![];

        for event in events.iter() {
            let event_data_payload = ESDBEventPayload {
                created_at: Utc::now(),
                aggregate_type: <A as TypeId>::type_id().to_string(),
                aggregate_id: id.to_string(),
                event_data: serde_json::to_value(event).map_err(Error::SerializeEvent)?,
            };

            let event_data = EventData::json(event.event_type(), &event_data_payload)
                .map_err(Error::SerializeEventDataPayload)?
                .id(Uuid::new_v4());

            payload.push(event_data);
        }

        let options = AppendToStreamOptions::default()
            .expected_revision(ExpectedRevision::Exact(revision as u64));
        let _ = self
            .client
            .append_to_stream(&self.stream_id::<A>(Some(id)), &options, payload)
            .map_err(|err| Error::WriteStreamError(revision as usize, err))
            .await?;

        Ok((revision..events.len() - 1).collect())
    }
}

impl Debug for ESDBEventStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ESDBEventStore")
            .field("client", &"eventstore::Client").finish()
    }
}

#[cfg(feature = "debug")]
impl ESDBEventStore {
    pub async fn print(&self) {
        println!("Calling Read All");
        let stream_res = self
            .client
            .read_all(&Default::default())
            .await;

        println!("Read Completed {:?}", stream_res);

        let mut events: Vec<(Uuid, u64, ESDBEventPayload)> = vec![];
        if let Ok(mut stream) = stream_res {
            while let Some(event) = stream.next().await.unwrap() {
                let event_data = event.get_original_event();

                if !event_data.event_type.starts_with("$") {
                    continue;
                }

                // TODO: Make ESDBEventPayload handle all this
                // and implement fns for RecordedEvent -> ESDBEventPayload, ESDBEventPayload -> AggregateEnvelope, ESDBPayload -> eventstore::EventData
                // make base struct handle id and revision seperately - update event trait for different id types (str, u64, uuid) etc
                events.push((
                    event_data.id,
                    event_data.revision,
                    event_data.as_json::<ESDBEventPayload>().unwrap(),
                ));
            }
        }

        let mut table = prettytable::Table::new();
        table.set_titles(
            [
                "ID",
                "Created At",
                "Aggregate Type",
                "Aggregate ID",
                "Sequence",
                "Event Data",
            ]
            .into(),
        );

        if events.is_empty() {
            table.add_row(["", "", "", "", "", ""].into());
        } else {
            for (uuid, revision, event) in events.iter() {
                table.add_row(
                    [
                        uuid.to_string(),
                        event.created_at.to_string(),
                        event.aggregate_type.to_string(),
                        event.aggregate_id.clone(),
                        revision.to_string(),
                        serde_json::to_string(&event.event_data).unwrap(),
                    ]
                    .into(),
                );
            }
        }
    }
}
