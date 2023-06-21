use std::{
    collections::HashMap,
    marker::PhantomData,
    net::{IpAddr, UdpSocket},
};

use eyre::{eyre, Context, Result};

#[derive(Debug)]
pub struct Player {
    name: String,
    score: u32,
    ping: u16,
}

#[derive(Debug)]
pub struct StatusResponse {
    header: String,
    variables: HashMap<String, String>,
    players: Vec<Player>,
}

#[derive(Default, Debug)]
pub struct Connect;

#[derive(Debug)]
pub struct Server {
    ip: IpAddr,
    port: u16,
    connection: Option<UdpSocket>,
}

impl Server {
    fn send_data(&self, data: &[u8]) -> Result<()> {
        match &self.connection {
            Some(socket) => {
                socket.send(data).context("Sending")?;
                Ok(())
            }
            _ => Err(eyre!("Cannot send data")),
        }
    }

    fn read_data(&self, buf: &mut [u8]) -> Result<String> {
        match &self.connection {
            Some(socket) => {
                let _recived = socket.recv(buf)?;
                let data = String::from_utf8_lossy(buf);
                Ok(data.into_owned())
            }
            _ => Err(eyre!("Cannot read data")),
        }
    }

    pub fn get_status(&self) -> Result<StatusResponse> {
        self.send_data(b"\xFF\xFF\xFF\xFFgetstatus")?;

        let mut buf = [0_u8; 1024];
        let data = self.read_data(buf.as_mut_slice())?;

        let Some((header, body)) = data.split_once('\n') else {
            return Err(eyre!("Cannot split header and body from response"));
        };

        let Some((variables, players)) = body.split_once('\n') else {
            return Err(eyre!("Cannot split cvars and players from body"));
        };

        let variables = variables
            .split('\\')
            .skip_while(|variable| variable.is_empty())
            .collect::<Vec<_>>()
            .chunks_exact(2)
            .map(|chunk| (chunk[0].to_string(), chunk[1].to_string()))
            .collect::<HashMap<_, _>>();

        let players = players
            .split('\n')
            .filter(|player| !player.starts_with('\0'))
            .map(|valid_player| valid_player.split_whitespace().collect::<Vec<_>>())
            .map(|player_data| Player {
                name: player_data[2].to_string(),
                score: player_data[0].parse().unwrap_or_default(),
                ping: player_data[1].parse().unwrap_or_default(),
            })
            .collect::<Vec<_>>();

        Ok(StatusResponse {
            header: header.to_string(),
            variables,
            players,
        })
    }
}

#[derive(Default, Debug)]
pub struct Init;
#[derive(Default, Debug)]
pub struct Ip;
#[derive(Default, Debug)]
pub struct Port;
#[derive(Default, Debug)]
pub struct Build;

#[derive(Default, Debug)]
pub struct ServerBuilder<S> {
    ip: Option<IpAddr>,
    port: Option<u16>,
    _s: PhantomData<S>,
}

impl ServerBuilder<Init> {
    pub fn new() -> ServerBuilder<Ip> {
        ServerBuilder::default()
    }
}

impl ServerBuilder<Ip> {
    pub fn ip(self, ip: impl Into<IpAddr>) -> ServerBuilder<Port> {
        ServerBuilder {
            ip: Some(ip.into()),
            port: self.port,
            _s: PhantomData,
        }
    }
}
impl ServerBuilder<Port> {
    pub fn port(self, port: impl Into<u16>) -> ServerBuilder<Build> {
        ServerBuilder {
            ip: self.ip,
            port: Some(port.into()),
            _s: PhantomData,
        }
    }
}

impl ServerBuilder<Build> {
    pub fn build(self) -> Result<Server> {
        match (self.ip, self.port) {
            (Some(ip), Some(port)) => {
                let socket = UdpSocket::bind("[::]:0").context("binding")?;
                socket.connect((ip, port)).context("connecting")?;

                Ok(Server {
                    ip: self.ip.unwrap(),
                    port: self.port.unwrap_or_default(),
                    connection: Some(socket),
                })
            }
            _ => Err(eyre!("Cannot build server")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
