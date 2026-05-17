pub enum ResultControlFlow<T, E> {
    Ok(T),
    Err(E),
    Break,
    Continue,
}

impl<T, E> ResultControlFlow<T, E> {
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break)
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

#[macro_export]
macro_rules! continue_if_fail {
    ($result:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => {
                tracing::error!("{err}");
                continue;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::ResultControlFlow;

    #[test]
    fn predicates_are_mutually_exclusive() {
        let ok: ResultControlFlow<i32, &str> = ResultControlFlow::Ok(1);
        assert!(ok.is_ok() && !ok.is_err() && !ok.is_break() && !ok.is_continue());

        let err: ResultControlFlow<i32, &str> = ResultControlFlow::Err("boom");
        assert!(err.is_err() && !err.is_ok() && !err.is_break() && !err.is_continue());

        let brk: ResultControlFlow<i32, &str> = ResultControlFlow::Break;
        assert!(brk.is_break() && !brk.is_ok() && !brk.is_err() && !brk.is_continue());

        let cont: ResultControlFlow<i32, &str> = ResultControlFlow::Continue;
        assert!(cont.is_continue() && !cont.is_ok() && !cont.is_err() && !cont.is_break());
    }

    #[test]
    fn continue_if_fail_skips_err_and_keeps_ok() {
        let inputs: Vec<Result<i32, &str>> = vec![Ok(1), Err("skip"), Ok(3)];
        let mut collected = Vec::new();
        for input in inputs {
            let value = continue_if_fail!(input);
            collected.push(value);
        }
        assert_eq!(collected, vec![1, 3]);
    }
}
