use std::error::Error;
use vergen_gitcl::{Emitter, Gitcl};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");

    // get commit hash
    let git = Gitcl::builder().sha(true).build();

    Emitter::default().add_instructions(&git)?.emit()?;
    Ok(())
}
