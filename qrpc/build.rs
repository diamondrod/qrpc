use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    //  Use `QRPC_PROTO_DIR` as a directory containing file descriptor set and proto files.
    let qrpc_proto_dir = PathBuf::from(env::var("QRPC_PROTO_DIR").expect("QRPC_PROTO_DIR is not set"));

    let paths = fs::read_dir(qrpc_proto_dir.clone())?;
    let mut proto_files = Vec::new();
    for entry_ in paths{
      let entry = entry_?;
      if entry.file_type()?.is_file(){
        let os_file_name = entry.file_name();
        proto_files.push(os_file_name.to_str().expect("failed to convert into str").to_string());
      }
    };

    tonic_build::configure()
      .format(true)
      // qrpc_fd_set is created in qRPC/.
      .file_descriptor_set_path("./qrpc_fd_set")
      .out_dir("src/client/proto")
      .include_file("mod.rs")
      .compile(&proto_files, &[qrpc_proto_dir.clone()])?;

    qrpc_build::generate_code(&proto_files, &[qrpc_proto_dir])?;
      
    Ok(())
}
