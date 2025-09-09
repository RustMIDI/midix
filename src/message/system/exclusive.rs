use alloc::borrow::Cow;

#[doc = r#"
A System Exclusive messsage, found in
both [`LiveEvent`](crate::prelude::LiveEvent)s and [`FileEvent`](crate::prelude::FileEvent)s.

# Overview

System Exclusive messages include a
Manufacturer's Identification (ID) code,
and are used to transfer any number of
data bytes in a format specified by the
referenced manufacturer.

Exclusive messages can contain any number of Data bytes, and can be
terminated either by an End of Exclusive (EOX) or any other Status byte (except
Real Time messages). An EOX should always be sent at the end of a System
Exclusive message. These messages include a Manufacturer's Identification (ID)
code. If a receiver does not recognize the ID code, it should ignore the following
data.

# Layout
System Exclusive.
```text
0iiiiiii
0ddddddd
..
..
0ddddddd
11110111

This message makes up for all that MIDI doesn't support.
(iiiiiii) is usually a seven-bit Manufacturer's I.D. code.
If the synthesiser recognises the I.D. code as its own, it
will listen to the rest of the message (ddddddd).

Otherwise, the message will be ignored. System Exclusive
is used to send bulk dumps such as patch parameters and
other non-spec data. (Note: Real-Time messages ONLY may
be interleaved with a System Exclusive.) This message also
is used for extensions called Universal Exclusive Messages.
```
"#]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct SystemExclusiveMessage<'a>(Cow<'a, [u8]>);

impl<'a> SystemExclusiveMessage<'a> {
    /// Create a new owned system exclusive message
    pub fn new<B: Into<Cow<'a, [u8]>>>(data: B) -> Self {
        Self(data.into())
    }

    // /// Returns a mutable reference to the underlying data.
    // pub fn data_mut(&mut self) -> &mut Vec<u8> {
    //     self.0.to_mut()
    // }

    /// Get the length of the sysex data
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// returns true without sysex data
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // /// Interprets the sysex as a live-streamed set of bytes.
    // ///
    // /// Note that live bytes don't have an identifying length, unlike a file system common message.
    // pub fn to_live_bytes(&self) -> Vec<u8> {
    //     let mut bytes = Vec::with_capacity(self.len() + 2);
    //     bytes.push(0xF0);
    //     bytes.extend(self.0.iter());
    //     bytes.push(0xF7);
    //     bytes
    // }
}
