use std::{env, io::Result, path::PathBuf};

fn main() -> Result<()> {
    let binding = PathBuf::from(env::current_dir().unwrap().parent().unwrap());
    let path = binding.parent().unwrap().to_str().unwrap();
    let abs_path = format!("{path}/docs/quests.proto");

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed={abs_path}");
    let mut prost_build_config = prost_build::Config::new();
    prost_build_config.service_generator(Box::new(dcl_rpc::codegen::RPCServiceGenerator::new()));
    prost_build_config
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .compile_protos(&[abs_path], &[path])?;

    Ok(())
}
