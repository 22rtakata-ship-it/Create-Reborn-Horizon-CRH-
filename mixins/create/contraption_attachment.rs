// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT

use super::{AttachmentType, Attachments, Entry, raw_value};
use crate::nbt::Compound;

/// The encoded data is already present on both the server and the client.
pub fn encoded<'a>(map: &'a Attachments, namespace: &str, key: &str) -> Option<&'a Compound> {
    match map.entries.get(&(namespace.into(), key.into()))? {
        Entry::Raw(value) => raw_value(value),
        Entry::Decoded(value) => value.encoded(),
    }
}

/// Borrow a cached typed value without cloning or changing its save policy.
pub fn decoded<'a, T: 'static>(map: &'a Attachments, ty: &AttachmentType<T>) -> Option<&'a T> {
    match map.entries.get(&(ty.namespace.into(), ty.name.into()))? {
        Entry::Decoded(value) => value.as_any().downcast_ref::<T>(),
        Entry::Raw(_) => None,
    }
}
