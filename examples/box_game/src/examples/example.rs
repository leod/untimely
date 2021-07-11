use std::collections::BTreeMap;

use malen::draw::plot::Plot;
use untimely::{PlayerId, LocalTimeDelta};

pub struct UpdateParams<'a, E: Example> {
    pub client_inbox: &'a BTreeMap<PlayerId, Vec<E::ClientMsg>>,
    pub server_inbox: &'a [E::ServerMsg],
    pub game_input: &'a GameInput,
    pub dt: LocalTimeDelta,

    pub client_outbox: &'a mut Vec<(PlayerId, E::ClientMsg)>,
    pub server_outbox: &'a mut Vec<E::ClientMsg>,
}

pub trait Example: Default + Clone {
    type ClientMsg: Clone;
    type ServerMsg: Clone;

    fn update(&mut self, p: &UpdateParams<E>);
    fn games(&self) -> Vec<(String, Game)>;
    fn plot(&self) -> Option<Plot>;
}

pub trait ExampleRunner {

}
