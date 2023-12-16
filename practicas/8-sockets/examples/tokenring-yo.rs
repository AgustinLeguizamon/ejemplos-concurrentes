use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};

const CLIENTES: usize = 5;

fn id_to_addr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

struct DistMutex {
    id: usize,
    socket: UdpSocket,
    lock_pedido: Arc<(Mutex<bool>, Condvar)>,
    tiene_token: Arc<(Mutex<bool>, Condvar)>


}

impl DistMutex {
    fn new(id: usize) -> DistMutex {
        let ret = DistMutex {
            id,
            socket: UdpSocket::bind(id_to_addr(id)).unwrap(),
            lock_pedido: Arc::new((Mutex::new(false), Condvar::new())),
            tiene_token: Arc::new((Mutex::new(false), Condvar::new())),
        };

        let clone = ret.clone();

        thread::spawn(move || clone.receiver());

        ret
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            lock_pedido: self.lock_pedido.clone(),
            tiene_token: self.tiene_token.clone(),
        }

    }

    fn acquire(&self) {
        *self.lock_pedido.0.lock().unwrap() = true;
        // TODO: Pregunta, entiendo que al modificar la variable del Monitor es normal hacer un notify_all
        // pero no es innecesario en este caso? dado que el cvar pide que lock_pedido sea true
        // En clase creo que lo mencionaron y sería abusar del dominio del problema, es buena practica meter el notify
        // Rta: Puedo comentar esto y sigue funcionando igual
        // self.lock_pedido.1.notify_all();

        self.tiene_token.1.wait_while(self.tiene_token.0.lock().unwrap(), |lo_tiene| !*lo_tiene);
    }

    fn release(&self) {
        *self.lock_pedido.0.lock().unwrap() = false;
        self.lock_pedido.1.notify_all();
    }

    fn receiver(&self) {
        // Para iniciar el pasaje de token
        if self.id == 0 {
            self.socket.send_to("TOKEN".as_bytes(), id_to_addr(0)).unwrap();
        }
        loop {
            let mut buf = [0; size_of::<u128>()];
            let (read, addr) = self.socket.recv_from(&mut buf).unwrap();
            println!("[{}] recibí token", self.id);
            *self.tiene_token.0.lock().unwrap() = true;
            self.tiene_token.1.notify_all();
            // La razon pq uso este condvar es para pasars el token
            // inmediatamente si es que no lo llego a necesitar pq nunca lo pedi
            self.lock_pedido.1.wait_while(self.lock_pedido.0.lock().unwrap(), |lo_necesita|*lo_necesita).unwrap();
            *self.tiene_token.0.lock().unwrap() = false;
            // TODO: Pregunta, entiendo que al modificar la variable del Monitor es normal hacer un notify_all
            // pero no es innecesario en este caso? dado que el cvar pide que tiene_token sea true
            // En clase creo que lo mencionaron y sería abusar del dominio del problema, es buena practica meter el notify
            // Rta: Puedo comentar esto y sigue funcionando igual
            // self.tiene_token.1.notify_all();
            thread::sleep(Duration::from_millis(100));
            self.socket.send_to("TOKEN".as_bytes(), id_to_addr((self.id + 1) % CLIENTES)).unwrap();


        }

    }
}

fn main() {
    let mut handlers = vec![];
    for id in 0..CLIENTES {
        handlers.push(thread::spawn(move || cliente(id)))
    }

    for handler in handlers.into_iter() {
        handler.join().unwrap();
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