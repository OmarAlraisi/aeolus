#[derive(Debug, Clone, Copy)]
pub enum ServerState {
    Healthy,
    Unhealthy,
}

#[derive(Debug, Clone, Copy)]
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

    pub fn is_healthy(&self) -> bool {
        match self.state {
            ServerState::Healthy => true,
            ServerState::Unhealthy => false,
        }
    }

    pub async fn run_health_check(&mut self) {
        println!("{}", self.ip_address);
        match self.state {
            ServerState::Healthy => self.state = ServerState::Unhealthy,
            ServerState::Unhealthy => self.state = ServerState::Healthy,
        }
    }
}
