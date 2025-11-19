#[cfg(stageleft_runtime)]
hydro_lang::setup!();

use hydro_lang::{live_collections::stream::NoOrder, prelude::*};

/// Lock Server implementation
/// 
/// # API
/// **Input:** Stream<(String, bool, usize)>
/// - String: person name
/// - bool: true = acquire, false = release
/// - usize: lock ID
/// 
/// **Output:** Stream<(String, bool, usize)>
/// - String: person name
/// - bool: true = success, false = failure
/// - usize: lock ID
/// 
/// # Invariants
/// 1. Only one person can hold a lock at a time
/// 2. Can only acquire when the lock is not held
/// 3. Can only release when you hold the lock
pub struct LockServer;
pub fn lock_server<'a>(
    input: Stream<(String, bool, usize), Process<'a, LockServer>>,
) -> Stream<(String, bool, usize), Process<'a, LockServer>, Unbounded, NoOrder> {
    let server = input.location();
    let server_tick = server.tick();

    // input stream is Stream<(String, bool usize)>), like ("Alice", true, 0) = Alice to acquire lock 0

    let ticked_inputs = input
        // map form ("Alice", true, 0) => (0, ("Alice", true))
        .map(q!(|(name, acquire, lock_id)| (lock_id, (name, acquire))))
        .batch(&server_tick, nondet!(#[doc = "go into tick so fold finishes"]));

    let lock_state = ticked_inputs.clone()
        .persist() // need this fold to persist
        .into_keyed() //now each lock_id has its own stream of reqs
        .fold(q!(|| None), q!(|lock_holder, (name, acquire)| {
            if acquire && lock_holder.is_none() { // can acquire
                *lock_holder = Some(name);
            } else if !acquire { // release
                if let Some(curr_holder) = lock_holder {
                    if *curr_holder == name {
                        *lock_holder = None;
                    }
                }
            }
        }));

    // ticked_inputs => new lock state after fold processes current tick's requests
    // lock_state => clone of ticked_inputs, previous tick's lock state

    // pairs each element from ticked_inputs with prev tick's lock state
    // lock_holder is lock state from prev tick
    lock_state.get_many_if_present(ticked_inputs.into_keyed())
    .all_ticks()
        .entries()
        .map(q!(|(lock_id, (lock_holder, (name, acquire)))| {
            let success = if acquire {
                lock_holder.as_ref() == Some(&name) // if you held it in prev state & try to release => success
            } else {
                true
                // lock_holder.is_none() // if lock was free in prev state & we try to acq => success
            };
            (name, success, lock_id)
        }))
        
        
        // .map(q!(|((lock_id, (name, acquire)), lock_holder)| {
        //     let success = if acquire {
        //         lock_holder.is_none() // if lock was free in prev state & we try to acq => success
        //     } else {
        //         lock_holder.as_ref() == Some(&name) // if you held it in prev state & try to release => success
        //     };
        //     (name, success, lock_id)
        // }))
}
