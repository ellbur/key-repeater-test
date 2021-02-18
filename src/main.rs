
// vim: shiftwidth=2

use std::collections::HashMap;
use tokio::sync::oneshot;
use tokio::time::{sleep, interval_at, Duration, Instant};
use std::cell::RefCell;
use tokio::task;
use std::rc::Rc;

#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Hash, Copy, Clone)]
enum KeyCode {
  A, B, C, D, E
}

struct PressedKey {
  cancel_tx: tokio::sync::oneshot::Sender<()>
}

struct Repeater {
  pressed_keys: RefCell<HashMap<KeyCode, PressedKey>>
}

impl Repeater {
  fn new() -> Repeater {
    Repeater {
      pressed_keys: RefCell::new(HashMap::new())
    }
  }
  
  async fn press_key(self: &Repeater, key: KeyCode) {
    println!("press_key({:?})", key);
    let mut ticking = interval_at(Instant::now() + Duration::from_millis(150), Duration::from_millis(30));
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    
    (*self.pressed_keys.borrow_mut()).insert(key, PressedKey { cancel_tx });
    
    tokio::pin!(cancel_rx);
    
    loop {
      tokio::select! {
        _ = &mut cancel_rx => {
          println!("Key released {:?}", key);
          break;
        }
        _ = ticking.tick() => {
          println!("Key repeated {:?}", key);
        }
      }
    }
  }
  
  async fn release_key(self: &Repeater, key: KeyCode) {
    println!("release_key({:?})", key);
    match (*self.pressed_keys.borrow_mut()).remove(&key) {
      None => (),
      Some(pk) => {
        match pk.cancel_tx.send(()) {
          Ok(_) => (),
          Err(_) => ()
        }
      }
    }
  }
}

#[tokio::main]
pub async fn main() {
  use KeyCode::*;
  let local = task::LocalSet::new();
  
  let repeater = Rc::new(Repeater::new());
  
  local.run_until(async move {
    let press_key = |k: KeyCode| {
      task::spawn_local({
        let repeater = repeater.clone();
        async move {
          repeater.press_key(k).await
        }
      });
    };
    press_key(A);
    sleep(Duration::from_millis(40)).await;
    press_key(B);
    sleep(Duration::from_millis(200)).await;
    repeater.release_key(A).await;
    sleep(Duration::from_millis(400)).await;
    repeater.release_key(B).await;
  }).await;
  local.await;
}

