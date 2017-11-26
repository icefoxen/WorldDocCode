#[macro_use]
extern crate rouille;
extern crate lazy_static;
extern crate serde;
extern crate rustc_serialize;
extern crate ring;
extern crate untrusted;
extern crate base64;


use std::collections::HashMap;
use std::sync::RwLock;

use rouille::Response;

extern crate protocol;
use protocol::*;

use ring::{signature, rand};


#[derive(Debug, Default, Clone)]
struct ServerData {
    names: HashMap<String, UpdateMessage>,
    keys: HashMap<String, Vec<u8>>,
}

impl ServerData {
    fn get_name(&self, name: &str) -> Option<&UpdateMessage> {
        self.names.get(name)
    }

    fn get_id_key(&self, id: &str) -> Option<&[u8]> {
        self.keys.get(id).map(|x| x.as_ref())
    }

    fn add_id(&mut self, id: &str, key: &[u8]) {
        self.keys.insert(id.into(), key.into());
    }

    fn validate_update(&self, msg: &UpdateMessage) -> Result<(), ValidationError> {
        match self.keys.get(&msg.user) {
            Some(key) => msg.verify_signature(key),
            None => Err(ValidationError::UnknownUser(msg.user.clone())) 
        }
    }

    fn update_name(&mut self, name: &str, contents: &UpdateMessage) {
        self.names.insert(name.to_string(), contents.clone());
    }

    fn apply_update_if_valid(&mut self, dest: &str, msg: &UpdateMessage) -> Result<(), ValidationError> {
        let _ = self.validate_update(msg)?;
        self.update_name(dest, &msg);
        Ok(())
    }

    fn add_user(&mut self, username: &str) {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let keypair = signature::Ed25519KeyPair::from_pkcs8(
            untrusted::Input::from(&pkcs8_bytes)
        ).unwrap();

        let encoded_privkey = base64::encode(&pkcs8_bytes[..]);
        println!("Private key for {} is: {}", username, encoded_privkey);
        
        let pubkey_bytes = keypair.public_key_bytes();
        self.add_id(username, pubkey_bytes);

    }

    fn run(server: ServerData, addr: &str) {
        let server = RwLock::new(server);
        server.write().unwrap().add_user("icefox");
        rouille::start_server(addr, move |request| {
            router!(
                request,
                (GET) (/id/{name:String}) => {
                    if let Some(n) = server.read().unwrap().get_id_key(&name) {
                        Response::text(base64::encode(n))
                    } else {
                        Response::empty_404()
                    }
                },
                (GET) (/name/{name:String}) => {
                    println!("Got get to {}", &name);
                    if let Some(n) = server.read().unwrap().get_name(&name) {
                        Response::json(n)
                    } else {
                        Response::empty_404()
                    }
                },
                (POST) (/name/{name:String}) => {
                    println!("Got post to {}", &name);
                    let rename_request: UpdateMessage = try_or_400!(rouille::input::json_input(request));
                    println!("Got post to {}: {:?}", &name, rename_request);
                    match server.write().unwrap().apply_update_if_valid(&name, &rename_request) {
                        Ok(_) => Response::text("ok"),
                        Err(v) => Response::text(format!("{:?}", v)).with_status_code(403),
                    }
                },
                _ => Response::text("hello world")
            )
        });
    }
}

fn main() {
    let s = ServerData::default();
    ServerData::run(s, "127.0.0.1:8888");
}


#[cfg(test)]
mod tests {
    extern crate reqwest;
    use lazy_static;
    use std::thread;
    use std::io::Read;
    use serde::Serialize;
    use ring::{rand, signature};
    use untrusted;
    use base64;

    const UNITTEST_USER: &str = "unittest_user";
    const UNITTEST_NAME: &str = "unittest_name";
    const UNITTEST_NAME_VALUE: &str = "unittest_name_value";

    fn start_test_server() {
        use super::ServerData;
        let mut s = ServerData::default();
        let pubkey_bytes = KEYPAIR.public_key_bytes();
        s.add_id(UNITTEST_USER, pubkey_bytes);
        s.update_name(UNITTEST_NAME, UNITTEST_NAME_VALUE);
        ServerData::run(s, "127.0.0.1:8888");

    }

    fn generate_keypair() -> signature::Ed25519KeyPair {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let keypair = signature::Ed25519KeyPair::from_pkcs8(
            untrusted::Input::from(&pkcs8_bytes)
        ).unwrap();
        keypair
    }

    lazy_static! {
        static ref SERVER_THREAD: thread::JoinHandle<()> = thread::spawn(start_test_server);
        static ref KEYPAIR: signature::Ed25519KeyPair = generate_keypair();
    }


    fn spawn_server_and_get(path: &str) -> reqwest::Response {
        lazy_static::initialize(&SERVER_THREAD);
        let new_path = String::from("http://localhost:8888") + path;
        reqwest::get(&new_path).unwrap()
    }

    fn spawn_server_and_post<T: Serialize>(path: &str, json: &T) -> reqwest::Response {
        lazy_static::initialize(&SERVER_THREAD);
        let client = reqwest::Client::new().unwrap();
        let new_path = String::from("http://localhost:8888") + path;
        client.post(&new_path).unwrap()
            .json(json).unwrap()
            .send().unwrap()
    }

    #[test]
    fn test_basic() {
        let mut resp = spawn_server_and_get("/");
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_id() {
        let mut resp = spawn_server_and_get((String::from("/id/") + UNITTEST_USER).as_str());
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        let pubkey_bytes = KEYPAIR.public_key_bytes();
        let pubkey_string = base64::encode(pubkey_bytes);        
        assert_eq!(content, pubkey_string);
    }

    #[test]
    fn test_get_name() {
        // Test unset name default
        let resp = spawn_server_and_get("/name/test_no_name");
        assert_eq!(resp.status(), reqwest::StatusCode::NotFound);

        // Test set name
        let mut resp = spawn_server_and_get((String::from("/name/") + UNITTEST_NAME).as_str());
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        assert_eq!(content, UNITTEST_NAME_VALUE);
    }

    
    #[test]
    fn test_post_name() {
        const NEWNAME: &str = "/name/test_post_name";
        // See that name DNE
        let resp = spawn_server_and_get(NEWNAME);
        assert!(!resp.status().is_success());

        let changed_name = "foo!";
        let data = super::UpdateMessage::signed_message(&KEYPAIR, UNITTEST_USER, changed_name);

        // Change name
        let mut resp = spawn_server_and_post(NEWNAME, &data);
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        assert_eq!(content, "ok");
        
        // Test name now that it's been changed
        let mut resp = spawn_server_and_get(NEWNAME);
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        assert_eq!(content, changed_name);

        // Try changing it again with unsigned request
        let baddata = super::UpdateMessage {
            user: UNITTEST_USER.into(),
            signature: "".into(),
            new_contents: "aieeee!".into(),
        };
        let resp = spawn_server_and_post(NEWNAME, &baddata);
        assert!(!resp.status().is_success());

        // Ensure it hasn't changed.
        let mut resp = spawn_server_and_get(NEWNAME);
        assert!(resp.status().is_success());
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        assert_eq!(content, changed_name);

    }

}
