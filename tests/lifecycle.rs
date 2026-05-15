//! 라이프사이클 훅 실행 순서 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::{Command, WrCliError};

#[test]
fn lifecycle_hooks_full_order() {
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(vec![]));
    let (l1, l2, l3, l4, l5, l6_root, l6_sub) = (
        log.clone(),
        log.clone(),
        log.clone(),
        log.clone(),
        log.clone(),
        log.clone(),
        log.clone(),
    );
    Command::new("app")
        .on_persistent_pre_run(move |_| l1.lock().unwrap().push("root:persistent_pre"))
        .on_persistent_post_run(move |_| l6_root.lock().unwrap().push("root:persistent_post"))
        .subcommand(
            Command::new("sub")
                .on_persistent_pre_run(move |_| l2.lock().unwrap().push("sub:persistent_pre"))
                .on_pre_run(move |_| l3.lock().unwrap().push("sub:pre"))
                .on_run(move |_| l4.lock().unwrap().push("sub:run"))
                .on_post_run(move |_| l5.lock().unwrap().push("sub:post"))
                .on_persistent_post_run(move |_| {
                    l6_sub.lock().unwrap().push("sub:persistent_post")
                }),
        )
        .execute_with(args("sub"))
        .unwrap();

    assert_eq!(
        *log.lock().unwrap(),
        vec![
            "root:persistent_pre",
            "sub:persistent_pre",
            "sub:pre",
            "sub:run",
            "sub:post",
            "sub:persistent_post",
            "root:persistent_post",
        ]
    );
}

#[test]
fn run_e_error_aborts_post_hooks() {
    let post_called = Arc::new(Mutex::new(false));
    let post2 = post_called.clone();
    let err = Command::new("app")
        .on_run_e(|_| Err(WrCliError::ArgValidationFailed("fail".to_owned())))
        .on_post_run(move |_| *post2.lock().unwrap() = true)
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
    assert!(!*post_called.lock().unwrap());
}
