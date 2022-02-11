use std::{env, io, path::PathBuf};

fn main() -> io::Result<()> {
    //  Use `QRPC_PROTO_DIR` as a directory containing file descriptor set and proto files.
    let qrpc_proto_dir = PathBuf::from(env::var("QRPC_PROTO_DIR").expect("QRPC_PROTO_DIR is not set"));
    let proto_dir = qrpc_proto_dir.join("proto");

    let proto_files = &["q.proto", "example.proto", "example_service.proto"];

    tonic_build::configure()
      .format(true)
      // qrpc_fd_set is created in qRPC/.
      .file_descriptor_set_path("./qrpc_fd_set")
      .out_dir("src/client/proto")
      .compile(proto_files, &[proto_dir])?;

    qrpc_build::generate_code(proto_files, &[proto_dir])?;
      
    Ok(())
}
