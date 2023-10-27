extern crate rand;

use std::collections::HashMap;
use std::time::Duration;
use rand::{thread_rng, Rng};
use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, ResponseActFuture, WrapFuture};
use actix_rt::System;
use tokio::time::sleep;

const N:usize = 5;

/**
Cinco filósofos se sientan alrededor de una mesa y pasan su vida cenando y pensando.
Cada filósofo tiene un plato de fideos y un palito chino a la izquierda de su plato.
Para comer los fideos son necesarios dos palitos y cada filósofo sólo puede tomar los que
están a su izquierda y derecha. Si cualquier filósofo toma un palito y el otro está ocupado,
se quedará esperando, con el tenedor en la mano, hasta que pueda tomar el otro tenedor,
para luego empezar a comer.

Resolvemos con algoritmos Chandy-Mistra
 */

type Vecinos = HashMap<IdPalito, Addr<Filosofo>>;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct IdPalito(usize);

#[derive(PartialEq)]
enum EstadoPalito {
    NoTiene,
    Sucio,
    Limpio,
    Solicitado
}

#[derive(Message)]
#[rtype(result = "()")]
struct SetVecinos {
    vecinos: HashMap<IdPalito, Addr<Filosofo>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Hambriento;

#[derive(Message)]
#[rtype(result = "()")]
struct SolicitarPalito {
    id_palito: IdPalito
}

#[derive(Message)]
#[rtype(result = "()")]
struct RespuestaPalito {
    id_palito: IdPalito
}

#[derive(Message)]
#[rtype(result = "()")]
struct TerminoDeComer;

struct Filosofo {
    id: usize,
    palitos: HashMap<IdPalito, EstadoPalito>,
    vecinos: Vecinos
}

impl Filosofo {
    fn sleep<WakeupMessage>(&self, msg: WakeupMessage) -> ResponseActFuture<Self, ()>
        where
            Self: Handler<WakeupMessage>,
            WakeupMessage: Message + Send + 'static,
            WakeupMessage::Result: Send,
    {
        println!("[{}] pensando", self.id);
        Box::pin(sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)))
            .into_actor(self)
            .map(move |_result, me, _ctx| {
                _ctx.address().try_send(msg).unwrap()
            }))
    }

    fn eat_if_ready(&self) -> ResponseActFuture<Self, ()> {
        if self.palitos.iter().all(|(id_palito_, estado)| *estado != EstadoPalito::NoTiene) {
            println!("[{}] comiendo", self.id);
            self.sleep(TerminoDeComer)
        } else {
            println!("[{}] aun no puedo comer", self.id);
            Box::pin(std::future::ready(()).into_actor(self))
        }
    }
}

impl Actor for Filosofo {
    type Context = Context<Self>;
}

impl Handler<SetVecinos> for Filosofo {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: SetVecinos, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}] recibi a mis vecinos", self.id);
        self.vecinos = msg.vecinos;
        self.sleep(Hambriento)
    }
}

impl Handler<Hambriento> for Filosofo {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: Hambriento, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}] por comer", self.id);
        for (id_palito, estado_palito) in self.palitos.iter() {
            if *estado_palito == EstadoPalito::NoTiene {
                println!("[{}] pido palito {}", self.id, id_palito.0);
                self.vecinos.get(id_palito).unwrap().try_send(SolicitarPalito {
                    id_palito: *id_palito
                }).unwrap()
            }
        }

        self.eat_if_ready()
    }
}

impl Handler<SolicitarPalito> for Filosofo {
    type Result = ();

    fn handle(&mut self, msg: SolicitarPalito, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}] me piden palito {}", self.id, msg.id_palito.0);
        let id_palito = msg.id_palito;
        let estado_palito = &self.palitos.get(&id_palito);
        match estado_palito {
            Some(EstadoPalito::Sucio) => {
                println!("[{}] se lo doy ahora", self.id);
                self.vecinos.get(&id_palito).unwrap().try_send(RespuestaPalito {
                    id_palito
                }).unwrap();
                self.palitos.insert(id_palito, EstadoPalito::NoTiene);
            }
            Some(EstadoPalito::Limpio) => {
                println!("[{}] se lo doy cuando termine", self.id);
                self.palitos.insert(id_palito, EstadoPalito::Solicitado);
            }
            _ => {
                println!("[{}] no deberia pasar", self.id);
            }
        }
    }
}

impl Handler<RespuestaPalito> for Filosofo {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: RespuestaPalito, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}] recibi palito {}", self.id, msg.id_palito.0);
        self.palitos.insert(msg.id_palito, EstadoPalito::Limpio);
        self.eat_if_ready()
    }
}


impl Handler<TerminoDeComer> for Filosofo {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TerminoDeComer, _ctx: &mut Self::Context) -> Self::Result {
        for (id_palito, mut estado) in self.palitos.iter_mut() {
            if *estado == EstadoPalito::Solicitado {
                println!("[{}] entrego palito {}", self.id, id_palito.0);
                self.vecinos.get(id_palito).unwrap().try_send(RespuestaPalito {
                    id_palito: *id_palito
                }).unwrap();
                *estado = EstadoPalito::NoTiene
            } else {
                println!("[{}] marco como sucio palito {}", self.id, id_palito.0);
                *estado = EstadoPalito::Sucio
            }
        }

        self.sleep(Hambriento)
    }
}




fn main() {
    let system = System::new();
    system.block_on(async {
        let mut philosophers = vec!();

        for id in 0..N {
            // Deadlock avoidance forcing the initial state
            philosophers.push(Filosofo {
                id,
                palitos: HashMap::from([
                    (IdPalito(id), if id == 0 { EstadoPalito::Sucio } else { EstadoPalito::NoTiene }),
                    (IdPalito((id + 1) % N), if id == N-1 { EstadoPalito::NoTiene } else { EstadoPalito::Sucio })
                ]),
                vecinos: HashMap::with_capacity(2)
            }.start())
        }

        for id in 0..N {
            let prev = if id == 0 { N - 1 } else { id - 1 };
            let next = (id + 1) % N;
            philosophers[id].try_send(
                SetVecinos {
                    vecinos: HashMap::from([
                        (IdPalito(id), philosophers[prev].clone()),
                        (IdPalito(next), philosophers[next].clone())
                    ])
                }
            ).unwrap();
        }
    });

    system.run().unwrap();

}
