use faketime::unix_time_as_millis;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn unix_time_as_millis_should_work() {
    assert!(unix_time_as_millis() > 0);
}
