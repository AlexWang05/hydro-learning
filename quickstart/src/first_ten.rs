use hydro_lang::prelude::*;

pub fn first_ten(process: &Process) {
    process
        .source_iter(q!(0..10)) // q! macro marks code that will be executed at runtime
        .for_each(q!(|n| println!("{}", n)));
}
