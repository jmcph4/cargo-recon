use std::{
    fmt,
    path::{Path, PathBuf},
};

use rustdoc_types::Item;
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
        match self {
            Self::BinaryOnly => false,
            _ => true,
        }
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

#[derive(Clone, Default)]
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
    let root = build_rustdoc(path)?;
    let mut candidates = functions(&root);

    if let Some(f) = filter {
        if let Some(vis) = f.visibility {
            candidates = functions_with_visibility(&root, vis);
        }
    }

    Ok(candidates.iter().cloned().map(Target::from).collect())
}

//fn is_fuzzable_type(ty: &Type, filter: ParamTypeFilter) -> bool {
//    match ty {
//        // Matches a reference type like &[u8]
//        Type::Reference(ref_type) => {
//            if let syn::Type::Slice(slice) = &*ref_type.elem {
//                if let syn::Type::Path(type_path) = &*slice.elem {
//                    return type_path.path.is_ident("u8")
//                        || type_path.path.is_ident("u16")
//                        || type_path.path.is_ident("u32")
//                        || type_path.path.is_ident("u64")
//                        || type_path.path.is_ident("u128")
//                        || type_path.path.is_ident("usize");
//                }
//            }
//        }
//        // Matches Vec<u8> and String
//        Type::Path(type_path) => {
//            if let Some(segment) = type_path.path.segments.first() {
//                if segment.ident == "Vec" {
//                    if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                        &segment.arguments
//                    {
//                        if let Some(syn::GenericArgument::Type(
//                            syn::Type::Path(inner_type_path),
//                        )) = angle_bracketed.args.first()
//                        {
//                            if inner_type_path.path.is_ident("u8")
//                                || inner_type_path.path.is_ident("u16")
//                                || inner_type_path.path.is_ident("u32")
//                                || inner_type_path.path.is_ident("u64")
//                                || inner_type_path.path.is_ident("u128")
//                                || inner_type_path.path.is_ident("usize")
//                            {
//                                return true;
//                            }
//                        }
//                    }
//                }
//                if segment.ident == "String" && filter.strings_allowed() {
//                    return true;
//                }
//            }
//        }
//        _ => {}
//    }
//    false
//}
