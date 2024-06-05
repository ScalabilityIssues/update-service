fn main() -> Result<(), Box<dyn std::error::Error>> {
    // compile protos
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_transport(false)
        .build_server(false)
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
