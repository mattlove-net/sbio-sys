mod sbio;
use sbio::sbio_sys::*;
use std::sync::{Arc, Mutex};
use std::thread::*;
use std::result::Result;

pub struct SbioSerializeData {
    buffer: sbio_serialized_data,
}

impl SbioSerializeData {
    pub fn name(&mut self) -> &str {
        unserialize_event_name(&self.buffer)
    }

    pub fn target(&mut self)  -> &str {
        unserialize_event_target(&self.buffer)
    }

    pub fn format(&mut self)  -> &str {
        unserialize_event_format(&self.buffer)
    }

    pub fn data<T>(&mut self) -> Result<&T, &'static str> {
        let (_, _, _, data, _) = unserialize(&self.buffer);
        Ok(data)
    }
}

impl Drop for SbioSerializeData {
    fn drop(&mut self) {
        free_buffer(&self.buffer)
    }
}

struct SbioConnectionData {
    channel_handle: sbio_channel_handle,
    channel_open: bool,
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
}

impl Drop for SbioConnection {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone, Debug)]
pub struct Sbio();

impl Sbio {
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

    #[derive(PartialEq, Debug, Clone)]
    struct TestData {
        var1: u32,
        var2: u16,
        var3: u16,
    }

    #[test]
    fn open_test() {
        let mut sbio = Sbio();
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
    fn serialize_test() {
        let mut sbio = Sbio();
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

    #[test]
    fn unserialize_test() {
        let mut sbio = Sbio();
        let target_in = "target";
        let name_in = "event1";
        let format_in = "4s1 var1 2u1 var2 2u1 var2";
        let data_in = TestData {
            var1: 1,
            var2: 2,
            var3: 3,
        };
        let size_in = 10;

        let result = sbio.serialize(target_in, name_in, format_in, data_in.clone(), size_in);
        let mut serialized_data = match result {
            Ok(data) => data,
            Err(_) => panic!(),
        };

        let name = serialized_data.name();
        assert_eq!(name, name_in);

        let target = serialized_data.target();
        assert_eq!(target, target_in);
        
        let format = serialized_data.format();
        assert_eq!(format, format_in);
    
        let result: Result<&TestData, &'static str> = serialized_data.data();
        let data = match result {
            Ok(data) => data,
            Err(_) => panic!(),
        };
        assert_eq!(data, &data_in);
    }

    #[test]
    fn send_receive_event_test() {
        let mut sbio = Sbio();
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
}
