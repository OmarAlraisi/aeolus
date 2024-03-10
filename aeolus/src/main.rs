mod config;
mod server;

use anyhow::Context;
use aya::{
    include_bytes_aligned,
    maps::{Array, HashMap, MapData},
    programs::{Xdp, XdpFlags},
    Bpf,
};
use aya_log::BpfLogger;
use config::Config;
use log::{debug, info, warn};
use server::Server;
use std::time::SystemTime;
use tokio::signal;

fn setup_logger(log_file: &str) -> Result<(), fern::InitError> {
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opt = Config::parse()?;
    let mut servers = opt.servers.clone();

    setup_logger(&opt.log_file)?;

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/aeolus"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/aeolus"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("aeolus").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut listeninig_ports: HashMap<_, u16, u16> =
        HashMap::try_from(bpf.map_mut("LISTENING_PORTS").unwrap())?;
    for port in opt.ports.iter() {
        listeninig_ports.insert(port, port, 0)?;
    }

    let mut healthy_servers: Array<_, [u8; 6]> = Array::try_from(bpf.take_map("SERVERS").unwrap())?;
    for (idx, server) in opt.servers.iter().enumerate() {
        healthy_servers.set(idx as u32, server.get_mac_address(), 0)?;
    }

    let mut servers_count: Array<_, u8> = Array::try_from(bpf.take_map("SERVERS_COUNT").unwrap())?;
    servers_count.set(0, servers.len() as u8, 0)?;
    health_checker(&mut servers, healthy_servers, servers_count).await?;

    info!("Aeolus running on '{}'!", opt.iface);
    signal::ctrl_c().await?;
    info!("Shutting down...");

    // TODO: Add health checks for servers

    Ok(())
}

async fn health_checker(servers: &mut Vec<Server>, mut healthy_servers: Array<MapData, [u8; 6]>, mut servers_cnt: Array<MapData, u8>) -> Result<(), anyhow::Error> {
    for server in &mut *servers {
        server.run_health_check().await;
    }

    let mut idx: u32 = 0;
    for server in servers {
        if server.is_healthy() {
            healthy_servers.set(idx, server.get_mac_address(), 0)?;
            idx += 1;
        }
        servers_cnt.set(0, idx as u8, 0)?;
    }
    Ok(())
}
