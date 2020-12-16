use hotpatch::patch;

#[patch]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("I'm from somewhere else!");
    Ok(())
}
