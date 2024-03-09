#![no_std]

#[derive(Debug)]
pub enum ServerState {
    Healthy,
    Unhealthy,
}

#[derive(Debug)]
pub struct Server {
    mac_address: [u8; 6],
    ip_address: u32,
    state: ServerState,
}

impl Server {
    pub fn new(mac_address: [u8; 6], ip_address: u32) -> Self {
        Server {
            mac_address,
            ip_address,
            state: ServerState::Healthy,
        }
    }

    pub fn get_mac_address(&self) -> [u8; 6] {
        self.mac_address
    }

    pub fn get_ip_address(&self) -> u32 {
        self.ip_address
    }

    pub fn is_healthy(&self) -> bool {
        match self.state {
            ServerState::Healthy => true,
            ServerState::Unhealthy => false,
        }
    }

    pub fn toggle_health(&mut self) {
        match self.state {
            ServerState::Healthy => self.state = ServerState::Unhealthy,
            ServerState::Unhealthy => self.state = ServerState::Healthy,
        }
    }
}
