use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
};

struct Client {
    nick: String,
    conn: TcpStream,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            nick: self.nick.clone(),
            conn: self.conn.try_clone().unwrap(),
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.nick == other.nick
    }
}

struct ChatState {
    listener: TcpListener,
    size: u32,
    clients: Arc<RwLock<Vec<Client>>>,
}

impl Clone for ChatState {
    fn clone(&self) -> Self {
        ChatState {
            listener: self.listener.try_clone().unwrap(),
            size: self.size,
            clients: self.clients.clone(),
        }
    }
}

fn handle(mut client: Client, chat: &mut ChatState) {
    chat.size += 1;
    chat.clients.write().unwrap().push(client.clone());

    println!("[{}] joined.", client.nick);

    client.conn.write_all(b"Welcome to Simple chat!\n").unwrap();
    client
        .conn
        .write_all(b"Use /nick <nick> to set your nick.\n")
        .unwrap();

    let mut buf: Vec<u8> = vec![0; 1024];
    loop {
        match client.conn.read(&mut buf) {
            Ok(n) => {
                let content = std::str::from_utf8(&buf[..n]).unwrap().trim();
                // use `/nick <nick>` to set nick.
                // otherwise, send message.
                if buf.starts_with(b"/nick") {
                    let nick = content.split(" ").collect::<Vec<_>>()[1];
                    let mut clients = chat.clients.write().unwrap();
                    for c in clients.iter_mut() {
                        if c.nick == client.nick {
                            c.nick = nick.to_string();
                            break;
                        }
                    }
                    println!("[{}] change nick to [{}]", nick, client.nick);

                    client.nick = nick.to_string();
                } else if buf.starts_with(b"/quit") {
                    let mut clients = chat.clients.write().unwrap();
                    clients.retain(|x| *x != client);

                    println!("[{}] quited.", client.nick);
                    break;
                } else {
                    let mut clients = chat.clients.write().unwrap();
                    for oc in clients.iter_mut() {
                        // if oc.nick != client.nick {
                        if *oc != client {
                            let msg: String = format!("{}> {}\n", client.nick, content);
                            oc.conn.write_all(msg.as_bytes()).unwrap();
                        }
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn main() {
    let chat = ChatState {
        listener: TcpListener::bind("127.0.0.1:7711").unwrap(),
        size: 0,
        clients: Arc::new(RwLock::new(Vec::new())),
    };

    loop {
        let (conn, _) = chat.listener.accept().unwrap();

        let client = Client {
            nick: conn.peer_addr().unwrap().port().to_string(),
            conn: conn,
        };

        let mut chat = chat.clone();
        thread::spawn(move || handle(client, &mut chat));
    }
}
