use crate::args::ListArgs;

pub fn list(args: ListArgs) -> Result<(), String> {
    println!("{:?}", args);
    Ok(())
}
