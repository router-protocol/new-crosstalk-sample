pub mod types {
    use ink::prelude::string::String;
    use ink::prelude::string::ToString;
    use ink::prelude::vec::Vec as InkVec;

    pub type Bytes = InkVec<u8>;

    /// The Gateway result type.
    pub type Result<T> = core::result::Result<T, TestError>;

    // get bytes to string
    pub fn _get_utf8_bytes_to_string(bytes_string: &Bytes) -> String {
        String::from_utf8_lossy(&bytes_string.as_slice()).to_string()
    }

    /// The Gateway error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum TestError {
        DecodeError,
        EmptyGreeting,
        NotFound,
    }
}

pub use types::*;
