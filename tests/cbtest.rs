// tests/cbtest.rs — does select! fire a recv arm on a disconnected channel?
use crossbeam_channel::{bounded, select};

#[test]
fn select_fires_on_disconnected_receiver() {
    let (s, r) = bounded::<()>(1);
    let r2 = r.clone();
    drop(s); // disconnect: recv is ready-with-error per the docs

    let mut fired = false;
    let (tx, rx) = bounded::<bool>(1);
    std::thread::spawn(move || {
        select! {
            recv(r2) -> _ => { tx.send(true).unwrap(); }
        }
    });
    let Ok(f) = rx.recv_timeout(std::time::Duration::from_secs(2)) else {
        panic!("select arm did NOT fire on a disconnected receiver");
    };
    fired = f;
    assert!(fired);
    drop(r);
}
