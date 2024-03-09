#![no_std]
#![no_main]

use aya_bpf::{
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

    // TODO: calculate 4-tuple hash
    // TODO: Get MAC address of the appropriate server

    // Modify destination MAC address
    unsafe {
        (*ethhdr).dst_addr = [0x52, 0x54, 0x00, 0x94, 0xdf, 0x40];
    }

    let mac = unsafe { *ethhdr }.dst_addr;
    info!(
        &ctx,
        "SRC: {:i}:{}, DST: {:i}:{} ---> redirecting to MAC: {:mac}",
        src_ip,
        src_port,
        dst_ip,
        dst_port,
        mac
    );
    Ok(xdp_action::XDP_TX)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
