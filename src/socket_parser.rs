use std::{
    env::Args,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

pub(crate) fn get_socketaddr(cli_args: Args) -> Result<SocketAddr, String> {
    let filtered_args = cli_args.into_iter().skip(1).collect::<Vec<String>>();

    let mut ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let mut port = 1024u16;

    if filtered_args.is_empty() {
        return Ok(SocketAddr::new(ip, port));
    }

    for (index, value) in filtered_args.iter().enumerate() {
        if index != 1 && index != 3 {
            match value.as_str() {
                "-ip" => {
                    if let Some(ip_value) = filtered_args.get(index + 1) {
                        match ip_value.parse() {
                            Ok(value) => ip = value,
                            Err(_) => {
                                return Err(
                                    "Invalid IP address provided to the command line arguments"
                                        .to_owned(),
                                );
                            }
                        };
                    } else {
                        return Err(format!("Expected a  IP Address Argument at index `{}` for `-ip` command line argument", index + 1
                      ),
                );
                    }
                }
                "-port" => {
                    if let Some(port_value) = filtered_args.get(index + 1) {
                        match port_value.parse() {
                            Ok(value) => port = value,
                            Err(_) => {
                                return Err("Invalid port provided to the command line arguments"
                                    .to_owned());
                            }
                        }
                    } else {
                        return Err(
                    format!("Expected a  port Argument at index `{}` for `-port` command line argument"
                        , index +1),
                );
                    }
                }
                _ => {
                    return Err(format!(
                        "Invalid Argument `{}`. Use `-h` to list available commands",
                        value
                    ))
                }
            }
        }
    }

    Ok(SocketAddr::new(ip, port))
}
