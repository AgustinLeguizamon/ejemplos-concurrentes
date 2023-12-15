use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use std_semaphore::Semaphore;

struct DistMutex {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl DistMutex {
    fn new(id: i32) -> DistMutex {
        let mut writer = TcpStream::connect(LIDER_ADDR).unwrap();
        let mut reader = BufReader::new(writer.try_clone().unwrap());

        writer.write_all((id.to_string() + "\n").as_bytes());

        DistMutex { writer, reader}
    }

    fn acquire(&mut self) {
        self.writer.write_all("acquire\n".as_bytes()).unwrap();

        let mut buffer = String::new();
        self.reader.read_line(&mut buffer).unwrap();
    }

    fn release(&mut self) {
        self.writer.write_all("release\n".as_bytes()).unwrap();
    }
}

const CLIENTES: i32 = 3;

const LIDER_ADDR: &str = "127.0.0.1:12345";

fn main() {

    let coordinador = thread::spawn(move || coordinador());

    let mut clientes_handler = vec![];
    for id in 0..CLIENTES {
        clientes_handler.push(thread::spawn(move || cliente(id)))
    }

    for i in clientes_handler.into_iter() {
        let _ = i.join();
    }

    let _ = coordinador.join();
}

fn coordinador() {
    let listener = TcpListener::bind(LIDER_ADDR).unwrap();

    let mutex = Arc::new(Semaphore::new(1));

    for opt_stream in listener.incoming() {
        if let Ok(stream) = opt_stream {
            let writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream.try_clone().expect(""));
            let local_mutex = mutex.clone();
            let mut id = String::new();
            // El cliente me dice su id al conectarse
            reader.read_line(&mut id).unwrap();
            let id_cliente = id.replace("\n", "");
            println!("[COORDINATOR] cliente {} conectado", id_cliente);
            thread::spawn(move || handle_cliente(writer, reader, local_mutex, id_cliente));
        }
    }
}

fn handle_cliente(mut stream: TcpStream, mut reader: BufReader<TcpStream>, local_mutex: Arc<Semaphore>, id_cliente: String) {
    let mut mine = false;

    loop {
        let mut buffer = String::new();
        if let Ok(read) = reader.read_line(&mut buffer) {
            match buffer.as_str() {
                "acquire\n" => {
                    if !mine {
                        println!("[COORDINADOR] cliente {} me pidio el lock", id_cliente);
                        // en vez de usar mutex y condvar uso un semaforo cuidandome de no hacer mas de un release
                        local_mutex.acquire();
                        stream.write_all("OK\n".as_bytes());
                        mine = true;
                        println!("[COORDINADOR] se lo dia a cliente {}", id_cliente);
                    }
                }
                "release\n" => {
                    println!("[COORDINADOR] cliente {} hizo release", id_cliente);
                    if mine {
                        mine = false;
                        local_mutex.release();
                    }
                }
                "" => {
                    println!("[COORDINATOR] desconectado {}", id_cliente);
                    break;
                }
                _ => {
                    println!("[COORDINATOR] ERROR: mensaje desconocido de {}", id_cliente);
                    break;
                }
            }
        }
    }

    if mine {
        println!("[COORDINATOR] ERROR: {} tenia el lock. Liberación forzosa", id_cliente);
        local_mutex.release()
    }
}

fn cliente(id: i32) {
    let mut mutex = DistMutex::new(id);

    let mut count = 0;

    loop {
        println!("[{}] pide el lock", id);
        mutex.acquire();

        println!("[{}] lo obtiene", id);
        thread::sleep(Duration::from_millis(1500));

        if count > 2 {
            break;
        }

        println!("[{}] lo libera", id);
        mutex.release();

        count += 1;
    }
}