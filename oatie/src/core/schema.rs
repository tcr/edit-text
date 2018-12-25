use super::doc::*;
use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Track: Copy + Debug + PartialEq + Sized {
    // Rename this do close split? if applicable?
    fn do_split(&self) -> bool;

    // Unsure about this naming
    fn do_open_split(&self) -> bool;

    fn supports_text(&self) -> bool;

    fn allowed_in_root(&self) -> bool;

    // TODO is this how this should work
    fn is_object(&self) -> bool;

    fn parents(&self) -> Vec<Self>;

    // TODO extrapolate this from parents()
    fn ancestors(&self) -> Vec<Self>;
}

pub trait Schema: Clone + Debug + PartialEq {
    type Track: Track + Sized;

    type GroupProperties: Sized + Clone + Debug + Serialize + PartialEq + DeserializeOwned;
    type CharsProperties: Sized + Clone + Debug + Serialize + PartialEq + DeserializeOwned + Default + StyleTrait;

    /// Determines if two sets of Attrs are equal.
    fn attrs_eq(a: &Self::GroupProperties, b: &Self::GroupProperties) -> bool;

    /// Get the track type from this Attrs.
    fn track_type_from_attrs(attrs: &Self::GroupProperties) -> Option<Self::Track>;

    /// Combine two Attrs into a new definition.
    fn merge_attrs(a: &Self::GroupProperties, b: &Self::GroupProperties) -> Option<Self::GroupProperties>;
}
