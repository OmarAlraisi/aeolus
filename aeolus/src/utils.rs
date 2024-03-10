use super::server::Server;
use aya::maps::{Array, MapData};
use futures::StreamExt;
use log::info;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::{
    process,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub async fn setup_sigint_handler() -> Result<(), anyhow::Error> {
    let mut signals = Signals::new(&[SIGINT])?;
    signals.handle();
    tokio::spawn(async move {
        while let Some(signal) = signals.next().await {
            if signal == SIGINT {
                info!("Shutting down...");
                process::exit(0);
            }
        }
    });

    Ok(())
}

pub fn setup_logger(log_file: &str) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()?;
    Ok(())
}

pub async fn start_health_checker(
    servers: Arc<Mutex<Vec<Server>>>,
    healthy_servers: &mut Array<MapData, [u8; 6]>,
    servers_cnt: &mut Array<MapData, u8>,
    health_period: u8,
) -> Result<(), anyhow::Error> {
    let mut servers = servers.lock().unwrap();
    loop {
        let mut state_changed = false;
        for server in &mut *servers {
            if server.run_health_check().await {
                state_changed = true;
            }
        }

        // Update only on state change to minimize lock acquisition
        if state_changed {
            let mut idx: u32 = 0;
            for server in &*servers {
                if server.is_healthy() {
                    healthy_servers.set(idx, server.get_mac_address(), 0)?;
                    idx += 1;
                }
            }
            servers_cnt.set(0, idx as u8, 0)?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(health_period as u64)).await;
    }
}
