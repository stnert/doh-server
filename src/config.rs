use libdoh::*;

use crate::constants::*;

use clap::Arg;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::time::Duration;

#[cfg(feature = "tls")]
use std::path::PathBuf;

pub fn parse_opts(globals: &mut Globals) {
    use crate::utils::{verify_remote_server, verify_sock_addr};

    let max_clients = MAX_CLIENTS.to_string();
    let timeout_sec = TIMEOUT_SEC.to_string();
    let max_concurrent_streams = MAX_CONCURRENT_STREAMS.to_string();
    let min_ttl = MIN_TTL.to_string();
    let max_ttl = MAX_TTL.to_string();
    let err_ttl = ERR_TTL.to_string();

    let _ = include_str!("../Cargo.toml");
    let options = app_from_crate!()
        .arg(
            Arg::new("hostname")
                .short('H')
                .long("hostname")
                .takes_value(true)
                .help("Host name (not IP address) DoH clients will use to connect"),
        )
        .arg(
            Arg::new("public_address")
                .short('g')
                .long("public-address")
                .takes_value(true)
                .help("External IP address DoH clients will connect to"),
        )
        .arg(
            Arg::new("public_port")
                .short('j')
                .long("public-port")
                .takes_value(true)
                .help("External port DoH clients will connect to, if not 443"),
        )
        .arg(
            Arg::new("listen_address")
                .short('l')
                .long("listen-address")
                .takes_value(true)
                .default_value(LISTEN_ADDRESS)
                .validator(verify_sock_addr)
                .help("Address to listen to"),
        )
        .arg(
            Arg::new("server_address")
                .short('u')
                .long("server-address")
                .takes_value(true)
                .default_value(SERVER_ADDRESS)
                .validator(verify_remote_server)
                .help("Address to connect to"),
        )
        .arg(
            Arg::new("local_bind_address")
                .short('b')
                .long("local-bind-address")
                .takes_value(true)
                .validator(verify_sock_addr)
                .help("Address to connect from"),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .takes_value(true)
                .default_value(PATH)
                .help("URI path"),
        )
        .arg(
            Arg::new("max_clients")
                .short('c')
                .long("max-clients")
                .takes_value(true)
                .default_value(&max_clients)
                .help("Maximum number of simultaneous clients"),
        )
        .arg(
            Arg::new("max_concurrent")
                .short('C')
                .long("max-concurrent")
                .takes_value(true)
                .default_value(&max_concurrent_streams)
                .help("Maximum number of concurrent requests per client"),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .takes_value(true)
                .default_value(&timeout_sec)
                .help("Timeout, in seconds"),
        )
        .arg(
            Arg::new("min_ttl")
                .short('T')
                .long("min-ttl")
                .takes_value(true)
                .default_value(&min_ttl)
                .help("Minimum TTL, in seconds"),
        )
        .arg(
            Arg::new("max_ttl")
                .short('X')
                .long("max-ttl")
                .takes_value(true)
                .default_value(&max_ttl)
                .help("Maximum TTL, in seconds"),
        )
        .arg(
            Arg::new("err_ttl")
                .short('E')
                .long("err-ttl")
                .takes_value(true)
                .default_value(&err_ttl)
                .help("TTL for errors, in seconds"),
        )
        .arg(
            Arg::new("disable_keepalive")
                .short('K')
                .long("disable-keepalive")
                .help("Disable keepalive"),
        )
        .arg(
            Arg::new("disable_post")
                .short('P')
                .long("disable-post")
                .help("Disable POST queries"),
        )
        .arg(
            Arg::new("allow_odoh_post")
                .short('O')
                .long("allow-odoh-post")
                .help("Allow POST queries over ODoH even if they have been disabed for DoH"),
        );

    #[cfg(feature = "tls")]
    let options = options
        .arg(
            Arg::new("tls_cert_path")
                .short('i')
                .long("tls-cert-path")
                .takes_value(true)
                .help(
                    "Path to the PEM/PKCS#8-encoded certificates (only required for built-in TLS)",
                ),
        )
        .arg(
            Arg::new("tls_cert_key_path")
                .short('I')
                .long("tls-cert-key-path")
                .takes_value(true)
                .help("Path to the PEM-encoded secret keys (only required for built-in TLS)"),
        );

    let matches = options.get_matches();
    globals.listen_address = matches.value_of("listen_address").unwrap().parse().unwrap();

    globals.server_address = matches
        .value_of("server_address")
        .unwrap()
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    globals.local_bind_address = match matches.value_of("local_bind_address") {
        Some(address) => address.parse().unwrap(),
        None => match globals.server_address {
            SocketAddr::V4(_) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
            SocketAddr::V6(s) => SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::UNSPECIFIED,
                0,
                s.flowinfo(),
                s.scope_id(),
            )),
        },
    };
    globals.path = matches.value_of("path").unwrap().to_string();
    if !globals.path.starts_with('/') {
        globals.path = format!("/{}", globals.path);
    }
    globals.max_clients = matches.value_of("max_clients").unwrap().parse().unwrap();
    globals.timeout = Duration::from_secs(matches.value_of("timeout").unwrap().parse().unwrap());
    globals.max_concurrent_streams = matches.value_of("max_concurrent").unwrap().parse().unwrap();
    globals.min_ttl = matches.value_of("min_ttl").unwrap().parse().unwrap();
    globals.max_ttl = matches.value_of("max_ttl").unwrap().parse().unwrap();
    globals.err_ttl = matches.value_of("err_ttl").unwrap().parse().unwrap();
    globals.keepalive = !matches.is_present("disable_keepalive");
    globals.disable_post = matches.is_present("disable_post");
    globals.allow_odoh_post = matches.is_present("allow_odoh_post");

    #[cfg(feature = "tls")]
    {
        globals.tls_cert_path = matches.value_of("tls_cert_path").map(PathBuf::from);
        globals.tls_cert_key_path = matches
            .value_of("tls_cert_key_path")
            .map(PathBuf::from)
            .or_else(|| globals.tls_cert_path.clone());
    }

    if let Some(hostname) = matches.value_of("hostname") {
        let mut builder =
            dnsstamps::DoHBuilder::new(hostname.to_string(), globals.path.to_string());
        if let Some(public_address) = matches.value_of("public_address") {
            builder = builder.with_address(public_address.to_string());
        }
        if let Some(public_port) = matches.value_of("public_port") {
            let public_port = public_port.parse().expect("Invalid public port");
            builder = builder.with_port(public_port);
        }
        println!(
            "Test DNS stamp to reach [{}] over DoH: [{}]\n",
            hostname,
            builder.serialize().unwrap()
        );

        let mut builder =
            dnsstamps::ODoHTargetBuilder::new(hostname.to_string(), globals.path.to_string());
        if let Some(public_port) = matches.value_of("public_port") {
            let public_port = public_port.parse().expect("Invalid public port");
            builder = builder.with_port(public_port);
        }
        println!(
            "Test DNS stamp to reach [{}] over Oblivious DoH: [{}]\n",
            hostname,
            builder.serialize().unwrap()
        );

        println!("Check out https://dnscrypt.info/stamps/ to compute the actual stamps.\n")
    } else {
        println!("Please provide a fully qualified hostname (-H <hostname> command-line option) to get test DNS stamps for your server.\n");
    }
}
