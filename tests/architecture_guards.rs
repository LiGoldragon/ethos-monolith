use std::{
    fs,
    path::{Path, PathBuf},
};

use syn::{Attribute, Fields, Item, ItemImpl, ItemMod, Meta, Type, TypePath};
use syn::{Token, parse::Parser, punctuated::Punctuated};

const FORBIDDEN_TERMS: [&str; 6] = ["transcode", "archive", "decode", "codec", "encode", "code"];

struct SourceUnit {
    module: Vec<String>,
    source: String,
    syntax: syn::File,
}

#[derive(Debug)]
struct ZstType {
    module: Vec<String>,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
enum ViolationKind {
    FreeFunction,
    InherentImpl,
    ZstBehavior,
}

#[derive(Debug)]
struct Violation {
    kind: ViolationKind,
}

fn identifier_name(identifier: &syn::Ident) -> String {
    let value = identifier.to_string();
    value
        .strip_prefix("r#")
        .unwrap_or(value.as_str())
        .to_owned()
}

fn source_module(root: &Path, path: &Path) -> Vec<String> {
    let relative = path
        .strip_prefix(root)
        .expect("source path is under scan root");
    let mut components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let file = components.pop().expect("source has a file name");
    let stem = file.strip_suffix(".rs").expect("source is Rust");
    if !matches!(stem, "lib" | "main" | "mod") {
        components.push(stem.to_owned());
    }
    components
}

fn load_sources(root: &Path) -> Vec<SourceUnit> {
    let mut paths = Vec::new();
    collect_paths(root, &mut paths);
    paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("read Rust source");
            let syntax = syn::parse_file(&source).unwrap_or_else(|error| {
                panic!("{} is not parseable Rust: {error}", path.display())
            });
            SourceUnit {
                module: source_module(root, &path),
                source,
                syntax,
            }
        })
        .collect()
}

fn collect_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read source directory") {
        let path = entry.expect("read directory entry").path();
        if path.is_dir() {
            collect_paths(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
    paths.sort();
}

fn load_one(path: &Path) -> SourceUnit {
    let source = fs::read_to_string(path).expect("read fixture");
    let syntax = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("{} is not parseable Rust: {error}", path.display()));
    SourceUnit {
        module: Vec::new(),
        source,
        syntax,
    }
}

fn cfg_test_only(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        let Meta::List(list) = &attribute.meta else {
            return false;
        };
        let nested = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .unwrap_or_default();
        meta_is_test_only(list.path.is_ident("all"), &nested)
    })
}

fn meta_is_test_only(is_all: bool, nested: &Punctuated<Meta, Token![,]>) -> bool {
    if !is_all {
        return nested.len() == 1
            && matches!(nested.first(), Some(Meta::Path(path)) if path.is_ident("test"));
    }
    nested.iter().any(|meta| {
        matches!(meta, Meta::Path(path) if path.is_ident("test"))
            || matches!(meta, Meta::List(list) if list.path.is_ident("all") && {
                Punctuated::<Meta, Token![,]>::parse_terminated
                    .parse2(list.tokens.clone())
                    .map(|nested| meta_is_test_only(true, &nested))
                    .unwrap_or(false)
            })
    })
}

fn test_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("test"))
}

fn is_zst_struct(item: &syn::ItemStruct) -> bool {
    match &item.fields {
        Fields::Unit => true,
        Fields::Named(fields) => fields.named.is_empty(),
        Fields::Unnamed(fields) => fields.unnamed.is_empty(),
    }
}

fn collect_zsts(items: &[Item], module: &[String], zsts: &mut Vec<ZstType>) {
    for item in items {
        match item {
            Item::Struct(item) if is_zst_struct(item) => zsts.push(ZstType {
                module: module.to_owned(),
                name: identifier_name(&item.ident),
            }),
            Item::Mod(ItemMod {
                ident,
                content: Some((_, items)),
                ..
            }) => {
                let mut nested = module.to_owned();
                nested.push(identifier_name(ident));
                collect_zsts(items, &nested, zsts);
            }
            _ => {}
        }
    }
}

fn path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| identifier_name(&segment.ident))
        .collect()
}

fn candidate_modules(module: &[String], segments: &[String]) -> Vec<Vec<String>> {
    if segments.is_empty() {
        return Vec::new();
    }
    if segments[0] == "crate" {
        return vec![segments[1..].to_vec()];
    }
    if segments[0] == "self" {
        let mut candidate = module.to_owned();
        candidate.extend_from_slice(&segments[1..]);
        return vec![candidate];
    }
    if segments[0] == "super" {
        let mut parent = module.to_owned();
        let mut index = 0;
        while index < segments.len() && segments[index] == "super" {
            parent.pop();
            index += 1;
        }
        parent.extend_from_slice(&segments[index..]);
        return vec![parent];
    }

    (0..=module.len())
        .rev()
        .map(|prefix| {
            let mut candidate = module[..prefix].to_vec();
            candidate.extend_from_slice(segments);
            candidate
        })
        .collect()
}

fn resolves_zst(path: &syn::Path, module: &[String], zsts: &[ZstType]) -> bool {
    let segments = path_segments(path);
    candidate_modules(module, &segments)
        .into_iter()
        .any(|candidate| {
            zsts.iter().any(|zst| {
                zst.module == candidate[..candidate.len().saturating_sub(1)]
                    && zst.name == candidate.last().map(String::as_str).unwrap_or_default()
            })
        })
}

fn type_is_zst(ty: &Type, module: &[String], zsts: &[ZstType]) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => resolves_zst(path, module, zsts),
        Type::Paren(paren) => type_is_zst(&paren.elem, module, zsts),
        Type::Reference(reference) => type_is_zst(&reference.elem, module, zsts),
        Type::Tuple(tuple) => tuple.elems.is_empty(),
        _ => false,
    }
}

fn scan_items(
    items: &[Item],
    module: &[String],
    zsts: &[ZstType],
    violations: &mut Vec<Violation>,
) {
    for item in items {
        match item {
            Item::Fn(function)
                if !(test_attribute(&function.attrs)
                    || cfg_test_only(&function.attrs)
                    || module.is_empty() && identifier_name(&function.sig.ident) == "main") =>
            {
                violations.push(Violation {
                    kind: ViolationKind::FreeFunction,
                });
            }
            Item::ForeignMod(foreign) => {
                for item in &foreign.items {
                    if matches!(item, syn::ForeignItem::Fn(_)) {
                        violations.push(Violation {
                            kind: ViolationKind::FreeFunction,
                        });
                    }
                }
            }
            Item::Impl(ItemImpl {
                trait_, self_ty, ..
            }) => {
                if trait_.is_none() {
                    violations.push(Violation {
                        kind: ViolationKind::InherentImpl,
                    });
                } else if type_is_zst(self_ty, module, zsts) {
                    violations.push(Violation {
                        kind: ViolationKind::ZstBehavior,
                    });
                }
            }
            Item::Mod(item) if !cfg_test_only(&item.attrs) => {
                if let Some((_, nested)) = &item.content {
                    let mut nested_module = module.to_owned();
                    nested_module.push(identifier_name(&item.ident));
                    scan_items(nested, &nested_module, zsts, violations);
                }
            }
            _ => {}
        }
    }
}

fn structural_violations(sources: &[SourceUnit]) -> Vec<Violation> {
    let mut zsts = Vec::new();
    for source in sources {
        collect_zsts(&source.syntax.items, &source.module, &mut zsts);
    }
    let mut violations = Vec::new();
    for source in sources {
        scan_items(&source.syntax.items, &source.module, &zsts, &mut violations);
    }
    violations
}

fn xid_continue(character: char) -> bool {
    character == '_' || unicode_ident::is_xid_continue(character)
}

fn vocabulary_violations(source: &str) -> usize {
    let lowercase = source.to_ascii_lowercase();
    let mut matches = Vec::new();
    for term in FORBIDDEN_TERMS {
        let mut offset = 0;
        while let Some(relative) = lowercase[offset..].find(term) {
            let start = offset + relative;
            matches.push((start, start + term.len()));
            offset = start + term.len();
        }
    }
    matches.sort_by_key(|(start, end)| (*start, std::cmp::Reverse(*end)));
    let mut accepted = Vec::new();
    for (start, end) in matches {
        if accepted
            .iter()
            .any(|(other_start, other_end)| start < *other_end && end > *other_start)
        {
            continue;
        }
        let before = source[..start].chars().next_back();
        let after = source[end..].chars().next();
        if before.is_some_and(xid_continue) || after.is_some_and(xid_continue) {
            continue;
        }
        accepted.push((start, end));
    }
    accepted.len()
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("checks/fixtures/architecture-guards")
}

fn assert_no_structural_violations(path: &Path) {
    let sources = vec![load_one(path)];
    assert!(
        structural_violations(&sources).is_empty(),
        "good fixture {} was rejected",
        path.display()
    );
}

fn assert_has_kind(path: &Path, kind: ViolationKind) {
    let sources = vec![load_one(path)];
    assert!(
        structural_violations(&sources)
            .iter()
            .any(|violation| violation.kind == kind),
        "bad fixture {} did not expose {kind:?}",
        path.display()
    );
}

#[test]
fn architecture_guard_fixtures_are_falsifiable() {
    let root = fixture_root();
    assert_no_structural_violations(&root.join("free-functions-good.rs"));
    assert_has_kind(
        &root.join("free-functions-bad.rs"),
        ViolationKind::FreeFunction,
    );
    assert_no_structural_violations(&root.join("inherent-methods-good.rs"));
    assert_has_kind(
        &root.join("inherent-methods-bad.rs"),
        ViolationKind::InherentImpl,
    );
    assert_no_structural_violations(&root.join("zst-behavior-good.rs"));
    let zst_bad = vec![
        load_one(&root.join("zst-behavior-bad.rs")),
        load_one(&root.join("zst-cross-file-decl.rs")),
        load_one(&root.join("zst-cross-file-impl.rs")),
    ];
    assert!(
        structural_violations(&zst_bad)
            .iter()
            .filter(|violation| violation.kind == ViolationKind::ZstBehavior)
            .count()
            >= 4,
        "ZST bad fixtures did not expose all namespace/tuple witnesses"
    );
    assert_eq!(
        vocabulary_violations(&fs::read_to_string(root.join("vocabulary-good.rs")).unwrap()),
        0
    );
    assert!(
        vocabulary_violations(&fs::read_to_string(root.join("vocabulary-bad.rs")).unwrap()) >= 7,
        "vocabulary bad fixture did not expose all witnesses"
    );
}

#[test]
fn production_src_obeys_architecture_binding_law() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let sources = load_sources(&root);
    let violations = structural_violations(&sources);
    assert!(
        violations.is_empty(),
        "production structural violations: {violations:?}"
    );
    let vocabulary = sources
        .iter()
        .map(|source| vocabulary_violations(&source.source))
        .sum::<usize>();
    assert_eq!(
        vocabulary, 0,
        "production forbidden vocabulary matches: {vocabulary}"
    );
}
