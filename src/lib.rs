use std::sync::Once;
static INIT_LIB: Once = Once::new();
// export rules
pub mod rules;
// export selector
pub mod selector;
// utils for crate
pub mod utils;
// export init, must execute `init()` first
pub fn init() {
	INIT_LIB.call_once(|| {
		rules::init();
	});
}
