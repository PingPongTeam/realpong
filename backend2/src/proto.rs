#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Cmd {
    CreateGame,
    // Join game by player id
    JoinGame(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameDesc {
    pub p1_id: String,
    pub p2_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Reply {
    Error(String),
    Game(GameDesc),
    Ok,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CmdMsg {
    pub id: String,
    pub cmd: Cmd,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReplyMsg {
    pub id: String,
    pub reply: Reply,
}
