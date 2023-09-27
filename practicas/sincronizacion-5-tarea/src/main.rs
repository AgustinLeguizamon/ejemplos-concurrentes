use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use rand::{Rng, thread_rng};
use std_semaphore::Semaphore;

const N: usize = 5;

fn main() {
    let palitos:Arc<Vec<Semaphore>> = Arc::new((0 .. N)
        .map(|_| Semaphore::new(1))
        .collect());

    let filosofos:Vec<JoinHandle<()>> = (0 .. N)
        .map(|id| {
            let palitos_local = palitos.clone();
            thread::spawn(move || filosofo(id, palitos_local))
        })
        .collect();

    for filosofo in filosofos {
        filosofo.join();
    }
}

fn filosofo(id: usize, palitos_local: Arc<Vec<Semaphore>>) {

    println!("filosofo {} pensando", id);

    println!("filosofo {} esperando palito izq", id);
    let palito_izq_guard = &palitos_local[id].access();

    println!("filosofo {} esperando palito der", id);
    let palito_der_guard = &palitos_local[(id + 1) % N].access();

    println!("filosofo {} comiendo", id);
    thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));

    println!("filosofo {} termino de comer", id);
}

