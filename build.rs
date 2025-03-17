#[cfg(feature = "js")]
extern crate napi_build;

fn main() {
    #[cfg(feature = "js")]
    napi_build::setup();
}
