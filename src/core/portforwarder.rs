/*
let forward_ports = |mut forwarder: Forwarder| {
    forwarder
        .forward_port(27015, 27015, PortMappingProtocol::UDP, "tf2 server")
        .map_err(|_| Error::PortForwardingError)
};

let create_forwarder = |network_interfaces: Vec<Interface>| {
    portforwarder_rs::port_forwarder::create_forwarder_from_any(
        network_interfaces
            .into_iter()
            .map(|interface| interface.addr),
    )
    .map_err(|_| Error::PortForwardingError)
};

let _ = portforwarder_rs::query_interfaces::get_network_interfaces()
    .map_err(|_| Error::PortForwardingError)
    .and_then(create_forwarder)
    .and_then(forward_ports)
    .map_err(|_| async {
        let _ = Notification::new()
            .appname("MANNager")
            .summary("[ MANNager ] Server running...")
            .body("Port forwarding failed.")
            .timeout(5)
            .show_async()
            .await;
    });
*/

use std::{io, net::Ipv4Addr};

use igd::{AddPortError, RemovePortError, SearchError};
use portforwarder_rs::port_forwarder::{Forwarder, PortMappingProtocol};
use snafu::{ResultExt, Snafu};

pub struct PortForwarder {
    forwarder: Forwarder,
    remote_port: u16,
    proto: PortMappingProtocol,
}

pub enum PortForwarderIP {
    Any,
    Ip(Ipv4Addr),
}

impl PortForwarder {
    pub fn open(
        ip: PortForwarderIP,
        local_port: u16,
        remote_port: u16,
        proto: PortMappingProtocol,
        name: &str,
    ) -> Result<Self, Error> {
        let mut forwarder = match ip {
            PortForwarderIP::Any => {
                let interfaces = portforwarder_rs::query_interfaces::get_network_interfaces()
                    .context(NoInterfacesSnafu)?;

                portforwarder_rs::port_forwarder::create_forwarder_from_any(
                    interfaces.into_iter().map(|interface| interface.addr),
                )
                .map_err(|errs| errs.into_iter().next().unwrap())
                .context(NoGatewayFoundSnafu)?
            }
            PortForwarderIP::Ip(_ip) => portforwarder_rs::port_forwarder::create_forwarder(_ip)
                .context(NoGatewayFoundSnafu)?,
        };

        forwarder
            .forward_port(local_port, remote_port, proto, name)
            .context(AddPortSnafu)
            .context(PortForwardingFailedSnafu)?;

        Ok(Self {
            forwarder,
            remote_port,
            proto,
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.forwarder
            .remove_port(self.remote_port, self.proto)
            .context(RemovePortSnafu)
            .context(PortForwardingFailedSnafu)
    }
}
// TODO improve display errors
#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display(""))]
    NoInterfacesError { source: io::Error },
    #[snafu(display(""))]
    NoGatewayFoundError { source: SearchError },
    #[snafu(display(""))]
    PortForwardingFailed { source: PortForwardingError },
}

#[derive(Snafu, Debug)]
pub enum PortForwardingError {
    #[snafu(display(""))]
    AddPortError {
        source: AddPortError,
    },
    RemovePortError {
        source: RemovePortError,
    },
}
