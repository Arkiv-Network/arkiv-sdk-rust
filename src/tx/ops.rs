pub mod chown;
pub mod create;
pub mod delete;
pub mod extend;
pub mod update;

pub use chown::Chown;
pub use create::{Create, CreateBuilder};
pub use delete::Delete;
pub use extend::Extend;
pub use update::{Update, UpdateBuilder};

/// A trait for attaching attributes to a transaction's operation.
///
/// Implementors provide distinct behavior depending on the wrapper type used
/// (e.g. string attributes vs numeric attributes).
pub trait WithAttribute<A> {
    /// Add a single attribute to a transaction type.
    fn with_attribute(self, attribute: A) -> Self;

    /// Extend attributes from any type which can produce an iterator.
    /// This can be particularly useful for avoiding duplicate keys
    /// using `std::collections::HashMap`.
    fn extend_attributes<I>(self, iter: I) -> Self
    where
        I: IntoIterator<Item = A>;
}
