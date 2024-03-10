use futures::{future, StreamExt};
use log::warn;
use std::net::{IpAddr, Ipv4Addr};
use tokio_icmp_echo::Pinger;

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

    /// returns true if the state change
    pub async fn run_health_check(&mut self) -> bool {
        let mut healthy = false;
        Pinger::new()
            .await
            .unwrap()
            .chain(IpAddr::V4(Ipv4Addr::from(self.ip_address)))
            .stream()
            .take(1)
            .for_each(|echo| {
                match echo {
                    Ok(Some(_)) => healthy = true,
                    Ok(None) => {}
                    Err(e) => eprintln!("Health Check Error: {:?}", e),
                }
                future::ready(())
            })
            .await;

        match self.state {
            ServerState::Healthy => match healthy {
                true => false,
                false => {
                    self.state = ServerState::Unhealthy;
                    warn!(
                        "Server with IP {} is offline",
                        Ipv4Addr::from(self.ip_address)
                    );
                    true
                }
            },
            ServerState::Unhealthy => match healthy {
                true => {
                    self.state = ServerState::Healthy;
                    warn!(
                        "Server with IP {} is online",
                        Ipv4Addr::from(self.ip_address)
                    );
                    true
                }
                false => false,
            },
        }
    }
}
