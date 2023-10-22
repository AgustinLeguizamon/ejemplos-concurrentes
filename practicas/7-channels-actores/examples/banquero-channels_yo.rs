extern crate rand;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const INVERSORES: i32 = 10;

fn main() {
    let mut plata = 1000.0;

    let (canal_devolucion_enviar, canal_devolucion_recibir) = mpsc::channel();

    let inversores: Vec<(Sender<f64>, JoinHandle<()>)> = (0..INVERSORES)
        .map(|id|{
            let (canal_inversor_enviar, canal_inversor_recibir) = mpsc::channel();
            let clone_canal_devolucion_enviar = canal_devolucion_enviar.clone();
            let thread_inversor = thread::spawn(move || inversor(id, canal_inversor_recibir, clone_canal_devolucion_enviar));
            (canal_inversor_enviar, thread_inversor)
    }).collect();

    loop {
        let mut plata_semana = iniciar_semana(&mut plata, &inversores);

        let mut devolvieron = HashSet::new();

        while devolvieron.len() < INVERSORES as usize {
            let (id, resultado) = canal_devolucion_recibir.recv().unwrap();
            if !devolvieron.contains(&id) {
                devolvieron.insert(id);
                plata_semana += resultado;
            }
        }

        println!("[Banquero] final de semana {}", plata_semana);
        plata = plata_semana
    }
}

fn iniciar_semana(plata: &mut f64, inversores: &Vec<(Sender<f64>, JoinHandle<()>)>) -> f64 {
    let prestamo_semana = *plata / (INVERSORES as f64);
    for (canal_inversor_enviar, _) in inversores {
        canal_inversor_enviar.send(prestamo_semana).unwrap()
    }
    return 0.0
}

fn inversor(id: i32, prestamo: Receiver<f64>, devolucion: Sender<(i32, f64)>) {
    loop {
        let prestamo_valor = prestamo.recv().unwrap();
        println!("[Inversor {}] me dan {}", id, prestamo_valor);
        thread::sleep(Duration::from_secs(2));
        let resultado = prestamo_valor * thread_rng().gen_range(0.5, 1.5);
        println!("[Inversor {}] devuelvo {}", id, resultado);
        devolucion.send((id, resultado));
    }
}