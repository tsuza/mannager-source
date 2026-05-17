use std::{io, net::Ipv4Addr, sync::Arc};

use igd::{AddPortError, RemovePortError, SearchError};
use portforwarder_rs::port_forwarder::{Forwarder, PortMappingProtocol};
use snafu::{ResultExt, Snafu};

#[derive(Debug)]
pub struct PortForwarder {
    forwarder: Forwarder,
    remote_port: u16,
    proto: PortMappingProtocol,
}

pub enum PortForwarderIP {
    Any,
    Ip(Ipv4Addr),
}

// TODO: Switch to igd_next
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
#[derive(Snafu, Debug, Clone)]
pub enum Error {
    #[snafu(display("No network interfaces found"))]
    NoInterfacesError {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },

    #[snafu(display("No gateway found"))]
    NoGatewayFoundError {
        #[snafu(source(from(SearchError, Arc::new)))]
        source: Arc<SearchError>,
    },

    #[snafu(display("Port forwarding failed"))]
    PortForwardingFailed { source: PortForwardingError },
}

#[derive(Snafu, Debug, Clone)]
pub enum PortForwardingError {
    #[snafu(display("Failed to add port mapping"))]
    AddPortError {
        #[snafu(source(from(AddPortError, Arc::new)))]
        source: Arc<AddPortError>,
    },

    #[snafu(display("Failed to remove port mapping"))]
    RemovePortError {
        #[snafu(source(from(RemovePortError, Arc::new)))]
        source: Arc<RemovePortError>,
    },
}
