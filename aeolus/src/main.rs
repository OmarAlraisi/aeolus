mod config;
mod server;
mod utils;

use anyhow::Context;
use aya::{
    include_bytes_aligned,
    maps::{Array, HashMap},
    programs::{Xdp, XdpFlags},
    Ebpf,
};
use aya_log::EbpfLogger;
use config::Config;
use log::{debug, info, warn};
use std::sync::Arc;
use utils::{setup_logger, setup_sigint_handler, start_health_checker};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Setup Aeolus configurations, logger, and SIGINT handler
    let opt = Config::parse()?;
    setup_logger(&opt.log_file)?;
    setup_sigint_handler().await?;

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
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/aeolus"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Ebpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/aeolus"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }
    let program: &mut Xdp = bpf.program_mut("aeolus").unwrap().try_into()?;
    program.load()?;
    program.attach(&opt.iface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    // Start
    info!("Aeolus running on '{}'!", opt.iface);

    let mut listeninig_ports: HashMap<_, u16, u16> =
        HashMap::try_from(bpf.map_mut("LISTENING_PORTS").unwrap())?;
    for port in opt.ports.iter() {
        listeninig_ports.insert(port, port, 0)?;
    }

    let mut healthy_servers: Array<_, [u8; 6]> = Array::try_from(bpf.take_map("SERVERS").unwrap())?;
    for (idx, server) in opt.servers.iter().enumerate() {
        healthy_servers.set(idx as u32, server.get_mac_address(), 0)?;
    }

    let servers = Arc::new(Mutex::new(opt.servers.clone()));
    let mut servers_count: Array<_, u8> = Array::try_from(bpf.take_map("SERVERS_COUNT").unwrap())?;
    servers_count.set(0, servers.lock().await.len() as u8, 0)?;

    let mut host_mac_address: Array<_, [u8; 6]> = Array::try_from(bpf.take_map("HOST_MAC_ADDRESS").unwrap())?;
    host_mac_address.set(0, opt.host_mac_address, 0)?;

    // Runs health checker
    let health_interval = opt.health_interval;
    start_health_checker(servers.clone(), &mut healthy_servers, &mut servers_count, health_interval).await?;
    Ok(())
}
