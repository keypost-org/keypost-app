/// Only non-database structs for API endpoints go here

// To quiet if vscode: https://rust-analyzer.github.io/manual.html#unresolved-macro-call

/// https://github.com/SergioBenitez/Rocket/blob/08e5b6dd0dd9d723ca2bd4488ff4a9ef0af8b91b/examples/json/src/main.rs#L22
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterStart {
    pub e: String,
    pub i: String,
    pub c: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterFinish {
    pub id: u32,
    pub e: String,
    pub i: String,
    pub v: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginStart {
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginFinish {
    pub id: u32,
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginVerify {
    pub id: u32,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterLockerStart {
    pub id: String,
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterLockerFinish {
    pub id: String,
    pub e: String,
    pub i: String,
    pub c: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenLockerStart {
    pub id: String,
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenLockerFinish {
    pub id: String,
    pub e: String,
    pub i: String,
    pub n: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteLockerStart {
    pub id: String,
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteLockerFinish {
    pub id: String,
    pub e: String,
    pub i: String,
    pub n: u32,
}
