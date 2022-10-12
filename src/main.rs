#[macro_use]
extern crate rocket;

use zbc_oracle_db::build_rocket;

#[launch]
fn rocket() -> _ {
    build_rocket()
}
