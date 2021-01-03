mod tick_buffer;
pub mod time;

pub mod util;

pub use tick_buffer::{ReceiverTickBuffer, SenderTickBuffer};
pub use time::{
    GameTime, GameTimeDelta, LocalTime, LocalTimeDelta, TickNum, TickNumDelta, TimeMapping,
    TimeMappingConfig,
};
