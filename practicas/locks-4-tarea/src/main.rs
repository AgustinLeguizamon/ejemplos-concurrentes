use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const COSTO_ORO_UNIDAD_MADERA: i32 = 1;
const COSTO_ORO_UNIDAD_ACERO: i32 = 2;
const COSTO_ORO_UNIDAD_PLATA: i32 = 4;

const GENERACION_ACERO_BASE: i32 = 2;
const GENERACION_MADERA_BASE: i32 = 3;
const GENERACION_PLATA_BASE: i32 = 1;

const COSTO_ACERO_HERRAMIENTA: i32 = 2;
const COSTO_MADERA_HERRAMIENTA: i32 = 2;
// Joya = 2 de plata y 1 de acero = 2 * 4 + 1 * 2 = 9
const COSTO_PLATA_JOYA: i32 = 2;
const COSTO_ACERO_JOYA: i32 = 1;
const COSTO_HERRAMIENTA: i32 = COSTO_ACERO_HERRAMIENTA * COSTO_ORO_UNIDAD_ACERO + COSTO_MADERA_HERRAMIENTA * COSTO_ORO_UNIDAD_MADERA;
const COSTO_JOYA: i32 = COSTO_PLATA_JOYA * COSTO_ORO_UNIDAD_PLATA + COSTO_ACERO_JOYA * COSTO_ORO_UNIDAD_ACERO;
// margen ganacia del 100%
const PRECIO_VENTA_JOYA: i32 = COSTO_JOYA * 2;
const PRECIO_VENTA_HERRAMIENTA: i32 = COSTO_HERRAMIENTA * 2;

const TESORERIA_ORO_INICIAL: i32 = 0;
const ALMACEN_PLATA_INICIAL: i32 = 0;
const ALMACEN_ACERO_INICIAL: i32 = 0;
const ALMACEN_MADERA_INICIAL: i32 = 0;
const ALMACEN_HERRAMIENTAS_INICIAL: i32 = 0;
const ALMACEN_JOYASS_INICIAL: i32 = 0;

/*podemos hacerlo sin limite de recursos asi no agrego tantos locks*/
const MINA_ACERO_INCIAL: i32 = 100000;
const MINA_PLATA_INCIAL: i32 = 100000;
const MADERA_BOSQUE_INCIAL: i32 = 100000;

const MINEROS: u32 = 5;
const HERREROS: u32 = 5;
const ARTESANOS: u32 = 5;


fn main() {
    println!("Hello, world!");

    // creo los locks para cada recurso
    let tesoreria = Arc::new(RwLock::new(TESORERIA_ORO_INICIAL));

    let almacen_plata = Arc::new(RwLock::new(ALMACEN_PLATA_INICIAL));
    let almacen_acero = Arc::new(RwLock::new(ALMACEN_ACERO_INICIAL));
    //let almacen_madera = Arc::new(RwLock::new(ALMACEN_MADERA_INICIAL));

    let almacen_herramientas = Arc::new(RwLock::new(ALMACEN_HERRAMIENTAS_INICIAL));
    let almacen_joyas = Arc::new(RwLock::new(ALMACEN_JOYASS_INICIAL));

    let mineros: Vec<JoinHandle<()>> = (0..MINEROS)
        .map(|id| {
            let almacen_plata_clone = almacen_plata.clone();
            let almacen_acero_clone = almacen_acero.clone();
            thread::spawn(move || minar_acero_y_plata(id, almacen_acero_clone, almacen_plata_clone))
        })
        .collect();

    let artesanos: Vec<JoinHandle<()>> = (0..ARTESANOS)
        .map(|id| {
            let almacen_plata_clone = almacen_plata.clone();
            let almacen_acero_clone = almacen_acero.clone();
            let almacen_joyas_clone = almacen_joyas.clone();
            thread::spawn(move || crear_joyas(id, almacen_acero_clone, almacen_plata_clone, almacen_joyas_clone))
        })
        .collect();

    thread::spawn(move || {
        log(almacen_herramientas, almacen_joyas, tesoreria);
    });

    mineros.into_iter()
        .flat_map(|x| x.join())
        .for_each(drop)


}

fn log(almacen_herramientas: Arc<RwLock<i32>>, almacen_joyas: Arc<RwLock<i32>>, tesoreria: Arc<RwLock<i32>>) {
    loop {
        thread::sleep(Duration::from_millis(5000));
        let mut almacen_joyas_guard = almacen_joyas.read().expect("Fallo en read");
        let mut almacen_herramientas_guard = almacen_herramientas.read().expect("Fallo en read");
        let mut tesoreria_guard = tesoreria.read().expect("Fallo en read");
        println!("Actualmente hay {} herramientas, {} joyas y {} unidades de oro", *almacen_herramientas_guard, *almacen_joyas_guard, *tesoreria_guard);
    }
}

fn minar_acero_y_plata(id:u32, almacen_acero:Arc<RwLock<i32>>, almacen_plata:Arc<RwLock<i32>>) {
    println!("[MINEROS {}] empiezan a trabajar", id);
    loop {
        thread::sleep(Duration::from_millis(5000));
        let acero_minado = GENERACION_ACERO_BASE;
        if let Ok(mut acero_guard) = almacen_acero.write() {
            *acero_guard += GENERACION_ACERO_BASE;
        }

        let plata_minada = GENERACION_PLATA_BASE;
        if let Ok(mut plata_guard) = almacen_plata.write() {
            *plata_guard += GENERACION_PLATA_BASE;
        }

        println!("[MINEROS {}] minaron {} de acero y {} de plata", id, acero_minado, plata_minada);
    }

}

fn crear_joyas(id:u32, almacen_acero:Arc<RwLock<i32>>, almacen_plata:Arc<RwLock<i32>>, almacen_joyas: Arc<RwLock<i32>>) {
    println!("[ARTESANOS {}] empiezan a trabajar", id);
    loop {
        thread::sleep(Duration::from_millis(5000));
        let mut acero_guard = almacen_acero.write().expect("Error en write");
        let mut plata_guard = almacen_plata.write().expect("Error en write");

        let acero_en_almacen = *acero_guard;
        let plata_en_almacen = *plata_guard;

        if acero_en_almacen < COSTO_ACERO_JOYA || plata_en_almacen < COSTO_PLATA_JOYA {
            continue;
        }

        *acero_guard -= COSTO_ACERO_JOYA;
        *plata_guard -= COSTO_PLATA_JOYA;

        if let Ok(mut almacen_joyas_guard) = almacen_joyas.write() {
            *almacen_joyas_guard += 1;
        }

        println!("[ARTESANOS {}] crearon 1 joya", id);
    }

}
