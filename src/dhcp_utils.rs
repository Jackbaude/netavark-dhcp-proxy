use nispor::{AddressFamily, IfaceConf, IfaceState, IpAddrConf, IpConf, NetConf, NetState, RouteConf, RouteProtocol};

const DEFAULT_METRIC: u32 = 500;

pub fn new_net_conf_with_ip_conf(iface_name: &str, ip_conf: IpConf) -> NetConf {
    let mut iface_conf = IfaceConf::default();
    iface_conf.name = iface_name.to_string();
    iface_conf.state = IfaceState::Up;
    iface_conf.ipv4 = Some(ip_conf);
    let mut net_conf = NetConf::default();
    net_conf.ifaces = Some(vec![iface_conf]);
    net_conf
}

pub fn apply_dhcp_ip_route(iface_name: &str, lease: &mozim::DhcpV4Lease) {
    let mut ip_addr_conf = IpAddrConf::default();
    ip_addr_conf.address = lease.yiaddr.to_string();
    ip_addr_conf.prefix_len = get_prefix_len(&lease.subnet_mask);
    ip_addr_conf.valid_lft = format!("{}sec", lease.lease_time);
    ip_addr_conf.preferred_lft = format!("{}sec", lease.lease_time);
    let mut ip_conf = IpConf::default();
    ip_conf.addresses = vec![ip_addr_conf];
    let mut net_conf = new_net_conf_with_ip_conf(iface_name, ip_conf);
    if let Some(gws) = lease.gateways.as_ref() {
        let mut routes = Vec::new();
        for (i, gw) in gws.as_slice().iter().enumerate() {
            routes.push(gen_rt_conf(
                false,
                "0.0.0.0/0",
                iface_name,
                &gw.to_string(),
                Some(DEFAULT_METRIC + i as u32),
            ));
        }
        if !routes.is_empty() {
            net_conf.routes = Some(routes);
        }
    }

    log::debug!("Applying {:?}", net_conf);
    net_conf.apply().unwrap();
}

pub fn get_prefix_len(ip: &std::net::Ipv4Addr) -> u8 {
    u32::from_be_bytes(ip.octets()).count_ones() as u8
}

// Remove all dynamic IP and dhcp routes of specified interface
pub fn purge_dhcp_ip_route(iface_name: &str) {
    let state = NetState::retrieve().unwrap();
    if let Some(ip_info) =
    state.ifaces.get(iface_name).and_then(|i| i.ipv4.as_ref())
    {
        let mut addrs_to_remove = Vec::new();
        for addr in ip_info
            .addresses
            .as_slice()
            .iter()
            .filter(|a| a.valid_lft != "forever")
        {
            let mut addr_conf = IpAddrConf::default();
            addr_conf.remove = true;
            addr_conf.address = addr.address.clone();
            addr_conf.prefix_len = addr.prefix_len;
            addrs_to_remove.push(addr_conf);
        }
        if !addrs_to_remove.is_empty() {
            let mut ip_conf = IpConf::default();
            ip_conf.addresses = addrs_to_remove;
            new_net_conf_with_ip_conf(iface_name, ip_conf)
                .apply()
                .expect("Likely not ran with root");
        }
    }
    let mut routes_to_remove = Vec::new();
    for rt in state.routes.as_slice().iter().filter(|rt| {
        rt.oif.as_deref() == Some(iface_name)
            && rt.protocol == RouteProtocol::Dhcp
            && rt.address_family == AddressFamily::IPv4
    }) {
        routes_to_remove.push(gen_rt_conf(
            true,
            rt.dst.as_deref().unwrap_or("0.0.0.0/0"),
            iface_name,
            rt.via
                .as_deref()
                .unwrap_or_else(|| rt.gateway.as_deref().unwrap_or("0.0.0.0")),
            None,
        ));
    }
    let mut net_conf = NetConf::default();
    net_conf.routes = Some(routes_to_remove);
    net_conf.apply().expect("Likely not ran with root");
}

pub fn gen_rt_conf(
    remove: bool,
    dst: &str,
    oif: &str,
    via: &str,
    metric: Option<u32>,
) -> RouteConf {
    let mut rt = RouteConf::default();
    rt.remove = remove;
    rt.dst = dst.to_string();
    rt.oif = Some(oif.to_string());
    rt.via = Some(via.to_string());
    rt.table = Some(254);
    rt.metric = metric;
    rt.protocol = Some(RouteProtocol::Dhcp);
    rt
}
