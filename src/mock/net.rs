use std::collections::BTreeMap;

use crate::{LocalClock, LocalTime, PlayerId};

use super::{MockChannel, MockChannelParams};

#[derive(Clone, Debug)]
pub struct MockSocketParams {
    pub server_out: MockChannelParams,
    pub client_out: MockChannelParams,
}

impl MockSocketParams {
    pub fn perfect() -> Self {
        Self {
            server_out: MockChannelParams::perfect(),
            client_out: MockChannelParams::perfect(),
        }
    }
}

#[derive(Clone)]
pub struct MockSocket<S, C> {
    params: MockSocketParams,
    server_out: MockChannel<S>,
    client_out: MockChannel<C>,
}

#[derive(Clone)]
pub struct MockNet<S, C> {
    clock: LocalClock,
    sockets: BTreeMap<PlayerId, MockSocket<S, C>>,
}

impl<S, C> MockNet<S, C> {
    pub fn new(players: &[PlayerId], clock: LocalClock) -> Self {
        let sockets = players
            .iter()
            .map(|player| {
                (
                    *player,
                    MockSocket {
                        params: MockSocketParams::perfect(),
                        server_out: MockChannel::new(clock.clone()),
                        client_out: MockChannel::new(clock.clone()),
                    },
                )
            })
            .collect();

        MockNet { clock, sockets }
    }

    fn socket_mut(&mut self, player: PlayerId) -> &mut MockSocket<S, C> {
        self.sockets.get_mut(&player).expect("Unknown PlayerId")
    }

    pub fn set_params(&mut self, player: PlayerId, params: MockSocketParams) {
        self.socket_mut(player).params = params;
    }

    pub fn send_to_server(&mut self, sender: PlayerId, message: C) {
        let socket = self.socket_mut(sender);
        socket.client_out.send(&socket.params.client_out, message);
    }

    pub fn send_to_client(&mut self, receiver: PlayerId, message: S) {
        let socket = self.socket_mut(receiver);
        socket.server_out.send(&socket.params.server_out, message);
    }

    pub fn receive_client(&mut self, receiver: PlayerId) -> Vec<(LocalTime, S)> {
        let socket = self.socket_mut(receiver);
        let mut messages = Vec::new();
        while let Some(message) = socket.server_out.receive() {
            messages.push(message);
        }

        messages
    }

    pub fn receive_server(&mut self) -> Vec<(LocalTime, PlayerId, C)> {
        let mut messages = Vec::new();
        for (sender, socket) in self.sockets.iter_mut() {
            while let Some((receive_time, message)) = socket.client_out.receive() {
                messages.push((receive_time, *sender, message));
            }
        }

        messages.sort_by(|(time1, _, _), (time2, _, _)| time1.partial_cmp(time2).unwrap());
        messages
    }
}
