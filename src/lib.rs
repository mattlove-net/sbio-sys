mod observer;
mod sbio;
use observer::*;
use sbio::sbio_sys::*;
use std::sync::{Arc, Mutex};
use std::thread::*;
use std::{collections::HashMap, result::Result};

pub struct SbioSerializeData {
    buffer: sbio_serialized_data,
}

impl SbioSerializeData {
    pub fn unserialize<'a, T>(&mut self) -> (&'a str, &'a str, &'a str, &T, i32) {
        unserialize(&self.buffer)
    }
}

impl Drop for SbioSerializeData {
    fn drop(&mut self) {
        free_buffer(&self.buffer)
    }
}

pub trait IEventCallback {
    fn callback(&self, event: &SbioSerializeData);
}

pub struct EventObserver {
    event: String,
    callback: Box<dyn IEventCallback>,
}

impl IObserver<SbioSerializeData> for EventObserver {
    fn update(&mut self, event: &SbioSerializeData) {
        self.callback.callback(event);
    }
}

struct SbioConnectionData {
    channel_handle: sbio_channel_handle,
    channel_open: bool,
    stop_receive_thread: bool,
    subscriptions: HashMap<String, Subject<SbioSerializeData>>,
}

#[allow(dead_code)]
pub struct SbioConnection {
    flags: u32,
    thread_handle: Option<JoinHandle<()>>,
    thread_data: Arc<Mutex<SbioConnectionData>>,
}

impl SbioConnection {
    // Close the SBIO channel and free the handle
    pub fn close(&mut self) {
        let mut thread_data = self.thread_data.lock().unwrap();
        if thread_data.channel_open {
            let sbio_channel_handle = &mut thread_data.channel_handle;
            close(sbio_channel_handle);
            thread_data.channel_open = false;
        }
    }

    // Send a serialized event
    pub fn send_serialized_event(
        &mut self,
        event: &SbioSerializeData,
    ) -> Result<i32, &'static str> {
        let thread_data = self.thread_data.lock().unwrap();
        send(&thread_data.channel_handle, &event.buffer)
    }

    // Send a event with the event target, name, format, data, and size
    pub fn send_event<T>(
        &mut self,
        target: &str,
        name: &str,
        format: &str,
        data: T,
        size: u32,
    ) -> Result<i32, &'static str> {
        let thread_data = self.thread_data.lock().unwrap();
        let event = match serialize(target, name, format, data, size) {
            Ok(event) => event,
            Err(err) => panic!("Problem sending event: {:?}", err),
        };

        let ret = send(&thread_data.channel_handle, &event);
        free_buffer(&event);

        ret
    }

    // Receive a serialized event
    pub fn receive(&mut self) -> Result<SbioSerializeData, &'static str> {
        let thread_data = self.thread_data.lock().unwrap();
        let buffer = match receive(&thread_data.channel_handle) {
            Ok(buffer) => buffer,
            Err(err) => return Err(err),
        };

        Ok(SbioSerializeData { buffer })
    }

    // Start a thread to receive events
    pub fn start_receive_thread(&mut self) -> Result<(), &'static str> {
        if let Some(th) = self.thread_handle.take() {
            th.is_finished();
        }

        let thread_data = self.thread_data.clone();
        self.thread_handle = Some(spawn(move || loop {
            let mut td = thread_data.lock().unwrap();
            if td.stop_receive_thread {
                break;
            }

            let buffer = match receive(&td.channel_handle) {
                Ok(buffer) => buffer,
                Err(_err) => continue, //Err("Problem receiving event: {:?}", err)
            };

            let name = unserialize_event_name(&buffer);
            let subject = td.subscriptions.get_mut(name);
            match subject {
                Some(subject) => {
                    let event = SbioSerializeData { buffer };
                    subject.notify_observers(&event);
                }
                None => {
                    free_buffer(&buffer);
                }
            }
        }));

        Ok(())
    }

    pub fn stop_receive_thread(&mut self) {
        let mut thread_data = self.thread_data.lock().unwrap();
        thread_data.stop_receive_thread = true;
        drop(thread_data);
        if let Some(th) = self.thread_handle.take() {
            let _ = th.join();
        }
    }

    // Register an event callback for receiving an event
    pub fn add_event_callback(
        &mut self,
        name: String,
        callback: Box<dyn IEventCallback>,
    ) -> Arc<Mutex<EventObserver>> {
        //let mut subject: Subject<T> = Subject::new();
        let mut thread_data = self.thread_data.lock().unwrap();
        let subject = thread_data
            .subscriptions
            .entry(name.clone())
            .or_insert(Subject::new());

        #[allow(clippy::arc_with_non_send_sync)]
        let observer: Arc<Mutex<EventObserver>> = Arc::new(Mutex::new(EventObserver {
            event: name,
            callback,
        }));
        subject.add_observer(observer.clone());
        observer
    }

    // Remove an event callback
    pub fn remove_event_callback<T>(&mut self, observer: Arc<Mutex<EventObserver>>) -> bool {
        let mut thread_data = self.thread_data.lock().unwrap();
        let subject = thread_data
            .subscriptions
            .get_mut(&observer.lock().unwrap().event);
        match subject {
            Some(subject) => {
                subject.remove_observer(observer);
                true
            }
            None => false,
        }
    }
}

impl Drop for SbioConnection {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone, Debug)]
pub struct SbioWrapper();

impl SbioWrapper {
    pub fn connect(
        &mut self,
        channel_name: &str,
        flags: SBIO_FLAGS,
    ) -> Result<SbioConnection, &'static str> {
        let handle = match open(channel_name, flags) {
            Ok(handle) => handle,
            Err(err) => return Err(err),
        };

        let connection_data = SbioConnectionData {
            channel_handle: handle,
            channel_open: true,
            stop_receive_thread: false,
            subscriptions: HashMap::new(),
        };

        Ok(SbioConnection {
            flags: 0,
            thread_handle: Some(spawn(|| {})),
            thread_data: Arc::new(Mutex::new(connection_data)),
        })
    }

    pub fn connect_send(&mut self, channel_name: &str) -> Result<SbioConnection, &'static str> {
        self.connect(channel_name, SBIO_FLAGS::WRONLY)
    }

    pub fn connect_receive(&mut self, channel_name: &str) -> Result<SbioConnection, &'static str> {
        self.connect(channel_name, SBIO_FLAGS::RDONLY | SBIO_FLAGS::NONBLOCK)
    }

    pub fn serialize<T>(
        &mut self,
        target: &str,
        name: &str,
        format: &str,
        data: T,
        size: u32,
    ) -> Result<SbioSerializeData, &'static str> {
        let buffer = match serialize(target, name, format, data, size) {
            Ok(buffer) => buffer,
            Err(err) => return Err(err),
        };

        Ok(SbioSerializeData { buffer })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observer::*;
    use sbio::*;

    struct TestData {
        var1: u32,
        var2: u16,
        var3: u16,
    }

    #[test]
    fn open_test() {
        let mut sbio = SbioWrapper();
        let rcv = match sbio.connect_receive("open_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem opening channel: {:?}", err),
        };

        let send = match sbio.connect_send("open_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem opening channel: {:?}", err),
        };
        drop(rcv);
        drop(send);
    }

    #[test]
    fn send_receive_event_test() {
        let mut sbio = SbioWrapper();
        let mut rcv = match sbio.connect_receive("send_receive_event_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem receiving event: {:?}", err),
        };
        let mut send = match sbio.connect_send("send_receive_event_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem sending event: {:?}", err),
        };
        let result = send.send_event(
            "target",
            "event1",
            "4s1 var1 2u1 var2 2u1 var3",
            TestData {
                var1: 1,
                var2: 2,
                var3: 3,
            },
            10,
        );
        assert!(result.is_ok());
        let result = rcv.receive();
        assert!(result.is_ok());
        rcv.close();
        send.close();
    }

    #[test]
    fn async_receive_test() {
        let mut sbio = SbioWrapper();
        let mut rcv = match sbio.connect_receive("async_receive_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem receiving event: {:?}", err),
        };
        // Send an event
        let mut send = match sbio.connect_send("async_receive_test") {
            Ok(connection) => connection,
            Err(err) => panic!("Problem sending event: {:?}", err),
        };

        let event_received = Arc::new(Mutex::new(false));
        let received = event_received.clone();
        struct TestCallback {
            received: Arc<Mutex<bool>>,
        }
        impl IEventCallback for TestCallback {
            fn callback(&self, event: &SbioSerializeData) {
                let mut received = self.received.lock().unwrap();
                *received = true;
            }
        }

        let callback = TestCallback { received: received };
        let observer = rcv.add_event_callback("event1".to_string(), Box::new(callback));
        let result = rcv.start_receive_thread();
        assert!(result.is_ok());

        let result = send.send_event(
            "target",
            "event1",
            "4s1 var1 2u1 var2 2u1 var3",
            TestData {
                var1: 1,
                var2: 2,
                var3: 3,
            },
            10,
        );
        assert!(result.is_ok());

        let received = event_received.clone();
        loop {
            let received = received.lock().unwrap();
            if *received {
                break;
            }
        }

        rcv.stop_receive_thread();
        rcv.close();
        send.close();
    }

    #[test]
    fn serialize_test() {
        let mut sbio = SbioWrapper();
        let target_in = "target";
        let name_in = "event1";
        let format_in = "4s1 var1 2u1 var2 2u1 var2";
        let data_in = TestData {
            var1: 1,
            var2: 2,
            var3: 3,
        };
        let size_in = 10;

        let result = sbio.serialize(target_in, name_in, format_in, data_in, size_in);
        assert!(result.is_ok());
    }
}
