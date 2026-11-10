use anyhow::{Ok, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Read, Write};
use std::process::Command;
use tokio;
use windows_sys::Win32::System::Console::{
    CONSOLE_SCREEN_BUFFER_INFO, COORD, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
    ENABLE_PROCESSED_INPUT, FillConsoleOutputCharacterW, GetConsoleMode,
    GetConsoleScreenBufferInfo, GetStdHandle, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    SetConsoleCursorPosition, SetConsoleMode, WriteConsoleW,
};

fn init_tui() -> Result<()> {
    println!("Initializing the tui application");

    unsafe {
        let handler = GetStdHandle(STD_INPUT_HANDLE);
        println!("handler: {:?}", handler);
        let mut org_mode: u32 = 0;
        GetConsoleMode(handler, &mut org_mode);
        println!("org_mode: {:?}", org_mode); // current mode of 503 is fully cooked mode
        let raw_mode = org_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
        // let raw_mode = org_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT); // i still want ctrl+c to break out
        SetConsoleMode(handler, raw_mode);

        let buffer = &mut [0_u8];
        loop {
            io::stdin().read_exact(buffer)?;
            let character = buffer[0];
            print!("{}", String::from_utf8(vec![character])?);
            io::stdout().flush()?;

            // command
            if character == b'/' {
                render_tool()?;

            // command send chat
            } else if character == b'\r' {
                println!("newline");
                break;
            // command send chat
            } else if character == b'@' {
                println!("u want file?");
            }
        }

        // set back to normal before exit
        SetConsoleMode(handler, org_mode);
    }

    Ok(())
}

fn clear_current_row() -> Result<()> {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut csbi = std::mem::zeroed::<CONSOLE_SCREEN_BUFFER_INFO>();
        GetConsoleScreenBufferInfo(handle, &mut csbi);
        let width = csbi.dwSize.X;
        let cursor_row = csbi.dwCursorPosition.Y;

        let mut char_written = 0;
        FillConsoleOutputCharacterW(
            handle,
            b' ' as u16,
            width as u32,
            COORD {
                X: 0,
                Y: cursor_row,
            },
            &mut char_written,
        );
    }
    Ok(())
}
// struct CONSOLE_SCREEN_BUFFER_INFO
// pub dwSize: COORD,
// pub dwCursorPosition: COORD,
// pub wAttributes: CONSOLE_CHARACTER_ATTRIBUTES,
// pub srWindow: SMALL_RECT,
// pub dwMaximumWindowSize: COORD,
//
// pub struct COORD {
//     pub X: i16,
//     pub Y: i16,
// }

fn render_tool() -> Result<()> {
    let tools = vec!["grill-me", "superpower", "TDD"];

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut csbi = std::mem::zeroed::<CONSOLE_SCREEN_BUFFER_INFO>();
        GetConsoleScreenBufferInfo(handle, &mut csbi);
        let org_coord = csbi.dwCursorPosition;
        let length = tools.len() as i16;
        let cur_row = org_coord.Y - length;
        for i in 0..length {
            SetConsoleCursorPosition(
                handle,
                COORD {
                    X: 0,
                    Y: cur_row + i,
                },
            );
            clear_current_row()?;

            let tool_name = tools[i as usize];
            let utf16: Vec<u16> = tool_name.encode_utf16().collect();
            let mut written = 0;
            WriteConsoleW(
                handle,
                utf16.as_ptr(),
                utf16.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
        SetConsoleCursorPosition(handle, org_coord);
    }

    Ok(())
}

// org_mode is 503
// Decimal Value	Constant Name	Status in 503	What It's Doing For You
// 1	ENABLE_PROCESSED_INPUT	ON	Allows the OS to process shortcuts like Ctrl+C.
// 2	ENABLE_LINE_INPUT	    ON	Line-buffering is active; commands wait for Enter.
// 4	ENABLE_ECHO_INPUT	    ON	Characters appear on your screen as you type them.   4 depends on 2, if 2 off, 4 is off too -> manually echo out
// 8	ENABLE_WINDOW_INPUT	    OFF	Disabled. Script ignores window resizing loops.
// 16	ENABLE_MOUSE_INPUT	    ON	Allows the console to listen for mouse clicks/scrolls.
// 32	ENABLE_INSERT_MODE	    ON	Typed text inserts smoothly rather than overwriting text.
// 64	ENABLE_QUICK_EDIT_MODE	ON	Lets you use your mouse to highlight, copy, and paste text.
// 128	ENABLE_EXTENDED_FLAGS	ON	Required by the OS to manage features like QuickEdit.
// 256	ENABLE_AUTO_POSITION	ON	Allows the console window to automatically snap and position itself.
// 512	ENABLE_VIRTUAL_TERMINAL_INPUT	OFF	Disabled. Traditional keyboard input (no ANSI/VT streams).

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
    println!("[DEBUG] listing model now");
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

fn check_keystroke() -> Result<bool> {
    //wtf is this bro
    println!("checking keyyyy");
    let buffer = &mut [0_u8];
    loop {
        io::stdin().read_exact(buffer)?;

        if buffer[0] == b'/' {
            println!("u want command?");
        } else if buffer[0] == b'\n' {
            break;
        }
    }
    Ok(true)
}

fn get_input() -> Result<String> {
    let mut buffer = String::new();

    print!("user: ");
    io::stdout().flush().unwrap(); // flush the print code or else it will wait til flush later

    check_keystroke();

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

struct Ollama {}

trait Completable {
    async fn complete(&self, history: &mut Vec<Chat>, input: String) -> Result<String>;
}

impl Completable for Ollama {
    async fn complete(&self, history: &mut Vec<Chat>, input: String) -> Result<String> {
        let chat = build_chat(history, input)?;

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
}

fn render_response(string: &str) -> Result<String> {
    // wait for idk render on the screen i guess
    let out = String::from(string);
    Ok(out)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    init_tui()?;

    //     let llm = Ollama {};
    //     check_ollama()?;
    //     println!("Model Shop:");
    //     ollama_list_model()?;
    //
    //     let mut history: Vec<Chat> = Vec::new();
    //     loop {
    //         let user_input = get_input()?;
    //         if user_input == "exit" {
    //             break;
    //         } else if user_input == "ping" {
    //             check_ollama()?;
    //             continue;
    //         }
    //
    //         let response = llm.complete(&mut history, user_input).await?;
    //
    //         println!("before format: {}", response);
    //         let format_res = render_response(&response)?;
    //         println!("Wendy: {}", format_res);
    //     }

    Ok(())
}
