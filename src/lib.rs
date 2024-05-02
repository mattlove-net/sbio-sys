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

pub struct EventObserver {
    event: String,
    callback: Box<dyn Fn(&SbioSerializeData)>,
}

impl IObserver<SbioSerializeData> for EventObserver {
    fn update(&mut self, event: &SbioSerializeData) {
        (self.callback)(event);
    }
}

impl IObserver<String> for EventObserver {
    fn update(&mut self, _event: &String) {}
}

#[allow(dead_code)]
pub struct SbioConnection {
    channel_handle: sbio_channel_handle,
    flags: u32,
    stop_receive_thread: bool,
    thread_handle: Option<JoinHandle<()>>,
    subscriptions: HashMap<String, Subject<SbioSerializeData>>,
}

impl SbioConnection {
    // Close the SBIO channel and free the handle
    pub fn close(&mut self) {
        close(&self.channel_handle)
    }

    // Send a serialized event
    pub fn send_serialized_event(
        &mut self,
        event: &SbioSerializeData,
    ) -> Result<i32, &'static str> {
        send(&self.channel_handle, &event.buffer)
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
        let event = match serialize(target, name, format, data, size) {
            Ok(event) => event,
            Err(err) => panic!("Problem sending event: {:?}", err),
        };

        let ret = send(&self.channel_handle, &event);
        free_buffer(&event);

        ret
    }

    // Receive a serialized event
    pub fn receive(&mut self) -> Result<SbioSerializeData, &'static str> {
        let buffer = match receive(&self.channel_handle) {
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

        self.stop_receive_thread = false;
        self.thread_handle = Some(spawn(move || {
            // loop {
            //     if self.stop_receive_thread {
            //         break;
            //     }

            //     let buffer = match receive(&self.channel_handle) {
            //         Ok(buffer) => buffer,
            //         Err(err) => panic!("Problem receiving event: {:?}", err),
            //     };

            //     let (target, name, format, data, size) = unserialize(&buffer);
            //     let subject = self.subscriptions.get_mut(name);
            //     match subject {
            //         Some(subject) => {
            //             let event = SbioSerializeData { buffer };
            //             subject.notify_observers(&event);
            //         }
            //         None => {
            //             free_buffer(&buffer);
            //         }
            //     }
            // }
        }));

        Ok(())
    }

    pub fn stop_receive_thread(&mut self) {
        self.stop_receive_thread = true;
        if let Some(th) = self.thread_handle.take() {
            let _ = th.join();
        }
    }

    // Register an event callback for receiving an event
    pub fn add_event_callback<T>(
        &mut self,
        name: String,
        callback_function: fn(event: &SbioSerializeData),
    ) -> Arc<Mutex<EventObserver>> {
        //let mut subject: Subject<T> = Subject::new();
        let subject = self
            .subscriptions
            .entry(name.clone())
            .or_insert(Subject::new());

        #[allow(clippy::arc_with_non_send_sync)]
        let observer = Arc::new(Mutex::new(EventObserver {
            event: name,
            callback: Box::new(callback_function),
        }));
        subject.add_observer(observer.clone());
        observer
    }

    // Remove an event callback
    pub fn remove_event_callback<T>(&mut self, observer: Arc<Mutex<EventObserver>>) -> bool {
        let subject = self.subscriptions.get_mut(&observer.lock().unwrap().event);
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

        Ok(SbioConnection {
            channel_handle: handle,
            flags: 0,
            stop_receive_thread: false,
            thread_handle: Some(spawn(|| {})),
            subscriptions: HashMap::new(),
        })
    }

    pub fn connect_send(&mut self, channel_name: &str) -> Result<SbioConnection, &'static str> {
        self.connect(channel_name, SBIO_FLAGS::WRONLY)
    }

    pub fn connect_receive(&mut self, channel_name: &str) -> Result<SbioConnection, &'static str> {
        self.connect(channel_name, SBIO_FLAGS::RDONLY)
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
}
