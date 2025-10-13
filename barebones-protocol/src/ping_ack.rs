use hydro_lang::prelude::*;
use serde::{Deserialize, Serialize};

// type tags/labels
pub struct Client;
pub struct Coord;
pub struct Participant;

// message tuples (travels on streams)
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ping {
    id: u64
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ack {
    id: u64
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Done {
    id: u64
}

pub fn ping_ack<'a>(
    client: &Process<'a, Client>,     // 1 client proc
    coord: &Process<'a, Coord>,       // 1 coord proc
    participants: &Cluster<'a, Participant>, // cluster of part. procs
) {
    // client emits ping (base fact)
    let pings = client.source_iter(q!(vec![
        Ping {id: 100}
    ]));

    // send ping from client -> coord
    let pings_at_coord = pings.send_bincode(coord);

    // coord broadcasts ping to every participant
    // broadcasting docs: https://hydro.run/docs/hydro/locations/clusters/#broadcasting-and-membership-lists
    let pings_at_parts = pings_at_coord.broadcast_bincode(
        participants,
        nondet!(#[doc = "membership may change; ok"])
    );

    // reply with ack(id) from part. -> coord
    let acks_at_coord = pings_at_parts
        .map(q!(|p| Ack { id: p.id }))
        .send_bincode(coord);
    

    // debug prints
    acks_at_coord.clone().values()
    .inspect(q!(|a: &Ack| println!("[COORD] Ack({})", a.id)));

    // convert Ack -> Done once per value in the tick
    let done_msgs = acks_at_coord
        .values()
        .map(q!(|a: Ack| Done { id: a.id }))
        .unique(); // emit only the first Done(100)

    // debug
    done_msgs.clone().inspect(q!(|d: &Done| println!("[COORD] Done({})", d.id)));

    // coord -> client Done message (chain directly)
    done_msgs
        .send_bincode(client)
        .inspect(q!(|d: &Done| println!("[CLIENT] Done({})", d.id)));
        // .for_each(q!(|d: &Done| println!("[CLIENT] Done({})", d.id)));

}
