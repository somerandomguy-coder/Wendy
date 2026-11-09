use anyhow::{Ok, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use std::process::Command;
use tokio;

fn check_ollama() -> Result<()> {
    let output = Command::new("curl")
        .arg("http://localhost:11434")
        .output()
        .expect("not recognized as an internal or external command");

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("ollama not on, serve ollama again");
        serve_ollama()?;
    }
    Ok(())
}

fn serve_ollama() -> Result<()> {
    println!("ollama, wakey wakey");

    unsafe {
        env::set_var("OLLAMA_ORIGINS", "*");
    }
    println!("allow everyone in");

    Command::new("ollama")
        .arg("serve")
        .spawn()
        .expect("not recognized as an internal or external command");

    println!("finish the serve now");
    Ok(())
}

fn ollama_list_model() -> Result<()> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .expect("not recognized as an internal or external command");

    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        println!("ollama not on");
    }

    Ok(())
}

fn get_input() -> Result<String> {
    let mut buffer = String::new();
    print!("user: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut buffer)?;
    buffer = String::from(buffer.trim());
    Ok(buffer)
}

#[derive(Serialize, Debug, Clone)]
struct Chat {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct Request {
    model: String,
    messages: Vec<Chat>,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct AIChat {
    content: String,
}

#[derive(Deserialize, Debug)]
struct CustomResponse {
    message: AIChat,
}

fn build_chat(history: &mut Vec<Chat>, req: String) -> Result<&Vec<Chat>> {
    let chat = Chat {
        role: String::from("user"),
        content: req,
    };

    history.push(chat);
    Ok(history)
}

async fn get_response(history: &mut Vec<Chat>, req: String) -> Result<String> {
    let chat = build_chat(history, req)?;

    let map = Request {
        model: String::from("qwen3.5:4b"),
        messages: chat.to_vec(),
        stream: false,
    };

    // println!("map: {:?}", map);

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&map)
        .send()
        .await?;

    // full response
    // println!("res: {:?}", response);
    let res_out = response.text().await?;

    let res_unmarshal = serde_json::from_str::<CustomResponse>(&res_out)?;
    // println!("res_unmarshal: {:?}", res_unmarshal);

    Ok(res_unmarshal.message.content)
}

struct LLM {}

trait Completable {
    async fn complete(&self, history: &mut Vec<Chat>, input: String) -> Result<String>;
}

impl Completable for LLM {
    async fn complete(&self, history: &mut Vec<Chat>, input: String) -> Result<String> {
        let res = get_response(history, input).await?;
        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    let llm = LLM {};
    check_ollama()?;
    ollama_list_model()?;

    let mut history: Vec<Chat> = Vec::new();
    loop {
        let user_input = get_input()?;
        if user_input == "exit" {
            break;
        }
        let response = llm.complete(&mut history, user_input).await?;
        println!("Wendy: {:?}", response);
    }

    Ok(())
}
