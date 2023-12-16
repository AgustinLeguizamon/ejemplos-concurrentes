use std::convert::TryInto;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};

fn id_to_ctrladdr(id: usize) -> String { "127.0.0.1:1234".to_owned() + &*id.to_string() }
fn id_to_dataaddr(id: usize) -> String { "127.0.0.1:1235".to_owned() + &*id.to_string() }

const TEAM_MEMBERS: usize = 5;
const TIMEOUT: Duration = Duration::from_secs(5);

struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>
}

impl LeaderElection {

    fn new(id: usize) -> LeaderElection {
        let ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
        };

        let mut clone = ret.clone();

        thread::spawn(move || clone.receiver());

        ret.find_new();
        ret
    }

    fn id_to_msg(&self, header:u8) -> Vec<u8> {
        let mut msg = vec!(header);
        msg.extend_from_slice(&self.id.to_le_bytes());
        msg
    }

    fn make_me_leader(&self) {
        println!("[{}] me anuncio como lider", self.id);
        let msg = self.id_to_msg(b'C');
        for id in 0..TEAM_MEMBERS {
            if id != self.id {
                self.socket.send_to(&msg, id_to_ctrladdr(id)).unwrap();
            }
        }
        *self.leader_id.0.lock().unwrap() = Some(self.id);
    }

    fn get_leader_id(&self) -> usize {
        self.leader_id.1.wait_while(self.leader_id.0.lock().unwrap(), |leader_id| leader_id.is_none()).unwrap().unwrap()
    }

    fn send_election(&self) {
        let msg = self.id_to_msg(b'E');
        for id in (self.id + 1)..TEAM_MEMBERS {
            self.socket.send_to(&msg, id_to_ctrladdr(id)).unwrap();
        }
    }

    fn find_new(&self) {
        if *self.stop.0.lock().unwrap() {
            return;
        }

        if self.leader_id.0.lock().unwrap().is_none() {
            return;
        }

        println!("[{}] buscando lider", self.id);
        *self.got_ok.0.lock().unwrap() = false;
        *self.leader_id.0.lock().unwrap() = None;
        self.send_election();

        let got_ok_guard = self.got_ok.1.wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT,|got_oked| !*got_oked, ).unwrap();
        if !*got_ok_guard.0 {
            self.make_me_leader()
        } else {
            // Si alguien me responde dejo que continue el la Election y yo me quedo esperando a que haya un nuevo leader,
            // esperando a que leader id deje de ser None
            self.leader_id.1.wait_while(self.leader_id.0.lock().unwrap(), |leader_id| !leader_id.is_none()).unwrap();
        }

    }

    fn receiver(&self) {
        while !*self.stop.0.lock().unwrap() {
            let mut buf = [0; size_of::<usize>() + 1];
            let (_read, addr) = self.socket.recv_from(&mut buf).unwrap();
            let id_from = usize::from_le_bytes(buf[1..].try_into().unwrap());
            match &buf[0] {
                b'E' => {
                    println!("[{}] recibí Election de {}", self.id, id_from);
                    if self.id > id_from {
                        self.socket.send_to(&self.id_to_msg(b'O'), addr).unwrap();
                        let mut me = self.clone();
                        // Tengo que lanzarlo en otro thread pq si no quedaria bloqueado
                        // y nunca leeria los mensajes de OK o Coordinator de los otros
                        thread::spawn(move || me.find_new());
                    }
                }
                b'O' => {
                    println!("[{}] recibí OK de {}", self.id, id_from);
                    *self.got_ok.0.lock().unwrap() = true;
                    self.got_ok.1.notify_all();
                }
                b'C' => {
                    println!("[{}] recibí nuevo coordinador {}", self.id, id_from);
                    *self.leader_id.0.lock().unwrap() = Some(id_from);
                    self.leader_id.1.notify_all();
                }
                _ => {
                    println!("[{}] ??? {}", self.id, id_from);
                }
            }
        }
        *self.stop.0.lock().unwrap() = false;
        self.stop.1.notify_all();
    }

    fn i_am_leader(&self) -> bool {
        self.id == self.get_leader_id()
    }

    fn stop(&self) {
        *self.stop.0.lock().unwrap() = true;
        // Espero a que el receiver termine (esperando a que me setee stop en false al salir del loop)
        self.stop.1.wait_while(self.stop.0.lock().unwrap(), |stop| *stop);
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ok: self.got_ok.clone(),
            stop: self.stop.clone(),
        }
    }

}

fn main() {
    let mut handles = vec!();
    for id in 0..TEAM_MEMBERS {
        handles.push(thread::spawn(move || { team_member(id) }));
    }
    handles.into_iter().for_each(|h| { h.join(); });
}

fn team_member(id: usize) {
    loop {
        println!("[{}] inicio", id);
        let socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        let mut scrum_master = LeaderElection::new(id);
        let mut buf = [0; 4];

        loop {
            if scrum_master.i_am_leader() {
                println!("[{}] soy SM", id);

                if thread_rng().gen_range(0, 100) >= 80 {
                    println!("[{}] me tomo vacaciones", id);
                    break;
                }

                socket.set_read_timeout(None).unwrap();
                let (_read, addr) = socket.recv_from(&mut buf).unwrap();
                println!("[{}] doy trabajo a {}", id, addr);
                socket.send_to("PONG".as_bytes(), addr).unwrap();
            } else {
                let leader_id = scrum_master.get_leader_id();
                println!("[{}] pido trabajo al SM {}", id, leader_id);
                socket.send_to("PING".as_bytes(), id_to_dataaddr(leader_id)).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();

                if let Ok((read, addr)) = socket.recv_from(&mut buf) {
                    println!("[{}] trabajando", id);
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
                } else {
                    println!("[{}] SM caido, disparo elección", id);
                    scrum_master.find_new();
                }
            }
        }

        scrum_master.stop();

        thread::sleep(Duration::from_secs(30));
    }


}
