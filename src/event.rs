//! Provides an abstract event type which can be downcasted into a more specific one.
//!
//! # Examples
//!
//! ```
//! use xmpp::event::{Event, AbstractEvent};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct EventA;
//!
//! impl Event for EventA {}
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct EventB;
//!
//! impl Event for EventB {}
//!
//! let event_a = AbstractEvent::new(EventA);
//!
//! assert_eq!(event_a.is::<EventA>(), true);
//! assert_eq!(event_a.is::<EventB>(), false);
//!
//! assert_eq!(event_a.downcast::<EventA>(), Some(&EventA));
//! assert_eq!(event_a.downcast::<EventB>(), None);
//! ```

use std::fmt::Debug;

use std::any::Any;

/// An abstract event.
pub struct AbstractEvent {
    inner: Box<Any>,
}

impl AbstractEvent {
    /// Creates an abstract event from a concrete event.
    pub fn new<E: Event>(event: E) -> AbstractEvent {
        AbstractEvent {
            inner: Box::new(event),
        }
    }

    /// Downcasts this abstract event into a concrete event.
    pub fn downcast<E: Event + 'static>(&self) -> Option<&E> {
        self.inner.downcast_ref::<E>()
    }

    /// Checks whether this abstract event is a specific concrete event.
    pub fn is<E: Event + 'static>(&self) -> bool {
        self.inner.is::<E>()
    }
}

/// A marker trait which all events must implement.
pub trait Event: Any + Debug {}
