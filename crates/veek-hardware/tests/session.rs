use std::{
    collections::VecDeque,
    io,
    sync::{Arc, Mutex},
};
use veek_hardware::{
    protocol::ControlEvent,
    transport::{HardwareError, Session, Transport},
    Model,
};

type Writes = Arc<Mutex<Vec<Vec<u8>>>>;
struct Mock {
    reads: VecDeque<Result<Vec<u8>, io::Error>>,
    writes: Writes,
    short_write: bool,
}
impl Transport for Mock {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, HardwareError> {
        let next = self.reads.pop_front().unwrap_or_else(|| Ok(vec![]))?;
        data[..next.len()].copy_from_slice(&next);
        Ok(next.len())
    }
    fn write(&mut self, data: &[u8]) -> Result<usize, HardwareError> {
        self.writes.lock().unwrap().push(data.to_vec());
        Ok(if self.short_write { 2 } else { data.len() })
    }
}
fn mock(reads: Vec<Result<Vec<u8>, io::Error>>, short_write: bool) -> (Box<dyn Transport>, Writes) {
    let writes = Arc::new(Mutex::new(vec![]));
    (
        Box::new(Mock {
            reads: reads.into(),
            writes: writes.clone(),
            short_write,
        }),
        writes,
    )
}

#[test]
fn timeout_is_not_an_event_or_disconnect() {
    let (transport, writes) = mock(vec![Ok(vec![]), Ok(vec![1, 0, 255])], false);
    let mut session = Session::new(Model::Mini, transport, true).unwrap();
    assert_eq!(writes.lock().unwrap().len(), 1);
    assert!(session.read().unwrap().events.is_empty());
    assert_eq!(
        session.read().unwrap().events[0],
        Ok(ControlEvent::Analog {
            index: 0,
            raw: 255,
            maximum: 255
        })
    );
}

#[test]
fn bad_report_does_not_kill_session() {
    let (transport, _) = mock(vec![Ok(vec![1]), Ok(vec![2, 3, 1])], false);
    let mut session = Session::new(Model::Mini, transport, false).unwrap();
    assert!(session.read().unwrap().events[0].is_err());
    assert!(session.read().unwrap().events[0].is_ok());
}

#[test]
fn short_init_write_fails_and_read_errors_propagate() {
    let (transport, _) = mock(vec![], true);
    assert!(matches!(
        Session::new(Model::Pro, transport, true),
        Err(HardwareError::ShortWrite(2))
    ));
    let (transport, _) = mock(
        vec![Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "unplugged",
        ))],
        false,
    );
    assert!(Session::new(Model::Pro, transport, true)
        .unwrap()
        .read()
        .is_err());
}

#[test]
fn serial_never_writes_and_new_session_drops_partial_state() {
    let (transport, writes) = mock(vec![Ok(b"v0x".to_vec())], false);
    let mut session = Session::new(Model::Original, transport, true).unwrap();
    assert!(session.read().unwrap().events.is_empty());
    assert!(writes.lock().unwrap().is_empty());
    drop(session);
    let (transport, _) = mock(vec![Ok(b"73\nv1x42\npong\n".to_vec())], false);
    let batch = Session::new(Model::Original, transport, true)
        .unwrap()
        .read()
        .unwrap();
    assert!(batch.events[0].is_err());
    assert_eq!(
        batch.events[1],
        Ok(ControlEvent::Analog {
            index: 1,
            raw: 42,
            maximum: 100
        })
    );
    assert_eq!(batch.heartbeats, 1);
}
