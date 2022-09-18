fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["./src/proxy.proto"], &["./src/"])?;
    Ok(())
}
