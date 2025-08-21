# Adding An Encodable/Decodable Protocol Data Type

## Context

In this context, a data type is defined as a structure used to encode a certain value having a distinct specification.

We are working with raw bytes coming from the server in Big Endian format (the most significant byte is first in the
data stream).

## Meta

All logic for data types is written inside the `/src/net/packet/data_types.rs`
file.

## Step-By-Step Guide

I.
Find the data type you want to include and get to know its specs. The specs for the protocol data types can be found
here: https://minecraft.wiki/w/Java_Edition_protocol/Data_types

II.
Write a struct for the type. We will take the example of the `VarInt` type from now on.
`struct VarInt {}` that will have to have and implement multiple things:

Struct members:

- `value` that will contain the value, an `i32`[^1] for a signed `VarInt`.
- `bytes` that will contain the raw bytes of what the type is. Should always be a contiguous array of `u8`, like a
  `Vec<u8>` or a fixed-size `[u8; N]` array;

Traits:  
The struct should derive the same traits as the custom `Encodable` trait:  
`Debug`, `Default`, `Clone`, `PartialEq` and `Eq`.

Our `VarInt` should look like this:

```rs
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VarInt {
    // We're storing both the value and bytes to avoid redundant conversions.
    value: i32,
    bytes: Vec<u8>,
}
```

III. (The semi-hard part)
Implement the methods and traits for the type.

III. A

As I like to do it, make two private functions to your struct, `read()` and `write()`, the first takes in an arbitrary
long array of bytes, and the other creates a `VarInt` from a value (`i32`).

Also, the `read()` function should only look for your type from the beginning of the bytes. Meaning that if you input
100 bytes into the `read()` function and there is a `VarInt` encoded in the first 2, it will decode it and just ignore
the remaining 98.

*In any case you will have to make those two, because the `Encodable` traits required two similar functions.*

The signatures of our two functions look like:

```rs
fn read<T: AsRef<[u8]>>(data: T) -> Result<(i32, usize), CodecError>;

fn write(mut value: i32) -> Result<Vec<u8>, CodecError>;
```

III. B

Implement the `Encodable` trait, it requires to write two functions and three methods:

- The `from_bytes()` function which creates your type from an array of bytes.
- The `from_value()` function which creates your type from a `ValueInput` type, which you'll most likely implement as a
  `i32` for a `VarInt`, for instance.
- The `get_bytes()` method which acts like a getter. Give access to the inner `bytes` member.
- The `get_value()` method which is the same but for the `value` member.
- The `len()` method which returns the size of the raw bytes of your types. I.e., a simple `self.get_bytes().len()`.

Make sure to also implement the `ValueInput` (used by `from_value()`) and `ValueOutpue` (used by `get_value()`) types,
both `i32` for `VarInt`.
By the way, `Encodable` auto-implements a convenience function for your custom type!

`consume_from_bytes()` accepts a mutable byte buffer and consumes exactly the bytes representing the target type. For
example, from a 100-byte buffer, parsing a 2-byte VarInt removes those 2 bytes in place, leaving 98.

IV. (Paradoxically the hard/annoying part)
Wiring your type into the whole ecosystem of types, testament to my level of ignorance to good structuring.

This is the more flimsy part of this whole `data_types.rs` structure. For this part, refer heavily on compiler error and
warnings to see if you messed up or forgot to wire something.

For these steps make sure to also implement the `impl` blocks of each struct/enum.

There is at least four places to add your data type in.

- Add your data type to the `DataType` enum.
- Add your data type to the `DataTypeContent` enum.

V. Write tests (very important)

Now unit test your data type very thoroughly to make sure it works even (and most importantly) on edge cases.

The best would be to test ALL public methods for your data type.

[^1]: I am now in fact not so sure a signed integer is the appropriate type for a `VarInt`,
because a Minecraft `VarInt` can hold five bytes at maximum.