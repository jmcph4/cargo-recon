use std::{
    fmt,
    path::{Path, PathBuf},
};

use log::info;
use rustdoc_types::{GenericArg, GenericArgs, Item, ItemEnum, Type};
use serde::Serialize;

use crate::rustdoc::{build_rustdoc, functions, functions_with_visibility};

#[derive(Copy, Clone, Debug)]
pub enum ParamTypeFilter {
    BinaryOnly,
    BinaryOrString,
    Arbitrary,
    Any,
}

impl ParamTypeFilter {
    pub fn strings_allowed(&self) -> bool {
        !matches!(self, Self::BinaryOnly)
    }
}

impl Default for ParamTypeFilter {
    fn default() -> Self {
        Self::BinaryOrString
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ParamCoverageFilter {
    Any,
    All,
    None,
}

impl Default for ParamCoverageFilter {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub visibility: Option<rustdoc_types::Visibility>,
    pub param_type: ParamTypeFilter,
    pub param_coverage: ParamCoverageFilter,
}

#[derive(Clone, Debug, Serialize)]
pub struct Target {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
}

impl From<Item> for Target {
    fn from(value: Item) -> Self {
        let name = value.name.unwrap();
        let span = value.span.unwrap();
        Self {
            name,
            file_path: span.filename,
            line: span.begin.0,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.file_path.display(),
            self.line,
            self.name
        )
    }
}

pub fn search_file<P>(
    path: P,
    filter: Option<Filter>,
) -> eyre::Result<Vec<Target>>
where
    P: AsRef<Path>,
{
    info!("Commencing search with {filter:?}");
    let root = build_rustdoc(path)?;
    let mut candidates = functions(&root);
    let mut candidates_after_params: Vec<Item> = vec![];

    if let Some(f) = filter {
        if let Some(vis) = f.visibility {
            candidates = functions_with_visibility(&root, vis);
        }

        for curr_item in &candidates {
            if let ItemEnum::Function(curr_func) = &curr_item.inner {
                let mut found_fuzzable_param = false;
                let mut all_params_fuzzable = true;

                for (_, param_type) in &curr_func.sig.inputs {
                    dbg!(&param_type);
                    if is_fuzzable_type(param_type, f.param_type) {
                        found_fuzzable_param = true;
                    } else {
                        all_params_fuzzable = false;
                    }
                }

                if found_fuzzable_param {
                    if let ParamCoverageFilter::All = f.param_coverage {
                        if all_params_fuzzable {
                            candidates_after_params.push(curr_item.clone());
                        }
                    } else {
                        candidates_after_params.push(curr_item.clone());
                    }
                }
            }
        }
    } else {
        candidates_after_params = candidates.to_vec();
    }

    Ok(candidates_after_params
        .iter()
        .cloned()
        .map(Target::from)
        .collect())
}

fn is_fuzzable_type(ty: &Type, filter: ParamTypeFilter) -> bool {
    match ty {
        // Matches a reference type like &[u8]
        Type::BorrowedRef {
            lifetime: _,
            is_mutable: _,
            type_,
        } => {
            if let Type::Slice(slice) = type_.as_ref() {
                if let Type::ResolvedPath(type_path) = slice.as_ref() {
                    let generic_args = type_path.args.clone().unwrap();
                    return are_generic_args_fuzzable(&generic_args, filter);
                }
            }
        }
        // Matches Vec<u8> and String
        Type::ResolvedPath(type_path) => {
            let generic_args = type_path.args.clone().unwrap();
            return are_generic_args_fuzzable(&generic_args, filter);
        }
        Type::Primitive(s) => match s.as_str() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16"
            | "i32" | "i64" | "i128" => return true,
            _ => {}
        },
        _ => {}
    }
    false
}

fn are_generic_args_fuzzable(
    args: &GenericArgs,
    filter: ParamTypeFilter,
) -> bool {
    match args {
        GenericArgs::AngleBracketed {
            args,
            constraints: _,
        } => are_any_generic_args_fuzzable(args, filter),
        GenericArgs::Parenthesized { inputs, output: _ } => {
            are_any_types_fuzzable(inputs, filter)
        }
        _ => false,
    }
}

fn are_any_types_fuzzable(xs: &[Type], filter: ParamTypeFilter) -> bool {
    xs.iter().any(|ty| is_fuzzable_type(ty, filter))
}

fn are_any_generic_args_fuzzable(
    xs: &[GenericArg],
    filter: ParamTypeFilter,
) -> bool {
    xs.iter().any(|arg| is_generic_arg_fuzzable(arg, filter))
}

fn is_generic_arg_fuzzable(arg: &GenericArg, filter: ParamTypeFilter) -> bool {
    match arg {
        GenericArg::Type(ty) => is_fuzzable_type(ty, filter),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fuzzable_type_primitives() {
        assert!(is_fuzzable_type(
            &Type::Primitive("u8".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u16".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u32".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u64".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u128".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("usize".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i8".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i16".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i32".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i64".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i128".to_string()),
            ParamTypeFilter::Any
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u8".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u16".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u32".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u64".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u128".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("usize".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i8".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i16".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i32".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i64".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i128".to_string()),
            ParamTypeFilter::BinaryOnly
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u8".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u16".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u32".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u64".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("u128".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("usize".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i8".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i16".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i32".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i64".to_string()),
            ParamTypeFilter::Arbitrary
        ));
        assert!(is_fuzzable_type(
            &Type::Primitive("i128".to_string()),
            ParamTypeFilter::Arbitrary
        ));
    }
}
