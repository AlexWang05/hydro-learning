// practice: mirror p(a) :- p(b), add(b, 2, a)
// start from base facts p(b). derive infinite table add(b,2,a) which just means a = b + 2

use hydro_lang::prelude::*;

// tag type for process (empty struct / zero-sized type)
pub struct Solo;

/*
------ RUN RESULTS ------
[hydro_template::add_two::Solo (process 0)] [base p(b)] 1
[hydro_template::add_two::Solo (process 0)] [derived p(a) 3]
[hydro_template::add_two::Solo (process 0)] [base p(b)] 5
[hydro_template::add_two::Solo (process 0)] [derived p(a) 7]
[hydro_template::add_two::Solo (process 0)] [base p(b)] 10
[hydro_template::add_two::Solo (process 0)] [derived p(a) 12]
*/
pub fn add_two<'a>(p: &Process<'a, Solo>) {
    // Process<'a, Solo> means a hydro process with tag type Solo

    // base facts: p(b) = {1, 5, 10}
    // q! = hydro macro marking code as runtime code
    let base = p.source_iter(q!(vec![1, 5, 10]));

    // derivation: a = b + 2 aka add(b, 2, a)
    // |b| b+2 is a Rust closure / anon func w/ arg b
    let derived = base
    .inspect(q!(|b| println!("[base p(b)] {}", b))) // return a stream so we can keep chaining
    .map(q!(|b| b + 2));

    // printing derived vals
    derived.for_each(q!(|a| println!("[derived p(a) {}]", a)));
}