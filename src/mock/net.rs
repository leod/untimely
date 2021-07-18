use std::collections::BTreeMap;

use crate::{PlayerId, LocalTime};

use super::{MockChannel, MockChannelParams};

#[derive(Clone)]
pub struct MockSocket<S, C> {
    server_out_params: MockChannelParams,
    client_out_params: MockChannelParams,
    server_out: MockChannel<S>,
    client_out: MockChannel<C>,
}

#[derive(Clone)]
pub struct MockNet<S, C> {
    time: LocalTime,
    sockets: BTreeMap<PlayerId, MockSocket<S, C>>,
}

impl<S, C> MockNet<S, C> {
    pub fn new(players: &[PlayerId]) -> Self {
        let sockets = players.iter().map(|player| (*player, MockSocket {
            server_out_params: MockChannelParams::PERFECT,
            client_out_params: MockChannelParams::PERFECT,
            server_out: MockChannel::new(),
            client_out: MockChannel::new(),
        }))
        .collect();

        MockNet {
            time: LocalTime::zero(),
            sockets,
        } 
    }

    pub fn socket_mut(&mut self, player: PlayerId) -> &mut MockSocket<S, C> {
        self.sockets.get_mut(&player).expect("Unknown PlayerId")
    }

    pub fn send_to_server(&mut self, sender: PlayerId, message: C) {
        let time = self.time;
        let socket = self.socket_mut(sender);
        socket.client_out.send(&socket.client_out_params, time, message);
    }

    pub fn send_to_client(&mut self, receiver: PlayerId, message: S) {
        let time = self.time;
        let socket = self.socket_mut(receiver);
        socket.server_out.send(&socket.server_out_params, time, message);
    }

    pub fn receive_client(&mut self, receiver: PlayerId) -> Vec<(LocalTime, S)> {
        let time = self.time;
        let socket = self.socket_mut(receiver);
        let mut messages = Vec::new();
        while let Some(message) = socket.server_out.receive(time) {
            messages.push(message);
        }

        messages
    }

    pub fn receive_server(&mut self) -> Vec<(LocalTime, PlayerId, C)> {
        let mut messages = Vec::new();
        for (sender, socket) in self.sockets.iter_mut() {
            while let Some((receive_time, message)) = socket.client_out.receive(self.time) {
                messages.push((receive_time, *sender, message));
            }
        }

        messages.sort_by(|(time1, _, _), (time2, _, _)| time1.partial_cmp(time2).unwrap());

        messages
    }
}