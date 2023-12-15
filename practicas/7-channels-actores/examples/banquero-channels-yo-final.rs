use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};

const INVERSORES: i32 = 5;
const FONDOS: f32 = 10000f32;

fn main() {
    let (devolucion_tx, devolucion_rx) = mpsc::channel();

    let mut inversion_txs = vec![];
    let mut join_handlers = vec![];

    for id in 0..INVERSORES {
        let (inversion_tx, inversion_rx) = mpsc::channel();
        inversion_txs.push(inversion_tx);
        let devolucion_rx_clone = devolucion_tx.clone();
        join_handlers.push(thread::spawn(move || inversor(id, devolucion_rx_clone, inversion_rx)))
    };

    let mut fondos = FONDOS;

    while fondos > 100f32 {
        let repartir = fondos / (INVERSORES as f32);
        println!("[BANQUERO] comienzo semana con {}", fondos);

        for inversion_tx in inversion_txs.iter() {
            inversion_tx.send(repartir).unwrap();
        }
        fondos = 0f32;

        let mut respondieron = vec![];
        println!("[BANQUERO] espero resultados");
        while respondieron.len() < INVERSORES as usize {
            let (id, resultado) = devolucion_rx.recv().unwrap();
            if !respondieron.contains(&id) {
                println!("[BANQUERO] recibo resultado de {}", id);
                fondos += resultado;
                respondieron.push(id);
            }
        }

    }

    let _ = join_handlers.into_iter().flat_map(|x| x.join());

}

fn inversor(id: i32, devolucion_tx: Sender<(i32, f32)>, inversion_rx: Receiver<f32>) {
    loop {
        let inversion = inversion_rx.recv().unwrap();

        println!("[INVERSOR] {} recibo inversion {}", id, inversion);
        let resultado = inversion * (thread_rng().gen_range(0.9, 1.1));

        thread::sleep(Duration::from_millis(1000));

        println!("[INVERSIOR] {} entrego resultado {}", id, resultado);
        devolucion_tx.send((id, resultado)).unwrap();
    }
}
