use itertools::EitherOrBoth;

#[derive(Debug)]
pub enum ChannelMode {
    Op { nickname: String },
    HalfOp { nickname: String },
}

#[derive(Debug)]
pub struct Channel {
    name: String,
    topic: String,
    clients: Vec<u64>,
    modes: Vec<ChannelMode>,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic: String::new(),
            clients: vec![],
            modes: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn topic(&self) -> &str {
        &self.topic
    }

    pub fn clients(&self) -> &[u64] {
        &self.clients[..]
    }

    pub fn join_user(&mut self, client_id: u64) {
        self.clients.push(client_id);
    }
}

#[derive(Debug)]
pub struct ChannelRef {
    pub name: String,
    pub key: Option<String>,
}

impl From<EitherOrBoth<&str, &str>> for ChannelRef {
    fn from(either: EitherOrBoth<&str, &str>) -> Self {
        match either {
            EitherOrBoth::Both(channel, key) => Self {
                name: channel.to_string(),
                key: Some(key.to_string()),
            },
            EitherOrBoth::Left(channel) => Self {
                name: channel.to_string(),
                key: None,
            },
            EitherOrBoth::Right(_) => panic!("Unable to construct `ChannelRef` from key only."),
        }
    }
}
