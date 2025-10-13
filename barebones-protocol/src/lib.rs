stageleft::stageleft_no_entry_crate!();

// expose modules
pub mod add_two;
pub mod ping_ack;

#[cfg(test)]
mod test_init {
    #[ctor::ctor]
    fn init() {
        hydro_lang::deploy::init_test();
    }
}
