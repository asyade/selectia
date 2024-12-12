pub use theater_core::{error, service, context, dispatcher};
pub use theater_derive as derive;

pub mod prelude {
    pub use theater_derive::*;
    pub use theater_core::prelude::*;
}
