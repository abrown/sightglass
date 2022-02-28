mod util;

use assert_cmd::prelude::*;
use util::sightglass_cli;

#[test]
fn help() {
    sightglass_cli().arg("help").assert().success();
}
