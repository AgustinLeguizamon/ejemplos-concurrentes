use std::collections::HashSet;
use std::mem::size_of;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::{Rng, thread_rng};

const CLIENTES: usize = 5;

fn id_to_addr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

struct DistMutex {
    id: usize,
    socket: UdpSocket,
    // timestamp y array de todos los que me pidieron entrar en la SC
    // El arc mutex es pq voy a estar escuchando a todos mis pares concurrentemente
    lock_pedido: Arc<Mutex<(Option<u128>, Vec<SocketAddr>)>>,
    // Conjunto de todos los que ya me dieron su OK, la Condvar es para poner al sistema en espera
    // al momento de hacer el acquire
    ok_acc: Arc<(Mutex<HashSet<SocketAddr>>, Condvar)>,
}

impl DistMutex {

    fn new(id: usize) -> DistMutex {
        let socket = UdpSocket::bind(id_to_addr(id)).unwrap();

        let ret = DistMutex {
            id,
            socket,
            lock_pedido: Arc::new(Mutex::new((None, vec![]))),
            ok_acc: Arc::new((Mutex::new(HashSet::new()), Condvar::new()))
        };
        let clone = ret.clone();

        thread::spawn(move || clone.receiver());

        ret
    }

    fn acquire(&self) {
        let timestamp: u128 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        self.lock_pedido.lock().unwrap().0 = Some(timestamp);

        for id_cliente in 0..CLIENTES {
            if id_cliente != self.id {
                self.socket.send_to(&timestamp.to_le_bytes(), id_to_addr(id_cliente)).unwrap();
            }
        }

        println!("[{}] esperando respuestas", self.id);
        let mut ok_acc_guard = self.ok_acc.1.wait_while(self.ok_acc.0.lock().unwrap(), |respuestas| respuestas.len() < (CLIENTES - 1)).unwrap();
        ok_acc_guard.clear();
    }

    fn release(&self) {
        let mut lock_pedido_guard = self.lock_pedido.lock().unwrap();
        lock_pedido_guard.0 = None;
        for addr_cliente in lock_pedido_guard.1.iter() {
            self.socket.send_to("OK".as_bytes(), addr_cliente).unwrap();
            println!("[{}] contesté a {}", self.id, addr_cliente);
        }
        lock_pedido_guard.1.clear();
    }

    fn receiver(&self) {
        loop {
            let mut buf = [0; size_of::<u128>()];
            let (_, addr) = self.socket.recv_from(&mut buf).unwrap();
            if [b'O', b'K'].eq(&buf[0..2]) {
                println!("[{}] recibí OK de {}", self.id, addr);
                self.ok_acc.0.lock().unwrap().insert(addr);
                self.ok_acc.1.notify_all();
            } else {
                let requested_timestamp = u128::from_le_bytes(buf);
                println!("[{}] recibí pedido de {}. timestamp {}", self.id, addr, requested_timestamp);
                let opt_my_timestamp = self.lock_pedido.lock().unwrap().0;
                if let Some(my_timestamp) = opt_my_timestamp {
                    if my_timestamp < requested_timestamp {
                        // Si mi tiempo es menor significa que estoy en la SC o pedi entrar a la SC
                        // antes, en ambos casos no le respondo y lo agrego a mi lista de pendientes
                        self.lock_pedido.lock().unwrap().1.push(addr);
                        println!("[{}] encolando a {}", self.id, addr);
                    } else {
                        self.socket.send_to("OK".as_bytes(), addr).unwrap();
                        println!("[{}] pidió timestamp menor, contesté a {}", self.id, addr);
                    }
                } else {
                    // No me interesa entrar a la SC asi que se lo doy
                    // Esperar para forzar el interleaving
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(500..1000)));
                    self.socket.send_to("OK".as_bytes(), addr).unwrap();
                    println!("[{}] contesté a {}", self.id, addr);
                }
            }
        }
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            lock_pedido: self.lock_pedido.clone(),
            ok_acc: self.ok_acc.clone()

        }
    }
}

fn main() {
    let mut handlers = vec![];

    for id in 0..CLIENTES {
        handlers.push(thread::spawn(move || cliente(id)))
    }

    for handler in handlers {
        handler.join().expect("error al hacer join");
    }

}

fn cliente(id: usize) {
    let mut mutex = DistMutex::new(id);
    println!("[{}] conectado", id);

    loop {
        println!("[{}] durmiendo", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000..3000)));
        println!("[{}] pidiendo lock", id);

        mutex.acquire();
        println!("[{}] tengo el lock", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000..3000)));
        println!("[{}] libero el lock", id);
        mutex.release();
    }
}
