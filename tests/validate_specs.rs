#[test]
fn validate_all_specs() {
    pipecat_rs::spec::validate_all_specs_in("tests/specs/**/*.yaml").expect("spec validation failed");
}


