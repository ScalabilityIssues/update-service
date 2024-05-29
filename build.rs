use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // compile protos
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("proto_descriptor.bin"))
        .build_transport(false)
        .compile(
            &[
                "proto/flightmngr/flights.proto",
                "proto/ticketsrvc/tickets.proto",
                "proto/validationsvc/validation.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
