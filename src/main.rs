use actix_web::{web, App, HttpResponse, HttpServer, Responder, web::Query};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{prelude::*, BufReader, Write};

use chrono::{Utc, DateTime};
// use serde_json::{Result};

// Json struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    content: String,
    date: String,
}

#[derive(Deserialize)]
pub struct PostMessage {
    content: String,
}

impl Message {
    pub fn new(m: String) -> Message {
        Message {
            content: m,
            date: Utc::now().to_rfc2822(),
        }
    }
}

#[derive(Deserialize)]
pub struct MsgRequest {
    num: i32, // num of request messages (Size?)
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

async fn msg_req(num_of_msg: web::Query<MsgRequest>) -> impl Responder {
    println!("Requested {} num of messages", num_of_msg.num);
    let (msgs, count) = get_messages(num_of_msg.num);
    println!("{} messages receivd from database", count);
    web::Json(msgs)
}

fn get_messages(num_of_msg: i32) -> (Vec<Message>, i32) {
    // get message from database
    // give user messages sorted date from newer
    let datafile = fs::File::open("messages").unwrap();
    let reader = BufReader::new(datafile);

    let mut msgs: Vec<Message> = Vec::new();

    let mut count = 0;
    for line in reader.lines() {
        if count >= num_of_msg {
            break;
        }
        let l = line.unwrap();
        let msg: Message = serde_json::from_str(&l).unwrap();
        msgs.push(msg);
        count += 1;
    }

    (msgs, count)
}

#[test]
fn read_message_works() {
    let req = MsgRequest { num: 3 };
    let (msgs, count) = get_messages(2);
    for msg in msgs {
        println!("{:?}", msg.content);
    }
}

async fn msg_post(messageq: Query<PostMessage>) -> impl Responder{
    let message = Message::new(messageq.into_inner().content);
    post_message(message);

    HttpResponse::Ok().body("successfuly posted message!")
}

fn post_message(message: Message) -> Result<(), String> {
    let mut database = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("messages")
        .unwrap();


    // let json_msg = serde_json::to_string(&message).unwrap();
    println!("{:?}", message);

    println!("{}", serde_json::to_string(&message).unwrap().to_string());

    if let Err(e) = writeln!(database, "{}",serde_json::to_string(&message).unwrap()) {
        eprintln!("Couldn't write to file: {}", e);
        return Err("File error".to_string());
    }
    Ok(())
}

#[test]
fn send_message_works() {
    let message = Message::new(String::from("Hellooooo"));
    post_message(message);
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/msg_req", web::get().to(msg_req))
            .route("/msg_post", web::get().to(msg_post))
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await
}
