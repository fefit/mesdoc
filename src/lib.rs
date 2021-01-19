use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
lazy_static! {
  static ref IS_RULES_INIT: AtomicBool = AtomicBool::new(false);
}
// export rules
pub mod rules;
// export selector
pub mod selector;
// utils for crate
pub mod utils;
// export init, must execute `init()` first
pub fn init(){
	if !IS_RULES_INIT.load(Ordering::SeqCst) {
		rules::init();
		// set init true
		IS_RULES_INIT.store(true, Ordering::SeqCst);
	}
}