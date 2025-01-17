use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use surge_ping::{IcmpPacket, PingSocket, PingSocketBuilder};
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ips = [
        "114.114.114.114",
        "8.8.8.8",
        "39.156.69.79",
        "172.217.26.142",
        "240c::6666",
        "2a02:930::ff76",
    ];
    let builder_v4 = PingSocketBuilder::new(socket2::Domain::IPV4)?;
    let ping_socket_v4 = Arc::new(builder_v4.build()?);
    let ping_socket_v6 = Arc::new(PingSocket::new(socket2::Domain::IPV6)?);
    let mut tasks = Vec::new();
    for ip in &ips {
        let addr: IpAddr = ip.parse()?;
        let psc = match addr {
            IpAddr::V4(_) => ping_socket_v4.clone(),
            IpAddr::V6(_) => ping_socket_v6.clone(),
        };
        tasks.push(tokio::spawn(async move {
            ping(psc, addr, 56).await.unwrap();
        }));
    }
    for t in tasks.into_iter() {
        t.await.unwrap();
    }
    //signal::ctrl_c().await?;
    //println!("ctrl-c received!");
    Ok(())
}
// Ping an address 5 times， and print output message（interval 1s）
async fn ping(
    ps: Arc<PingSocket>,
    addr: IpAddr,
    size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pinger = ps.pinger(addr).await;
    pinger.size(size).timeout(Duration::from_secs(1));
    let mut interval = time::interval(Duration::from_secs(1));
    for idx in 0..5 {
        interval.tick().await;
        match pinger.ping(idx).await {
            Ok((IcmpPacket::V4(packet), dur)) => println!(
                "{} bytes from {}: icmp_seq={} ttl={} time={:?}",
                packet.get_size(),
                packet.get_source(),
                packet.get_sequence(),
                packet.get_ttl(),
                dur
            ),
            Ok((IcmpPacket::V6(packet), dur)) => println!(
                "{} bytes from {}: icmp_seq={} hlim={} time={:?}",
                packet.get_size(),
                packet.get_source(),
                packet.get_sequence(),
                packet.get_max_hop_limit(),
                dur
            ),
            Err(e) => println!("{} ping {}", addr, e),
        };
    }
    println!("[+] {} done.", addr);
    Ok(())
}
