//! Shared request storage for audited model copies and renderer producers.
//! Bounded imports and ordinary documents use the same semantic rules.
mod clone_with_storage;
pub(crate) use clone_with_storage::{clone_copy, clone_fields, try_clone, CloneWithStorage};
use std::convert::Infallible;

/// Owned request hook for containers whose public API cannot borrow a Storage.
/// Hash work is logical payload work, not an allocator/RSS guarantee.
pub(crate) trait Reservation: std::fmt::Debug + Send + Sync {
    fn hash_payload(&self, element_bytes: usize, capacity: usize) -> Result<(), String>;
    fn reject(&self, reason: &'static str) -> String;
}

pub(crate) trait Storage {
    type Error;
    const UNBOUNDED: bool = false;
    fn binary<P: crate::serializer::byte_writer::BinaryProducer>(
        &self,
        producer: &P,
    ) -> Result<Vec<u8>, Self::Error>;
    fn clone_table(
        &self,
        value: &crate::model::table::Table,
    ) -> Result<crate::model::table::Table, Self::Error> {
        self.clone_record(value)
    }
    fn clone_cell(
        &self,
        value: &crate::model::table::Cell,
    ) -> Result<crate::model::table::Cell, Self::Error> {
        self.clone_record(value)
    }
    fn clone_record<T: Clone + serde::Serialize>(&self, value: &T) -> Result<T, Self::Error>;
    fn clone_section_def(
        &self,
        value: &crate::model::document::SectionDef,
    ) -> Result<crate::model::document::SectionDef, Self::Error>;
    fn clone_paragraph(
        &self,
        value: &crate::model::paragraph::Paragraph,
    ) -> Result<crate::model::paragraph::Paragraph, Self::Error>;
    fn clone_doc_info(
        &self,
        value: &crate::model::document::DocInfo,
    ) -> Result<crate::model::document::DocInfo, Self::Error>;
    fn clone_document(
        &self,
        value: &crate::model::document::Document,
    ) -> Result<crate::model::document::Document, Self::Error>;
    fn bin_data(
        &self,
        bytes: &crate::model::bin_data::BinDataBytes,
        maximum: usize,
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    fn display<T: ?Sized + std::fmt::Display>(&self, value: &T) -> Result<String, Self::Error>;
    fn bytes(&self, count: usize) -> Result<(), Self::Error>;
    fn chars<I: Iterator<Item = char> + Clone>(&self, chars: I) -> Result<String, Self::Error>;

    fn vec<T>(&self, count: usize) -> Result<Vec<T>, Self::Error>;
    fn string(&self, text: &str) -> Result<String, Self::Error>;
    fn join(&self, values: &[String], separator: &str) -> Result<String, Self::Error>;
    fn reserve<T>(&self, values: &mut Vec<T>, additional: usize) -> Result<(), Self::Error>;
    fn reserve_map<K: Eq + std::hash::Hash, V>(
        &self,
        values: &mut std::collections::HashMap<K, V>,
        additional: usize,
    ) -> Result<(), Self::Error>;

    fn insert_map<K: Eq + std::hash::Hash, V>(
        &self,
        out: &mut std::collections::HashMap<K, V>,
        key: K,
        value: V,
    ) -> Result<Option<V>, Self::Error> {
        if !out.contains_key(&key) {
            self.reserve_map(out, 1)?;
        }
        Ok(out.insert(key, value))
    }

    fn insert_tree_map<K: Ord, V>(
        &self,
        out: &mut std::collections::BTreeMap<K, V>,
        key: K,
        value: V,
    ) -> Result<Option<V>, Self::Error> {
        if !out.contains_key(&key) {
            self.tree_entry::<K, V>()?;
        }
        Ok(out.insert(key, value))
    }

    fn reserve_set<T: Eq + std::hash::Hash>(
        &self,
        values: &mut std::collections::HashSet<T>,
        additional: usize,
    ) -> Result<(), Self::Error>;
    fn insert_set<T: Eq + std::hash::Hash>(
        &self,
        out: &mut std::collections::HashSet<T>,
        value: T,
    ) -> Result<bool, Self::Error> {
        if !out.contains(&value) {
            self.reserve_set(out, 1)?;
        }
        Ok(out.insert(value))
    }
    /// Logical ordered-map entry work; node allocation and process heap are separate.
    fn tree_entry<K, V>(&self) -> Result<(), Self::Error> {
        self.bytes(std::mem::size_of::<(K, V)>() + 4 * std::mem::size_of::<usize>())
    }

    fn reserve_string(&self, value: &mut String, additional: usize) -> Result<(), Self::Error>;
    fn push_char(&self, value: &mut String, ch: char) -> Result<(), Self::Error> {
        self.reserve_string(value, ch.len_utf8())?;
        value.push(ch);
        Ok(())
    }
    fn push_str(&self, value: &mut String, text: &str) -> Result<(), Self::Error> {
        self.reserve_string(value, text.len())?;
        value.push_str(text);
        Ok(())
    }

    fn extend<T, I: IntoIterator<Item = T>>(
        &self,
        out: &mut Vec<T>,
        values: I,
    ) -> Result<(), Self::Error> {
        for value in values {
            self.push(out, value)?;
        }
        Ok(())
    }
    fn set_from<T: Ord, I: IntoIterator<Item = T>>(
        &self,
        values: I,
    ) -> Result<std::collections::BTreeSet<T>, Self::Error> {
        let mut out = std::collections::BTreeSet::new();
        for value in values {
            self.insert_tree_set(&mut out, value)?;
        }
        Ok(out)
    }
    fn insert_tree_set<T: Ord>(
        &self,
        out: &mut std::collections::BTreeSet<T>,
        value: T,
    ) -> Result<(), Self::Error> {
        if !out.contains(&value) {
            self.tree_entry::<T, ()>()?;
            out.insert(value);
        }
        Ok(())
    }
    fn filled<T: Copy>(&self, count: usize, value: T) -> Result<Vec<T>, Self::Error> {
        let mut out = self.vec(count)?;
        out.resize(count, value);
        Ok(out)
    }
    fn stable_sort_by<T>(
        &self,
        values: &mut [T],
        compare: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> Result<(), Self::Error> {
        self.bytes(std::mem::size_of_val(values))?;
        values.sort_by(compare);
        Ok(())
    }
    fn collect<T, I: IntoIterator<Item = T>>(&self, input: I) -> Result<Vec<T>, Self::Error> {
        let input = input.into_iter();
        let mut result = self.vec(input.size_hint().0)?;
        for value in input {
            self.push(&mut result, value)?;
        }
        self.bytes(0)?;
        Ok(result)
    }
    fn push<T>(&self, values: &mut Vec<T>, value: T) -> Result<(), Self::Error> {
        self.reserve(values, 1)?;
        values.push(value);
        Ok(())
    }
    fn try_collect<T, I: IntoIterator<Item = Result<T, Self::Error>>>(
        &self,
        input: I,
    ) -> Result<Vec<T>, Self::Error> {
        let input = input.into_iter();
        let mut out = self.vec(input.size_hint().0)?;
        for value in input {
            self.push(&mut out, value?)?;
        }
        Ok(out)
    }
    fn unzip<A, B, I: IntoIterator<Item = (A, B)>>(
        &self,
        input: I,
    ) -> Result<(Vec<A>, Vec<B>), Self::Error> {
        let input = input.into_iter();
        let count = input.size_hint().0;
        let mut a = self.vec(count)?;
        let mut b = self.vec(count)?;
        for (x, y) in input {
            self.push(&mut a, x)?;
            self.push(&mut b, y)?;
        }
        Ok((a, b))
    }
    fn copy_slice<T: Copy>(&self, values: &[T]) -> Result<Vec<T>, Self::Error> {
        let mut result = self.vec(values.len())?;
        result.extend_from_slice(values);
        Ok(result)
    }
}

pub(crate) struct Unbounded;
impl Storage for Unbounded {
    const UNBOUNDED: bool = true;
    type Error = Infallible;
    fn collect<T, I: IntoIterator<Item = T>>(&self, input: I) -> Result<Vec<T>, Infallible> {
        Ok(input.into_iter().collect())
    }
    fn extend<T, I: IntoIterator<Item = T>>(
        &self,
        out: &mut Vec<T>,
        values: I,
    ) -> Result<(), Self::Error> {
        out.extend(values);
        Ok(())
    }
    fn unzip<A, B, I: IntoIterator<Item = (A, B)>>(
        &self,
        input: I,
    ) -> Result<(Vec<A>, Vec<B>), Self::Error> {
        Ok(input.into_iter().unzip())
    }
    fn binary<P: crate::serializer::byte_writer::BinaryProducer>(
        &self,
        producer: &P,
    ) -> Result<Vec<u8>, Self::Error> {
        let mut writer = crate::serializer::byte_writer::ByteWriter::new();
        producer.write(&mut writer).expect("Vec binary writer");
        Ok(writer.into_bytes())
    }
    fn bin_data(
        &self,
        bytes: &crate::model::bin_data::BinDataBytes,
        maximum: usize,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(bytes.load_limited(maximum))
    }
    fn display<T: ?Sized + std::fmt::Display>(&self, value: &T) -> Result<String, Self::Error> {
        Ok(value.to_string())
    }
    fn clone_document(
        &self,
        value: &crate::model::document::Document,
    ) -> Result<crate::model::document::Document, Self::Error> {
        Ok(value.clone())
    }
    fn clone_section_def(
        &self,
        value: &crate::model::document::SectionDef,
    ) -> Result<crate::model::document::SectionDef, Self::Error> {
        Ok(value.clone())
    }
    fn reserve_set<T: Eq + std::hash::Hash>(
        &self,
        values: &mut std::collections::HashSet<T>,
        additional: usize,
    ) -> Result<(), Self::Error> {
        values.reserve(additional);
        Ok(())
    }
    fn clone_paragraph(
        &self,
        value: &crate::model::paragraph::Paragraph,
    ) -> Result<crate::model::paragraph::Paragraph, Self::Error> {
        Ok(value.clone())
    }
    fn reserve_string(&self, value: &mut String, additional: usize) -> Result<(), Self::Error> {
        value.reserve(additional);
        Ok(())
    }
    fn clone_doc_info(
        &self,
        value: &crate::model::document::DocInfo,
    ) -> Result<crate::model::document::DocInfo, Self::Error> {
        Ok(value.clone())
    }
    fn clone_record<T: Clone + serde::Serialize>(&self, value: &T) -> Result<T, Self::Error> {
        Ok(value.clone())
    }
    fn bytes(&self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
    fn chars<I: Iterator<Item = char> + Clone>(&self, chars: I) -> Result<String, Infallible> {
        Ok(chars.collect())
    }

    fn vec<T>(&self, count: usize) -> Result<Vec<T>, Infallible> {
        Ok(Vec::with_capacity(count))
    }
    fn string(&self, text: &str) -> Result<String, Infallible> {
        Ok(text.to_owned())
    }
    fn join(&self, values: &[String], separator: &str) -> Result<String, Infallible> {
        Ok(values.join(separator))
    }
    fn reserve_map<K: Eq + std::hash::Hash, V>(
        &self,
        values: &mut std::collections::HashMap<K, V>,
        additional: usize,
    ) -> Result<(), Infallible> {
        values.reserve(additional);
        Ok(())
    }
    fn reserve<T>(&self, values: &mut Vec<T>, additional: usize) -> Result<(), Infallible> {
        values.reserve(additional);
        Ok(())
    }
}

/// Run an optional proof without treating a failed reservation as a negative proof.
/// `optional(result, failure)?` stops the proof immediately and preserves its error.
pub(crate) fn with_optional<T, E>(
    operation: impl FnOnce(&mut Option<E>) -> Option<T>,
) -> Result<Option<T>, E> {
    let mut failure = None;
    let value = operation(&mut failure);
    match failure {
        Some(error) => Err(error),
        None => Ok(value),
    }
}
pub(crate) fn optional<T, E>(result: Result<T, E>, failure: &mut Option<E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            *failure = Some(error);
            None
        }
    }
}
