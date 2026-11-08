//! Fallible JSON literals. Value/string/container reservations happen while
//! building the tree; a completed unbounded json! tree is never the preflight.
macro_rules! request_json {
    (@value $a:ident null) => { Ok::<serde_json::Value, $crate::parser::pdf::allocation::Failure>(serde_json::Value::Null) };
    (@value $a:ident [$($items:tt)*]) => {{
        let mut items=Vec::new();
        request_json!(@array $a items $($items)*);
        Ok::<serde_json::Value, $crate::parser::pdf::allocation::Failure>(serde_json::Value::Array(items))
    }};
    (@value $a:ident {$($items:tt)*}) => {{
        let mut object=$a.object()?;
        request_json!(@object $a object $($items)*);
        Ok::<serde_json::Value, $crate::parser::pdf::allocation::Failure>(serde_json::Value::Object(object))
    }};
    (@value $a:ident $value:expr) => { Ok::<serde_json::Value, $crate::parser::pdf::allocation::Failure>($a.value(&($value))?) };
    (@array $a:ident $out:ident) => {};
    (@array $a:ident $out:ident null $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a null)?; $a.reserve(&mut $out,1)?; $out.push(value);
        request_json!(@array $a $out $($($rest)*)?);
    }};
    (@array $a:ident $out:ident {$($value:tt)*} $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a {$($value)*})?; $a.reserve(&mut $out,1)?; $out.push(value);
        request_json!(@array $a $out $($($rest)*)?);
    }};
    (@array $a:ident $out:ident [$($value:tt)*] $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a [$($value)*])?; $a.reserve(&mut $out,1)?; $out.push(value);
        request_json!(@array $a $out $($($rest)*)?);
    }};
    (@array $a:ident $out:ident $value:expr $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a $value)?; $a.reserve(&mut $out,1)?; $out.push(value);
        request_json!(@array $a $out $($($rest)*)?);
    }};
    (@object $a:ident $out:ident) => {};
    (@object $a:ident $out:ident $key:literal : null $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a null)?; $a.object_entry(&mut $out,$key,value)?;
        request_json!(@object $a $out $($($rest)*)?);
    }};
    (@object $a:ident $out:ident $key:literal : {$($value:tt)*} $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a {$($value)*})?; $a.object_entry(&mut $out,$key,value)?;
        request_json!(@object $a $out $($($rest)*)?);
    }};
    (@object $a:ident $out:ident $key:literal : [$($value:tt)*] $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a [$($value)*])?; $a.object_entry(&mut $out,$key,value)?;
        request_json!(@object $a $out $($($rest)*)?);
    }};
    (@object $a:ident $out:ident $key:literal : $value:expr $(,$($rest:tt)*)?) => {{
        let value=request_json!(@value $a $value)?; $a.object_entry(&mut $out,$key,value)?;
        request_json!(@object $a $out $($($rest)*)?);
    }};
    ($allocation:expr,$($value:tt)+) => {{
        let allocation=$allocation;
        (|| -> Result<serde_json::Value,$crate::parser::pdf::allocation::Failure> {
            request_json!(@value allocation $($value)+)
        })()
    }};
}

pub(super) use request_json;
