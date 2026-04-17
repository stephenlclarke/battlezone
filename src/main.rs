fn main() {
    std::process::exit(exit_code_for_result(battlezone::app::run()));
}

fn exit_code_for_result(result: anyhow::Result<()>) -> i32 {
    if let Err(err) = result {
        eprintln!("battlezone: {err:#}");
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exit_code_is_zero_for_success() {
        assert_eq!(super::exit_code_for_result(Ok(())), 0);
    }

    #[test]
    fn exit_code_is_one_for_failure() {
        let result = Err(anyhow::anyhow!("boom"));
        assert_eq!(super::exit_code_for_result(result), 1);
    }
}
