extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, Addr, AsyncContext, WrapFuture, Recipient, ActorFutureExt, ResponseActFuture};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;

const INVERSORES: usize = 5;

#[derive(Message)]
#[rtype(result = "()")]
struct Invertir {
    amount: f64,
    sender: Recipient<ResultadoInversion>
}

#[derive(Message)]
#[rtype(result = "()")]
struct Semana(f64);

#[derive(Message)]
#[rtype(result = "()")]
struct ResultadoInversion(usize, f64);

struct Banquero {
    plata: f64,
    inversores: Vec<Recipient<Invertir>>,
    devoluciones: HashSet<usize>
}

struct Inversor {
    id: usize,
}

impl Actor for Banquero {
    type Context = Context<Self>;
}

impl Actor for Inversor {
    type Context = Context<Self>;
}

// El banquero recibe el mensaje Semana
impl Handler<Semana> for Banquero {
    type Result = ();

    fn handle(&mut self, msg: Semana, _ctx: &mut Self::Context) -> Self::Result {
        let plata_semana = msg.0;
        let amount = plata_semana / self.inversores.len() as f64;
        self.devoluciones.clear();
        self.plata = 0.0;

        println!("[BANQUERO] empieza la semana con {}", plata_semana);
        // Envio un mensaje a cada inversor, paso el monto y la direccion de mi correo/mailbox
        // asi saben a donde enviar el resultado de su inversión
        for inversor in self.inversores.iter() {
            inversor.try_send(Invertir{amount, sender: _ctx.address().recipient()}).unwrap()
        }
    }
}

// El banquero recibe el mensaje ResultadoInversion
impl Handler<ResultadoInversion> for Banquero {
    type Result = ();
    fn handle(&mut self, msg: ResultadoInversion, _ctx: &mut Self::Context) -> Self::Result {
        println!("[BANQUERO] recibí resultado de la inversion {}", msg.0);
        if !self.devoluciones.contains(&msg.0) {
            self.plata += msg.1;
            self.devoluciones.insert(msg.0);

            if self.devoluciones.len() == self.inversores.len() {
                // Me mensajeo a mi mismo el comienzo de la nueva semana
                println!("[Banquero] final de semana {}", self.plata);
                _ctx.address().try_send(Semana(self.plata)).unwrap();
            }
        }
    }
}

// El inversor recibe el msg Invertir
impl Handler<Invertir> for Inversor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Invertir, _ctx: &mut Self::Context) -> Self::Result {
        println!("[INV {}] recibo inversion por {}", self.id, msg.amount);
        Box::pin(sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)))
            .into_actor(self)
            .map(move |_result, me, _ctx| {
                let resultado = msg.amount * thread_rng().gen_range(0.5, 1.5);
                println!("[INV {}] devuelvo {}", me.id, resultado);
                msg.sender.try_send(ResultadoInversion(me.id, resultado)).unwrap();
            }))
    }
}

fn main() {
    let system = System::new();
    system.block_on(async {
        let mut inversores = vec!();

        for id in 0..INVERSORES {
            inversores.push(Inversor { id }.start().recipient())
        }


        Banquero { plata: 0.0, inversores, devoluciones: HashSet::with_capacity(INVERSORES) }.start()
            .do_send(Semana(1000.0));
    });

    system.run().unwrap();

}