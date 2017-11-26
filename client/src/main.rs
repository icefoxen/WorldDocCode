extern crate reqwest;
extern crate rustyline;
extern crate protocol;
extern crate base64;
extern crate ring;
extern crate untrusted;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use protocol::*;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::str;
use std::io::Read;
use ring::{signature, rand};

type ClientState = Option<Connection>;

struct Connection {
    target_server: String,
    username: String,
    key: signature::Ed25519KeyPair,
}

fn do_help() {
    println!("Help!");
    println!("Ok, first thing you do is connect to a server with the 'server' command, like this:");
    println!("server localhost:8888 icefox MFMCAQEwBQYDK2VwBCIEICFMtBQqf3puaJMwdOIHTDfuE5jpTKwaSSSqQKquI5lYoSMDIQC3VOwaNbCzRzRXDPnSyMqgMAREGco+J0oLhDQ0cTj9yg==");
    println!("Obviously you need to be running the server on localhost.  You also need to be running an IPFS node.");
    println!("You can then type 'post', which will ask you for some input, then publish it as an IPFS document and send a request to the server to update the name to that document.")
    println!("You can then type 'get' which will ask the server what the latest document is, and retrieve that from IPFS.")
    println!("Exciting, huh?")
}

fn do_server(args: &mut str::SplitWhitespace) -> ClientState {
    if let (Some(servername), Some(username), Some(keystring)) = (args.next(), args.next(), args.next()) {
        let target_server = String::from(servername);
        let username = String::from(username);
        let pkcs8_bytes = base64::decode(keystring).unwrap();
        let keypair = signature::Ed25519KeyPair::from_pkcs8(
            untrusted::Input::from(&pkcs8_bytes)
        ).unwrap();
        Some(Connection {
            target_server: target_server,
            username: username,
            key: keypair,
        })
    } else {
        println!("Syntax: server <domain> <username> <private key>");
        None
    }
}

const CONVERSATION: &str = "/name/conversation";

fn get_ipfs_doc(name: &str) {
    // Using 'get' here 
    let url = format!("http://localhost:5001/api/v0/cat?arg={}", name);
    let mut resp = reqwest::get(&url).expect("Could not get IPFS doc?");
    let mut content = String::new();
    resp.read_to_string(&mut content).unwrap();
    println!("{}", content);
}

fn do_get(client: &ClientState, args: &mut str::SplitWhitespace) {
    if let Some(ref state) = *client {
        let url = String::from("http://") + state.target_server.as_ref() + CONVERSATION;
        let mut resp = reqwest::get(&url).expect("Error getting URL?");
        let msg: UpdateMessage = resp.json().expect("Error parsing json response?");
        println!("Message set by {} on {} to document {}", &msg.user, &msg.utc, &msg.new_contents);
        // let mut content = String::new();
        // resp.read_to_string(&mut content).unwrap();
        // println!("Got {}", content);
        println!("Fetching IPFS document...");
        get_ipfs_doc(&msg.new_contents);
    } else {
        println!("Not connected to a server!");
    }
}

#[derive(Deserialize, Debug)]
struct IpfsAddResponse {
    Hash: String,
}

/// Adds a chunk of data to IPFS
/// horrifically writing it out to a temp file and posting that
/// and returns a string containing the IPFS hash of the new data.
fn add_data_to_ipfs(data: &str) -> String {
    let url = format!("http://localhost:5001/api/v0/add");
    let client = reqwest::Client::new();
    {
        // Write the stupid stuff out to a file
        use std::fs;
        use std::io::Write;
        let mut f = fs::File::create("tempfile.txt").unwrap();
        f.write(data.as_bytes()).unwrap();
    }
    // use reqwest::header::ContentType;
    // use reqwest::mime;
    // let form = reqwest::multipart::Form::new()
    //     .file("foo", "tempfile.txt").unwrap()
    //     .text("path", "bar");
    // let req = client.post(&url)
    //     .multipart(form)
    //     .body("rawr")
    //     .build().unwrap();
    // println!("Request is: {:?}\nBody is: {:?}", req, req.body());

    let form = reqwest::multipart::Form::new()
        .file("foo", "tempfile.txt").unwrap()
        .text("path", "bar");
    let mut resp = client.post(&url)
        .multipart(form)
        // .body("file")
        // .header(ContentType(mime::MULTIPART_FORM_DATA))
        // .body(data.to_owned())
        .send().unwrap();

    let ipfs_response: IpfsAddResponse = resp.json().unwrap();

    // let mut content = String::new();
    // resp.read_to_string(&mut content).unwrap();
    // println!("Got response: {:?}\n'{:?}'", resp, ipfs_response);
    ipfs_response.Hash
}

fn do_post(client: &ClientState, args: &mut str::SplitWhitespace) {
    if let Some(ref s) = *client {
        let url = String::from("http://") + s.target_server.as_ref() + CONVERSATION;
        let mut rl = Editor::<()>::new();
        let dataline = rl.readline("Enter data to post: ").unwrap();
        let ipfs_hash = add_data_to_ipfs(&dataline);
        let data = UpdateMessage::signed_message(&s.key, &s.username, &ipfs_hash);
        // let data = UpdateMessage {
        //     user: "rawr".into(),
        //     signature: "".into(),
        //     new_contents: "aieeee!".into(),
        // };
        let client = reqwest::Client::new();
        let mut resp = client.post(&url)
            .json(&data)
            .send().expect("Could not send?");
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        println!("Got {}", content);
    } else {
        println!("Not connected to a server!");
    }

}


fn parse_and_do_command(client: &mut ClientState, cmd: &str) {
    let mut tokens = cmd.split_whitespace();
    if let Some(token) = tokens.next() {
        match token {
            "help" => do_help(),
            "server" => {
                let s = do_server(&mut tokens);
                *client = s;
            },
            "get" => do_get(client, &mut tokens),
            "post" => do_post(client, &mut tokens),
            other => println!("Unknown command: {}", other),
        }
    } else {
        // Do nothing.
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    println!("Type 'help' for help.");
    let mut s: ClientState = None;
    loop {
        let mut prompt = match s {
            Some(ref state) => String::from(state.target_server.as_ref()),
            None => String::from("(not connected)"),
        };
        // let mut prompt = String::from(server);
        prompt += " > ";
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                parse_and_do_command(&mut s, &line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("EOF");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("Interrupted");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}