// BSD 3-Clause Clear License

// Copyright © 2023 ZAMA.
// All rights reserved.

#[macro_use]
extern crate rocket;

use fhevm_decryptions_db::build_and_configure_rocket;

#[launch]
fn rocket() -> _ {
    build_and_configure_rocket()
}
