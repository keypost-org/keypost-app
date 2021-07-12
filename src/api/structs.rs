/// Only non-database structs for API endpoints go here

// To quiet if vscode: https://rust-analyzer.github.io/manual.html#unresolved-macro-call

/// https://github.com/SergioBenitez/Rocket/blob/08e5b6dd0dd9d723ca2bd4488ff4a9ef0af8b91b/examples/json/src/main.rs#L22
#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterStart {
    pub e: String,
    pub i: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterFinish {
    pub id: u32,
    pub e: String,
    pub i: String,
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
