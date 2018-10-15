#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate serde;
extern crate serde_json;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate futures;
extern crate rocket;
extern crate tokio_core;
extern crate tokio_timer;
extern crate websocket;

mod fs_server;
mod proto;

use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Core, Handle};

use tokio_timer::Interval;

use std::fmt::Debug;

use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use websocket::async::Server;
use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;

use std::collections::HashMap;

struct Game {
    id: String,
    p1_id: String,
    p2_id: String,
    //    p1_send_cb: Option<Box<Fn<msg: proto::CmdMsg>>>,
    //    p2_send_cb: Option<Box<Fn(msg: proto::CmdMsg)>>,
}

// HashMap<String, Rc<RefCell<Match>>>,
struct ServerState {
    // Player id to game id
    player_to_game: HashMap<String, String>,
    games: HashMap<String, Rc<RefCell<Game>>>,
}

struct PlayerConnection {
    id: Option<String>,
    sink: mpsc::UnboundedSender<OwnedMessage>,
}

impl PlayerConnection {
    fn new(sink: mpsc::UnboundedSender<OwnedMessage>) -> Self {
        PlayerConnection {
            id: None,
            sink: sink,
        }
    }

    fn process_msg(&mut self, msg: proto::CmdMsg) -> proto::ReplyMsg {
        println!("Cmd: {:?}", msg);

        let reply = match msg.cmd {
            proto::Cmd::CreateGame => {
                // Create a new game
                proto::Reply::Ok
            }
            proto::Cmd::JoinGame(game_id) => proto::Reply::Ok,
        };

        proto::ReplyMsg {
            id: msg.id,
            reply: reply,
        }
    }
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            player_to_game: HashMap::new(),
            games: HashMap::new(),
        }
    }

    fn create_game(&mut self) -> Rc<RefCell<Game>> {
        let game_id = Uuid::new_v4().to_string();
        let game = Game {
            id: game_id.clone(),
            p1_id: Uuid::new_v4().to_string(),
            p2_id: Uuid::new_v4().to_string(),
        };
        self.player_to_game
            .insert(game.p1_id.clone(), game_id.clone());
        self.player_to_game
            .insert(game.p2_id.clone(), game_id.clone());

        let game = Rc::new(RefCell::new(game));
        self.games.insert(game_id.clone(), game.clone());
        return game;
    }

    fn game_from_id(&mut self, game_id: &str) -> Option<Rc<RefCell<Game>>> {
        self.games.get(game_id).and_then(|game| Some(game.clone()))
    }
}

//use std::error::Error;
use bytes::BytesMut;
use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};

fn ws_server() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    // bind to the server
    let server = Server::bind("0.0.0.0:8081", &handle).unwrap();

    let server_state = Rc::new(RefCell::new(ServerState::new()));

    /*    let mut counter = Rc::new(RefCell::new(0));
    
    let counter1 = counter.clone();
    let counter2 = counter.clone();
    let interval = Interval::new_interval(Duration::new(1, 0)).for_each(move |(_)| {
        *counter1.borrow_mut() += 1;
        println!("interval1: {}", counter1.borrow());
        Ok(())
    });
    handle.spawn(interval.map_err(|_| ()));
    let interval2 = Interval::new_interval(Duration::new(1, 0)).for_each(move |(_)| {
        *counter2.borrow_mut() += 1;
        println!("interval2: {}", *counter2.borrow());
        Ok(())
    });
    handle.spawn(interval2.map_err(|_| ()));*/

    let mut connection_counter: u64 = 0;

    // a stream of incoming connections
    let f = server
        .incoming()
        // we don't wanna save the stream if it drops
        .map_err(|InvalidConnection { error, .. }| error)
        .for_each(move |(upgrade, addr)| {
            // New connection
            connection_counter += 1;

            println!(
                "Got a connection (from: {}, total count: {})",
                addr, connection_counter
            );

            let server_state = server_state.clone();

            // accept the request to be a ws connection if it does
            let f = upgrade.accept().and_then(move |(s, _)| {
                let (sink, stream) = s.split();

                // Create a multi producer single consumer channel for this web socket.
                // So that messages can be sent
                let (unbounded_sink, unbounded_stream) = mpsc::unbounded::<OwnedMessage>();

                let sender = sink.send_all(
                    unbounded_stream
                        .map_err(|_err| Error::new(ErrorKind::Other, "Some stream error")),
                );
                spawn_future(sender, "Bla", &handle);

                let mut player_connection = PlayerConnection::new(unbounded_sink);
                let inner_sink = unbounded_sink.clone();
                stream
                    .take_while(|m| Ok(!m.is_close()))
                    .filter_map(move |m| {
                        //server_state.borrow_mut().create_game();
                        println!("Message from Client: {:?}", m);
                        match m {
                            OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
                            OwnedMessage::Pong(_) => None,
                            OwnedMessage::Text(text) => {
                                println!("Cmd: {}", text);
                                let msg: proto::CmdMsg = serde_json::from_str(&text).unwrap();
                                let reply = player_connection.process_msg(msg);
                                let j = serde_json::to_string(&reply).unwrap();
                                println!("Reply: {}", j);
                                Some(OwnedMessage::Text(j))
                            }
                            _ => Some(m),
                        }
                    })
                    .forward(
                        inner_sink
                            .map_err(|_err| Error::new(ErrorKind::Other, "Another stream error")),
                    )
                    .and_then(|(_, inner_sink)| inner_sink.send(OwnedMessage::Close(None)))
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
