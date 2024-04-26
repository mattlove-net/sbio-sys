mod observer;
mod sbio;
use sbio::sbio_sys::*;
//use observer::*;

impl SBIO_FLAGS {
    pub fn as_u32(&self) -> u32 {
        self.bits()
    }
}

pub struct SbioSerializeData {
    buffer: sbio_serialized_data,
}

pub struct SbioConnection {
    channel_handle: sbio_channel_handle,
}

impl SbioConnection {
    pub fn close(&mut self) {
        close(&self.channel_handle)
    }

    pub fn send_event(&mut self, event: &SbioSerializeData) -> Result<i32, &'static str> {
        send(&self.channel_handle, &event.buffer)
    }

    pub fn send<T>(
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
        free_buffer(event);

        ret
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
    pub fn connect(channel_name: &str, flags: SBIO_FLAGS) -> Result<SbioConnection, &'static str> {
        let handle = match open(channel_name, flags) {
            Ok(handle) => handle,
            Err(err) => return Err(err),
        };

        Ok(SbioConnection {
            channel_handle: handle,
        })
    }

    pub fn serialize<T>(
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

// pub fn create_send_channel(channel_name: &str, flags: SBIO_FLAGS) -> SbioConnection {
//     let channel_handle = open(channel_name, SBIO_FLAGS::RDONLY | flags);
//     SbioConnection { channel_handle: channel_handle }
// }

// pub fn create_receive_channel(channel_name: &str, flags: SBIO_FLAGS) -> SbioConnection {
//     let channel_handle = open(channel_name, SBIO_FLAGS::RDONLY | flags);
//     SbioConnection { channel_handle: channel_handle }
// }

// pub fn destroy_channel(handle: SbioConnection) {
//     close(handle.channel_handle)
// }

// pub fn send_event<T>(
//     _handle: SbioConnection,
//     _name: String,
//     _format: String,
//     _data: &[T],
// ) -> bool {
//     true
// }

// pub fn add_event_callback<T>(
//     _handle: SbioConnection,
//     _name: String,
//     _callback_function: fn(name: String, format: String, data: &[T]),
// ) -> bool {
//     let mut _subject: Subject<T> = Subject::new();
//     true
// }

// pub fn remove_event_callback<T>(
//     _handle: SbioConnection,
//     _callback_function: fn(name: String, format: String, data: &[T]),
// ) -> bool {
//     true
// }

#[cfg(test)]
mod tests {
    use super::*;
    use observer::*;
    use sbio::*;
}
