use std::fmt;

use async_trait::async_trait;

use crate::{Aggregate, AggregateType, Error};

/// An `Event` is something that happened as a result to a [`Command`].
///
/// Typically a type which implements this trait would be an enum.
///
/// # Examples
///
/// ```no_run
/// #[derive(Event)]
/// enum BankAccountEvent {
///     AccountOpened(AccountOpenedEvent),
///     FundsDeposited(FundsDepositedEvent),
///     FundsWithdrawn(FundsWithdrawnEvent),
/// }
/// ```
pub trait Event:
    serde::de::DeserializeOwned + serde::ser::Serialize + Clone + fmt::Debug + PartialEq + Send + Sync
{
    /// The event varaint identifier type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn event_type(&self) -> &'static str {
    ///     match self {
    ///         BankAccountEvent::AccountOpened(_) => "AccountOpened",
    ///         BankAccountEvent::FundsDeposited(_) => "FundsDeposited",
    ///         BankAccountEvent::FundsWithdrawn(_) => "FundsWithdrawn",
    ///     }
    /// }
    /// ```
    fn event_type(&self) -> &'static str;
}

pub trait CombinedEvent:
    serde::de::DeserializeOwned + serde::ser::Serialize + Clone + fmt::Debug + PartialEq + Send + Sync
{
    fn aggregate_types() -> Vec<&'static str>;
}

// impl<A: Aggregate> CombinedEvent for <A as Aggregate>::Event {
//     fn aggregate_types() -> Vec<&'static str> {
//         vec![<E as Event>::Aggregate::aggregate_type()]
//     }
// }

pub trait EventView<E> {
    fn view(&self) -> Result<&E, Error>;

    fn view_opt(&self) -> Option<&E>;
}

/// EventHandler must run once only when multiple nodes of the
/// application are running at the same time (via locks in the database).
///
/// They keep track of their latest sequence and only process events that
/// have not yet been processed yet.
#[async_trait]
pub trait EventHandler {
    type Event: CombinedEvent + Send + Sync;
    type View: Send + Sync;

    /// Handle an event and return an updated view.
    async fn handle(
        &self,
        view: Self::View,
        event: Self::Event,
        event_id: i64,
        event_sequence: i64,
    ) -> Result<Self::View, Error>;
}
