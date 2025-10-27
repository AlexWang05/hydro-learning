use hydro_lang::prelude::*;
use serde::{Deserialize, Serialize};

// type tags/labels
pub struct Client;
pub struct Coord;
pub struct Participant;

// messages
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Begin {
    tx: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Prepare {
    tx: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Vote {
    tx: u64,
    yes: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Decision {
    tx: u64,
    commit: bool, // all-yes => true; any-no => false
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ack {
    tx: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Done {
    tx: u64,
}

/// 2PC (no log)
///
/// Phase 1:
///   - coord tells participants to prepare
///   - participants respond with yes/no votes  (here: yes)
///
/// Phase 2:
///   - coord disseminates result (all yes => commit; any no => abort)
///   - participants respond with Ack; coord reports Done to client
pub fn two_pc<'a>(
    client: &Process<'a, Client>,            // 1 client
    coord: &Process<'a, Coord>,              // 1 coordinator
    participants: &Cluster<'a, Participant>, // N participants
) {
    // client seeds a single transaction
    let begins = client
        .source_iter(q!(vec![Begin { tx: 1 }]))  // create stream from static data
        .inspect(q!(|b: &Begin| println!("[CLIENT] Begin(tx={})", b.tx)));  // peek at values (doesn't consume)

    // client -> coord
    let begins_at_coord = begins
        .send_bincode(coord)  // serialize & send to coord; returns stream at coord
        .inspect(q!(|b: &Begin| println!("[COORD] Received Begin(tx={})", b.tx)));

    // ===================== Phase 1: PREPARE =====================
    // coord -> parts: Prepare(tx)
    let prepares_at_coord = begins_at_coord
        .map(q!(|b: Begin| {  // transform Begin -> Prepare
            println!("[COORD] Sending Prepare(tx={})", b.tx);
            Prepare { tx: b.tx }
        }));

    let prepares_at_parts = prepares_at_coord
        .broadcast_bincode(  // send to ALL participants in cluster
            participants,
            nondet!(#[doc = "membership may change; ok"]),
        )
        .inspect(q!(|p: &Prepare| println!("[PARTICIPANT] Received Prepare(tx={})", p.tx)));

    // parts -> coord: Vote(tx, yes/no). Keep it simple: all YES.
    let votes_at_coord = prepares_at_parts
        .map(q!(|p: Prepare| {  // each participant: Prepare -> Vote
            println!("[PARTICIPANT] Sending Vote(tx={}, yes=true)", p.tx);
            Vote { tx: p.tx, yes: true }
        }))
        .send_bincode(coord);  // each participant sends vote to coord

    // coord receives votes from all participants, aggregates, and decides
    let decisions_one = votes_at_coord
        .values()  // iterate over multiset (may have multiple votes)
        .inspect(q!(|v: &Vote| println!("[COORD] Received Vote(tx={}, yes={})", v.tx, v.yes)))
        .map(q!(|v: Vote| Decision { tx: v.tx, commit: true }))  // Vote -> Decision
        .unique();  // deduplicate: emit only ONE decision per tx (not one per vote)

    // debug: print decisions at coord before broadcasting
    let decisions_with_log = decisions_one
        .inspect(q!(|d: &Decision| println!("[COORD] Broadcasting Decision(tx={}, commit={})", d.tx, d.commit)));

    // ===================== Phase 2: DECISION + ACK =====================
    // coord -> parts: Decision(tx, commit/abort)
    let decisions_at_parts = decisions_with_log.broadcast_bincode(
        participants,
        nondet!(#[doc = "membership may change; ok"]),
    );

    // parts -> coord: Ack(tx) with logging
    let acks_at_coord = decisions_at_parts
        .inspect(q!(|d: &Decision| println!("[PARTICIPANT] Received Decision(tx={}, commit={})", d.tx, d.commit)))
        .map(q!(|d: Decision| {  // each participant: Decision -> Ack
            println!("[PARTICIPANT] Sending Ack(tx={})", d.tx);
            Ack { tx: d.tx }
        }))
        .send_bincode(coord);

    // coord receives acks, transforms to Done for client
    let done_msgs = acks_at_coord
        .values()  // iterate over acks from all participants
        .inspect(q!(|a: &Ack| println!("[COORD] Received Ack(tx={})", a.tx)))
        .map(q!(|a: Ack| Done { tx: a.tx }))  // Ack -> Done
        .unique();  // deduplicate: emit ONE Done per tx (not one per ack)

    // coord -> client: Done (and print on client)
    done_msgs
        .send_bincode(client)
        .assume_ordering(nondet!(#[doc = "demo"]))  // ok with out-of-order delivery
        .assume_retries(nondet!(#[doc = "demo"]))   // ok with duplicates
        .for_each(q!(|d: Done| println!("[CLIENT] Done(tx={})", d.tx)));  // terminal: consume stream
}
