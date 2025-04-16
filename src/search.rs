use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use syn::{ItemFn, Type, visit::Visit};

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
    pub visibility: Option<syn::Visibility>,
    pub param_type: ParamTypeFilter,
    pub param_coverage: ParamCoverageFilter,
}

#[derive(Clone, Debug)]
pub struct Target {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
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

pub struct FunctionFinder {
    root: PathBuf,
    results: Vec<Target>,
    filter: Filter,
}

impl FunctionFinder {
    pub fn new(root: PathBuf, filter: Option<Filter>) -> Self {
        Self {
            root,
            results: Vec::new(),
            filter: filter.unwrap_or_default(),
        }
    }
}

impl<'ast> Visit<'ast> for FunctionFinder {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if let Some(visibility) = &self.filter.visibility {
            if !matches!(&i.vis, visibility) {
                return;
            }
        }

        for input in &i.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                let ty = &*pat_type.ty;
                if is_fuzzable_type(ty, self.filter.param_type) {
                    let start = i.sig.ident.span().start();
                    self.results.push(Target {
                        name: i.sig.ident.to_string(),
                        file_path: self.root.clone(),
                        line: start.line,
                    });
                }
            }
        }
        syn::visit::visit_item_fn(self, i);
    }
}

fn is_fuzzable_type(ty: &Type, filter: ParamTypeFilter) -> bool {
    match ty {
        // Matches a reference type like &[u8]
        Type::Reference(ref_type) => {
            if let syn::Type::Slice(slice) = &*ref_type.elem {
                if let syn::Type::Path(type_path) = &*slice.elem {
                    return type_path.path.is_ident("u8")
                        || type_path.path.is_ident("u16")
                        || type_path.path.is_ident("u32")
                        || type_path.path.is_ident("u64")
                        || type_path.path.is_ident("u128")
                        || type_path.path.is_ident("usize");
                }
            }
        }
        // Matches Vec<u8> and String
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                if segment.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(angle_bracketed) =
                        &segment.arguments
                    {
                        if let Some(syn::GenericArgument::Type(
                            syn::Type::Path(inner_type_path),
                        )) = angle_bracketed.args.first()
                        {
                            if inner_type_path.path.is_ident("u8")
                                || inner_type_path.path.is_ident("u16")
                                || inner_type_path.path.is_ident("u32")
                                || inner_type_path.path.is_ident("u64")
                                || inner_type_path.path.is_ident("u128")
                                || inner_type_path.path.is_ident("usize")
                            {
                                return true;
                            }
                        }
                    }
                }
                if segment.ident == "String" && filter.strings_allowed() {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

pub fn search_file<P>(
    path: P,
    filter: Option<Filter>,
) -> eyre::Result<Vec<Target>>
where
    P: AsRef<Path>,
{
    let syntax = syn::parse_file(&fs::read_to_string(&path)?)?;
    let mut finder = FunctionFinder::new(path.as_ref().to_path_buf(), filter);
    finder.visit_file(&syntax);
    Ok(finder.results)
}
