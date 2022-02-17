//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

pub(crate) mod q;
pub(crate) mod ticket;

use q::{Date, Symbol};
use ticket::{processed, Class, Processed, ReservationFailure, TicketInfo};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

impl Processed {
    /// Build successful message for processed request.
    pub(crate) fn success(name: String, seats: Vec<Symbol>, date: Option<Date>) -> Self {
        Processed {
            result: Some(processed::Result::Ticket(TicketInfo { name, seats, date })),
        }
    }

    /// Build failure message for processed request.
    pub(crate) fn failure(available: usize, requested: i32) -> Self {
        Processed {
            result: Some(processed::Result::Failure(ReservationFailure {
                message: format!(
                    "we cannnot reserve {} seats. Only {} available",
                    requested, available
                ),
            })),
        }
    }
}

impl ToString for Class {
    fn to_string(&self) -> String {
        match self {
            Self::NoPreference => "no_preference",
            Self::Stand => "stand",
            Self::Arena => "arena",
            Self::Vip => "vip",
        }
        .to_string()
    }
}
