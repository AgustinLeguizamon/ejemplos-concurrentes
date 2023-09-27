use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use rand::{Rng, thread_rng};
use std_semaphore::Semaphore;

const N: usize = 5;

#[derive(PartialEq)]
enum ESTADO {
    PENSANDO,
    HAMBRIENTO,
    COMIENDO,
}

fn main() {
    let filosofos: Arc<Vec<Semaphore>> = Arc::new((0 .. N)
        .map(|_| Semaphore::new(0))
        .collect());

    // Modelo estado de los filosofos como un array con un Rwlock
    // por ahora solo uso un bool para indicar si esta comiendo o no
    let estado_filosofos = Arc::new(RwLock::new(vec![ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO]));

    let threads_filosofos:Vec<JoinHandle<()>> = (0 .. N)
        .map(|id| {
            let filosofos_local = filosofos.clone();
            let estado_filosofos_local = estado_filosofos.clone();
            thread::spawn(move || filosofo(id, filosofos_local, estado_filosofos_local))
        })
        .collect();

    for filosofo in threads_filosofos {
        filosofo.join();
    }
}

fn der(id: usize) -> usize {
    return (id + 1) % N;
}

fn izq(id: usize) -> usize {
    return if id == 0 { N - 1 } else { id - 1 };;
}

fn filosofo(id: usize, filosofos: Arc<Vec<Semaphore>>, estado_filosofos: Arc<RwLock<Vec<ESTADO>>>) {
    let mut esperar = false;
    thread::sleep(Duration::from_millis(100 * id as u64));

    loop {
        println!("filosofo {} pensando", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));

        println!("filosofo {} hambriento", id);
        // si mis vecinos estan comiendos entonces espero
        if let Ok(mut estados_filosofos_guard) = estado_filosofos.write() {
            (*estados_filosofos_guard)[id] = ESTADO::HAMBRIENTO;
            if ((*estados_filosofos_guard)[der(id)] == ESTADO::COMIENDO) || ((*estados_filosofos_guard)[izq(id)] == ESTADO::COMIENDO) {
                esperar = true
            }
        }
        if esperar {
            filosofos[id].acquire();
        }
        esperar = false;

        // otro filoso me tiene que hacer un release a mi semaforo
        println!("filosofo {} comiendo", id);
        if let Ok(mut estados_filosofos_guard) = estado_filosofos.write() {
            (*estados_filosofos_guard)[id] = ESTADO::COMIENDO
        }
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));

        if let Ok(mut estados_filosofos_guard) = estado_filosofos.write() {
            (*estados_filosofos_guard)[id] = ESTADO::PENSANDO;
            // si el que esta a mi derecha esta hambriento y el que esta a su derecha NO esta comiendo entonces le puedo dar el palito
            if ((*estados_filosofos_guard)[der(id)] == ESTADO::HAMBRIENTO) && ((*estados_filosofos_guard)[der(der(id))] != ESTADO::COMIENDO) {
                // libero el semaforo del filosofo a mi derecha
                filosofos[der(id)].release()
            }
            if ((*estados_filosofos_guard)[izq(id)] == ESTADO::HAMBRIENTO) && ((*estados_filosofos_guard)[izq(izq(id))] != ESTADO::COMIENDO) {
                filosofos[izq(id)].release()
            }
        }

        println!("filosofo {} termino de comer", id);
    }

}

