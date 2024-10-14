include!(concat!(env!("OUT_DIR"), "/snakes.rs"));

use game_message::*;

impl AckMsg {
    pub fn new(sender_id: i32, receiver_id: i32, seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: Some(sender_id),
            receiver_id: Some(receiver_id),
            r#type: Some(Type::Ack(AckMsg {})),
        }
    }
}

impl PingMsg {
    pub fn new(seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Ping(PingMsg {})),
        }
    }
}

impl ErrorMsg {
    pub fn new<S>(error: S, seq: i64) -> GameMessage
    where
        S: Into<String>,
    {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Error(ErrorMsg {
                error_message: error.into(),
            })),
        }
    }
}

impl SteerMsg {
    pub fn new(direction: Direction, seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Steer(SteerMsg {
                direction: direction.into(),
            })),
        }
    }
}

impl JoinMsg {
    pub fn new(
        player_name: impl Into<String>,
        game_name: impl Into<String>,
        role: NodeRole,
        seq: i64,
    ) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Join(JoinMsg {
                player_type: Some(PlayerType::Human.into()),
                player_name: player_name.into(),
                game_name: game_name.into(),
                requested_role: role.into(),
            })),
        }
    }
}

impl DiscoverMsg {
    pub fn new(seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Discover(DiscoverMsg {})),
        }
    }
}

impl RoleChangeMsg {
    pub fn new(
        sender_id: i32,
        sender_role: Option<NodeRole>,
        receiver_id: i32,
        receiver_role: Option<NodeRole>,
        seq: i64,
    ) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: Some(sender_id),
            receiver_id: Some(receiver_id),
            r#type: Some(Type::RoleChange(RoleChangeMsg {
                sender_role: sender_role.map(|role| role.into()),
                receiver_role: receiver_role.map(|role| role.into()),
            })),
        }
    }
}

impl AnnouncementMsg {
    pub fn new(game: GameAnnouncement, seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::Announcement(AnnouncementMsg { games: vec![game] })),
        }
    }
}

impl StateMsg {
    pub fn new(state: GameState, seq: i64) -> GameMessage {
        GameMessage {
            msg_seq: seq,
            sender_id: None,
            receiver_id: None,
            r#type: Some(Type::State(StateMsg { state })),
        }
    }
}
