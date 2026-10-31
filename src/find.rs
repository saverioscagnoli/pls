use crate::args::FindArgs;

pub fn find(args: FindArgs) -> Result<(), String> {
    println!("{:?}", args);

    Ok(())
}
