use crate::{ConnecterError, Error};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use trust_dns_resolver::{IntoName, TokioAsyncResolver};

async fn connect_to_host(
    resolver: &TokioAsyncResolver,
    host: &str,
    port: u16,
) -> Result<TcpStream, Error> {
    let ips = resolver
        .lookup_ip(host)
        .await
        .map_err(ConnecterError::Resolve)?;
    for ip in ips.iter() {
        match TcpStream::connect(&SocketAddr::new(ip, port)).await {
            Ok(stream) => return Ok(stream),
            Err(_) => {}
        }
    }
    Err(Error::Disconnected)
}

pub async fn connect(
    domain: &str,
    srv: Option<&str>,
    fallback_port: u16,
) -> Result<TcpStream, Error> {
    if let Ok(ip) = domain.parse() {
        return Ok(TcpStream::connect(&SocketAddr::new(ip, fallback_port)).await?);
    }

    let resolver = TokioAsyncResolver::tokio_from_system_conf().map_err(ConnecterError::Resolve)?;

    let srv_records = match srv {
        Some(srv) => {
            let srv_domain = format!("{}.{}.", srv, domain)
                .into_name()
                .map_err(ConnecterError::Dns)?;
            resolver.srv_lookup(srv_domain).await.ok()
        }
        None => None,
    };

    match srv_records {
        Some(lookup) => {
            // TODO: sort lookup records by priority/weight
            for srv in lookup.iter() {
                match connect_to_host(&resolver, &srv.target().to_ascii(), srv.port()).await {
                    Ok(stream) => return Ok(stream),
                    Err(_) => {}
                }
            }
            Err(Error::Disconnected)
        }
        None => {
            // SRV lookup error, retry with hostname
            connect_to_host(&resolver, domain, fallback_port).await
        }
    }
}
