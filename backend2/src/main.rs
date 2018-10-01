#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate rocket;
extern crate tokio_core;
extern crate websocket;

mod fs_server;

use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Core, Handle};

use std::fmt::Debug;

use std::env;
use std::process;
use std::thread;

use websocket::async::Server;
use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;

use std::collections::HashMap;

enum PlayerState {
    AwaitingJoinOrCreate,
    AwaitingOpponent,
    AwaitngHit,
}

struct Player {
    id: String,
    game_id: String,
    state: PlayerState,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum MsgCmd {
    CreateMatch,
    JoinMatch(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum MsgReply {
    Error(String),
    CreateMatch(String),
    JoinMatch,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum MsgType {
    Cmd(MsgCmd),
    Reply(MsgReply),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Msg {
    id: String,
    msg_type: MsgType,
}

struct Match {
    p1_id: String,
    p2_id: String,
}

fn process_msg(msg: Msg) -> std::option::Option<websocket::OwnedMessage> {
    println!("{:?}", msg);

    let reply = Msg {
        id: msg.id,
        msg_type: MsgType::Reply(MsgReply::Error(String::from("some error"))),
    };

    let j = serde_json::to_string(&reply).unwrap();
    Some(websocket::OwnedMessage::Text(j))
}

struct ServerState {
    match_register: HashMap<String, Match>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            match_register: HashMap::new(),
        }
    }
}

fn ws_server() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    // bind to the server
    let server = Server::bind("0.0.0.0:8081", &handle).unwrap();

    let server_state = ServerState::new();

    //    match_register.insert(String::from("first match"), Match

    let mut connection_counter: u64 = 0;

    // a stream of incoming connections
    let f = server
        .incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(|(upgrade, addr)| {
            // New connection
            connection_counter += 1;

            println!(
                "Got a connection (from: {}, total count: {})",
                addr, connection_counter
            );

            let player = Player {
                id: String::from(""),
                game_id: String::from(""),
                state: PlayerState::AwaitingJoinOrCreate,
            };

            // accept the request to be a ws connection if it does
            let f = upgrade
                .accept()
                // send a greeting!
                .and_then(|(s, _)| {
                    s.send(Message::text("Givf player game id or create match!").into())
                }).and_then(|s| {
                    let (sink, stream) = s.split();
                    stream
                        .take_while(|m| Ok(!m.is_close()))
                        .filter_map(|m| {
                            println!("Message from Client: {:?}", m);
                            match m {
                                OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
                                OwnedMessage::Pong(_) => None,
                                OwnedMessage::Text(text) => {
                                    let msg: Msg = serde_json::from_str(&text).unwrap();
                                    process_msg(msg)
                                }
                                _ => Some(m),
                            }
                        }).forward(sink)
                        .and_then(|(_, sink)| sink.send(OwnedMessage::Close(None)))
                });

            spawn_future(f, "Client Status", &handle);
            Ok(())
        });

    core.run(f).unwrap();
}

fn main() {
    let http_server_port = match env::var("HTTP_PORT") {
        Ok(val) => val.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };
    let http_root = match env::var("HTTP_ROOT") {
        Ok(val) => val,
        Err(_) => "../frontend".to_string(),
    };

    let fs_server_thread = fs_server::start(http_root, http_server_port);
    let ws_server_thread = thread::spawn(move || {
        // Websocket server. Will run forever.
        ws_server();
    });

    let _res = fs_server_thread.join();
    let _res = ws_server_thread.join();
    return process::exit(0);
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
where
    F: Future<Item = I, Error = E> + 'static,
    E: Debug,
{
    handle.spawn(
        f.map_err(move |e| println!("{}: '{:?}'", desc, e))
            .map(move |_| println!("{}: Finished.", desc)),
    );
}
