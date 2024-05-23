pub mod types {
    use ink::prelude::vec::Vec as InkVec;

    pub type Bytes = InkVec<u8>;

    /// The Gateway result type.
    pub type Result<T> = core::result::Result<T, TestError>;

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
