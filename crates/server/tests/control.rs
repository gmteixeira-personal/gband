use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use gband_lua::Side;
use gband_protocol::{Answer, ClientMessage, Entry, Operation, Process, ServerMessage, Target};
use gband_server::{ANSWER_TIMEOUT, Scripting, ServerConfig};
use gband_test_support::*;

async fn start(name: &str, loads: &Arc<AtomicU64>) -> TestServer {
    let runtime_dir = runtime_dir(name);
    let (scripting, _) = Scripting::new(gband_lua::defaults(Side::Server), None);
    let counted = Arc::clone(loads);
    let scripting = scripting.with_loader(Arc::new(move || {
        counted.fetch_add(1, Ordering::Relaxed);
        Ok(gband_lua::defaults(Side::Server))
    }));
    let config = ServerConfig {
        scripting: Some(scripting),
        channel: None,
        ..config(&runtime_dir, &["/bin/sh"])
    };
    TestServer::start_with(runtime_dir, config).await
}

fn calls(client: &TestClient) -> Vec<(u64, Operation)> {
    client
        .bridge
        .iter()
        .filter_map(|message| match message {
            ServerMessage::Control { call, operation } => Some((*call, operation.clone())),
            _ => None,
        })
        .collect()
}

async fn answer(client: &mut TestClient, answer: Answer) {
    client.wait_until(|client| !calls(client).is_empty()).await;
    let (call, operation) = calls(client).remove(0);
    assert_eq!(operation, Operation::Errors);
    client
        .send(&ClientMessage::ControlAnswer { call, answer })
        .await;
}

fn errors(load: u64) -> Answer {
    Answer::Errors {
        load,
        errors: Vec::new(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn control_request_answered_by_every_client() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = Arc::new(start("control-every", &loads).await);
    let mut first = server.attach_to("work", Path::new("/"), 80, 24).await;
    let mut second = server.attach_to("work", Path::new("/"), 80, 24).await;
    let mut other = server.attach_to("other", Path::new("/"), 80, 24).await;
    let asking = Arc::clone(&server);
    let request = tokio::spawn(async move {
        asking
            .control("work", Target::EveryClient, Operation::Errors)
            .await
    });
    answer(&mut second, errors(5)).await;
    answer(&mut first, errors(3)).await;
    let ServerMessage::ControlResults(entries) = request.await.unwrap() else {
        panic!("no control results");
    };
    let numbers: Vec<Process> = entries.iter().map(|entry| entry.process).collect();
    let mut sorted = numbers.clone();
    sorted.sort();
    assert_eq!(numbers, sorted);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].answer, Some(errors(3)));
    assert_eq!(entries[1].answer, Some(errors(5)));
    other.pump(Duration::from_millis(200)).await;
    assert!(calls(&other).is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn server_and_every_client_with_the_all_target() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = Arc::new(start("control-all", &loads).await);
    let mut client = server.attach_to("work", Path::new("/"), 80, 24).await;
    let asking = Arc::clone(&server);
    let request =
        tokio::spawn(async move { asking.control("work", Target::All, Operation::Errors).await });
    answer(&mut client, errors(1)).await;
    let ServerMessage::ControlResults(entries) = request.await.unwrap() else {
        panic!("no control results");
    };
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0],
        Entry {
            process: Process::Server,
            answer: Some(errors(1)),
        }
    );
    assert!(matches!(entries[1].process, Process::Client(_)));
}

#[tokio::test(flavor = "multi_thread")]
async fn control_request_for_an_absent_session() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = start("control-absent", &loads).await;
    assert_eq!(
        server
            .control("absent", Target::All, Operation::Reload)
            .await,
        ServerMessage::NoSuchSession
    );
    assert_eq!(loads.load(Ordering::Relaxed), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn targets_with_no_client_give_no_entry() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = start("control-none", &loads).await;
    for target in [Target::EveryClient, Target::Chosen, Target::Client(99)] {
        assert_eq!(
            server.control("default", target, Operation::Errors).await,
            ServerMessage::ControlResults(Vec::new()),
            "{target:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn reload_message_answered() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = start("control-reload", &loads).await;
    let mut client = server.attach(80, 24).await;
    client.send(&ClientMessage::Reload).await;
    client
        .wait_until(|client| client.bridge.contains(&ServerMessage::Reloaded))
        .await;
    assert_eq!(loads.load(Ordering::Relaxed), 1);
    let ServerMessage::ControlResults(entries) = server
        .control("default", Target::Server, Operation::Errors)
        .await
    else {
        panic!("no control results");
    };
    assert_eq!(entries[0].answer, Some(errors(2)));
}

#[tokio::test(flavor = "multi_thread")]
async fn stopped_client() {
    let loads = Arc::new(AtomicU64::new(0));
    let server = Arc::new(start("control-stopped", &loads).await);
    let mut stopped = server.attach(80, 24).await;
    let started = Instant::now();
    let asking = Arc::clone(&server);
    let request = tokio::spawn(async move {
        asking
            .control("default", Target::All, Operation::Errors)
            .await
    });
    stopped.wait_until(|client| !calls(client).is_empty()).await;
    let ServerMessage::ControlResults(entries) = request.await.unwrap() else {
        panic!("no control results");
    };
    let elapsed = started.elapsed();
    assert!(elapsed >= ANSWER_TIMEOUT, "{elapsed:?}");
    assert!(
        elapsed < ANSWER_TIMEOUT + Duration::from_secs(1),
        "{elapsed:?}"
    );
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].answer, None);
    let (call, _) = calls(&stopped).remove(0);
    stopped
        .send(&ClientMessage::ControlAnswer {
            call,
            answer: errors(1),
        })
        .await;
    stopped.pump(Duration::from_millis(200)).await;
    assert!(matches!(
        server
            .control("default", Target::Server, Operation::Errors)
            .await,
        ServerMessage::ControlResults(_)
    ));
}
