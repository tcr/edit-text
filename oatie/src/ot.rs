/// A type that can have operational transform applied to it.
/// The `OT` trait is implemented on an operation object, and its
/// associated type `Doc` is what the operation should operate on.
pub trait OT
where
    Self: Sized,
{
    type Doc;

    /// Applies an operation to a `Self::Doc`, returning the modified `Self::Doc`.
    fn apply(doc: &Self::Doc, op: &Self) -> Self::Doc;

    /// Returns an empty operation.
    fn empty() -> Self;

    /// Composes two operations, returning a single operation encapsulating them
    /// both.
    fn compose(a: &Self, b: &Self) -> Self;

    /// Composes an iterator of operations into a single operation.
    /// If no operations are returned from the iterator, the Op::empty() should be
    /// returned.
    fn compose_iter<'a, I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
        Self: 'a;

    /// Transform a document given the corresponding Schema trait.
    fn transform(a: &Self, b: &Self) -> (Self, Self);

    /// Utility function to transform an operation against a competing one,
    /// returning the results of composing them both.
    fn transform_advance(a: &Self, b: &Self) -> Self;
}
