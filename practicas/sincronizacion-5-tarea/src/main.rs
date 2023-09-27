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
    let palitos:Arc<Vec<Semaphore>> = Arc::new((0 .. N)
        .map(|_| Semaphore::new(1))
        .collect());

    // Modelo estado de los filosofos como un array con un Rwlock
    // por ahora solo uso un bool para indicar si esta comiendo o no
    let estado_filosofos = Arc::new(RwLock::new(vec![ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO, ESTADO::PENSANDO]));


    let filosofos:Vec<JoinHandle<()>> = (0 .. N)
        .map(|id| {
            let palitos_local = palitos.clone();
            let estado_filosofos_local = estado_filosofos.clone();
            thread::spawn(move || filosofo(id, palitos_local, estado_filosofos_local))
        })
        .collect();

    for filosofo in filosofos {
        filosofo.join();
    }
}

fn filosofo(id: usize, palitos_local: Arc<Vec<Semaphore>>, estado_filosofos_local: Arc<RwLock<Vec<ESTADO>>>) {
    loop {
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("filosofo {} pensando", id);
        let der = (id + 1) % 5;
        let izq = if id == 0 { N - 1 } else { id - 1 };
        // si resulta que mis vecinos estan esperando palitos entonces no intento tomar ninguno
        if let Ok(mut estados_filosofos_guard) = estado_filosofos_local.write() {
            if (*estados_filosofos_guard)[der] == ESTADO::HAMBRIENTO && (*estados_filosofos_guard)[izq] == ESTADO::HAMBRIENTO {
                continue
            }
        }

        if let Ok(mut estados_filosofos_guard) = estado_filosofos_local.write() {
            (*estados_filosofos_guard)[id] = ESTADO::HAMBRIENTO
        }

        println!("filosofo {} esperando palito izq", id);
        let palito_izq_guard = &palitos_local[id].access();

        println!("filosofo {} esperando palito der", id);
        let palito_der_guard = &palitos_local[(id + 1) % N].access();

        if let Ok(mut estados_filosofos_guard) = estado_filosofos_local.write() {
            (*estados_filosofos_guard)[id] = ESTADO::COMIENDO
        }

        println!("filosofo {} comiendo", id);

        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));

        if let Ok(mut estados_filosofos_guard) = estado_filosofos_local.write() {
            (*estados_filosofos_guard)[id] = ESTADO::PENSANDO
        }

        println!("filosofo {} termino de comer", id);
    }

}

