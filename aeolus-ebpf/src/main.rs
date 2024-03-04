#![no_std]
#![no_main]

use core::mem;
use network_types::{eth::{EthHdr, EtherType}, ip::{IpProto, Ipv4Hdr}, tcp::TcpHdr, udp::UdpHdr};
use aya_bpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::info;

#[xdp]
pub fn aeolus(ctx: XdpContext) -> u32 {
    match try_aeolus(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

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

fn try_aeolus(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?;
    let mut offset = EthHdr::LEN;

    match unsafe { *ethhdr }.ether_type {
        EtherType::Ipv4 => {},
        _ => return Ok(xdp_action::XDP_PASS)
    };

    let iphdr: *const Ipv4Hdr = ptr_at(&ctx, offset)?;
    offset += Ipv4Hdr::LEN;

    let src_addr = u32::from_be(unsafe { *iphdr }.src_addr);
    let dst_addr = u32::from_be(unsafe { *iphdr }.dst_addr);

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
        _ => return Ok(xdp_action::XDP_PASS)
    };
    
    info!(&ctx, "SRC: {:i}:{}, DST: {:i}:{}", src_addr, src_port, dst_addr, dst_port);
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
