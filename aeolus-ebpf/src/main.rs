#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{Array, HashMap},
    programs::XdpContext,
};
use aya_log_ebpf::info;
use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

#[xdp]
pub fn aeolus(ctx: XdpContext) -> u32 {
    match try_aeolus(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

// 1MB
#[map]
static LISTENING_PORTS: HashMap<u16, u16> = HashMap::with_max_entries(512, 0);

// ~1MB (1020 Bytes)
#[map]
static SERVERS: Array<[u8; 6]> = Array::with_max_entries(170, 0);

// 1B - Number of healthy servers
#[map]
static SERVERS_COUNT: Array<u8> = Array::with_max_entries(1, 0);

// 8B 
#[map]
static HOST_MAC_ADDRESS: Array<[u8; 6]> = Array::with_max_entries(1, 0);

#[inline(always)]
fn get_destination_mac(src_ip: u32, dst_ip: u32, src_port: u16, dst_port: u16) -> [u8; 6] {
    if let Some(servers_cnt) = SERVERS_COUNT.get(0) {
        let servers_cnt = *servers_cnt as u32;
        let hash_key = (src_ip + dst_ip + (src_port + dst_port) as u32) % servers_cnt;

        if let Some(server) = SERVERS.get(hash_key) {
            *server
        } else {
            [0; 6]
        }
    } else {
        [0; 6]
    }
}

#[inline(always)]
fn is_own_address(mac_address: &[u8; 6]) -> bool {
    if let Some(host_mac_address) = HOST_MAC_ADDRESS.get(0) {
        if mac_address == host_mac_address {
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let size = mem::size_of::<T>();

    if start + offset + size > end {
        Err(())
    } else {
        Ok((start + offset) as *const T)
    }
}

#[inline(always)]
fn ptr_at_mut<T>(ctx: &XdpContext, offset: usize) -> Result<*mut T, ()> {
    Ok((ptr_at::<T>(ctx, offset)?) as *mut T)
}

fn is_listening_port(port: u16) -> bool {
    unsafe { LISTENING_PORTS.get(&port).is_some() }
}

fn try_aeolus(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *mut EthHdr = ptr_at_mut(&ctx, 0)?;
    let mut offset = EthHdr::LEN;

    match unsafe { *ethhdr }.ether_type {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    };

    let iphdr: *const Ipv4Hdr = ptr_at(&ctx, offset)?;
    offset += Ipv4Hdr::LEN;

    let src_ip = u32::from_be(unsafe { *iphdr }.src_addr);
    let dst_ip = u32::from_be(unsafe { *iphdr }.dst_addr);

    let (src_port, dst_port) = match unsafe { *iphdr }.proto {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr = ptr_at(&ctx, offset)?;
            let src_port = u16::from_be(unsafe { *tcphdr }.source);
            let dst_port = u16::from_be(unsafe { *tcphdr }.dest);
            (src_port, dst_port)
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr = ptr_at(&ctx, offset)?;
            let src_port = u16::from_be(unsafe { *udphdr }.source);
            let dst_port = u16::from_be(unsafe { *udphdr }.dest);
            (src_port, dst_port)
        }
        _ => return Ok(xdp_action::XDP_PASS),
    };

    // Check if port should be balanced
    if !is_listening_port(dst_port) {
        return Ok(xdp_action::XDP_PASS);
    }

    // Get destination Mac
    let dst_mac = get_destination_mac(src_ip, dst_ip, src_port, dst_port);

    // Chekc if own mac address
    if is_own_address(&dst_mac) {
        return Ok(xdp_action::XDP_PASS);
    }

    info!(&ctx, "Destination MAC address: {:mac}", dst_mac);

    // Modify destination MAC address
    unsafe {
        (*ethhdr).dst_addr = dst_mac;
    }

    Ok(xdp_action::XDP_TX)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
