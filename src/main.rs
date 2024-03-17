use anyhow::Error;
fn main() -> Result<(), Error> {
    let x = op::entry();
    x
}

#[cfg(test)]
mod basic_test;
mod op;