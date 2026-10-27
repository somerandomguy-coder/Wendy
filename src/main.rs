use anyhow::Result;
use std::io;
use std::{thread, time::Duration};

fn get_input() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    buffer = String::from(buffer.trim());
    Ok(buffer)
}

struct LLM {}

trait Completable {
    fn complete(&self, input: String) -> Result<String>;
}

impl Completable for LLM {
    fn complete(&self, input: String) -> Result<String> {
        thread::sleep(Duration::from_millis(500));
        let response: String = String::from("ai generate this hehe");
        Ok(response)
    }
}

fn main() -> Result<()> {
    println!("Hello, world!");
    let llm = LLM {};
    while true {
        let user_input = get_input()?;
        let response = llm.complete(user_input);
        println!("response is: {:?}", response)
    }

    Ok(())
}
