use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

use syn::{
    Attribute, Expr, ExprLit, Fields, GenericArgument, Item, ItemImpl, Lit, Meta, PathArguments,
    Token, Type, TypePath, UseTree, parse::Parser, punctuated::Punctuated,
};

const FORBIDDEN_TERMS: [&str; 6] = ["transcode", "archive", "decode", "codec", "encode", "code"];
const ALLOWED_DERIVES: [&str; 7] = [
    "Clone",
    "Copy",
    "Debug",
    "Eq",
    "PartialEq",
    "Default",
    "thiserror::Error",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TypeKey {
    module: Vec<String>,
    name: String,
}

#[derive(Clone, Debug)]
struct PathRef {
    segments: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum GenericParameterKind {
    Type,
    Const,
    Lifetime,
}

#[derive(Clone, Debug)]
struct GenericParameter {
    name: String,
    kind: GenericParameterKind,
}

#[derive(Clone)]
enum GenericArgumentValue {
    Type(Type),
    Const(Expr),
    Lifetime(syn::Lifetime),
}

#[derive(Clone, Default)]
struct GenericSubstitutions {
    types: BTreeMap<String, Type>,
    consts: BTreeMap<String, Expr>,
    lifetimes: BTreeMap<String, syn::Lifetime>,
}

#[derive(Clone)]
struct TypeRef {
    path: PathRef,
    arguments: Vec<GenericArgumentValue>,
    unsupported_arguments: bool,
}

#[derive(Clone)]
struct GenericLayout {
    parameters: Vec<GenericParameter>,
    fields: Vec<Type>,
}

#[derive(Clone)]
struct GenericAlias {
    parameters: Vec<GenericParameter>,
    target: Type,
}

struct RelativePath {
    bases: Vec<Vec<String>>,
    tail_start: usize,
}

struct Unit {
    module: Vec<String>,
    items: Vec<Item>,
}

#[derive(Default)]
struct Corpus {
    units: Vec<Unit>,
    sources: BTreeMap<PathBuf, String>,
    loaded_files: HashSet<PathBuf>,
    loaded_modules: HashSet<(PathBuf, Vec<String>)>,
    active_files: HashSet<PathBuf>,
}

struct Resolver<'a> {
    corpus: &'a Corpus,
    type_defs: BTreeSet<TypeKey>,
    aliases: BTreeMap<TypeKey, GenericAlias>,
    layouts: BTreeMap<TypeKey, GenericLayout>,
    bindings: BTreeMap<TypeKey, PathRef>,
    globs: BTreeMap<Vec<String>, Vec<PathRef>>,
    modules: BTreeSet<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ViolationKind {
    FreeFunction,
    InherentImpl,
    ZstBehavior,
    MacroSurface,
}

#[derive(Debug)]
struct Violation {
    kind: ViolationKind,
    detail: String,
}

fn identifier_name(identifier: &syn::Ident) -> String {
    let value = identifier.to_string();
    value
        .strip_prefix("r#")
        .unwrap_or(value.as_str())
        .to_owned()
}

fn module_path_from_file(root: &Path, file: &Path) -> Vec<String> {
    let relative = file
        .strip_prefix(root)
        .expect("source path is under scan root");
    let mut components = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if let Some(last) = components.last_mut() {
        if last == "lib.rs" || last == "main.rs" {
            components.pop();
        } else if let Some(stem) = last.strip_suffix(".rs") {
            *last = stem.to_owned();
        }
    }
    if components
        .last()
        .is_some_and(|component| component == "mod")
    {
        components.pop();
    }
    components
}

impl Corpus {
    fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut corpus = Self::default();
        for path in paths {
            corpus.add_file(path, Vec::new());
        }
        corpus
    }

    fn production(root: &Path) -> Self {
        let root = root.canonicalize().expect("canonical source root");
        let mut corpus = Self::default();
        corpus.add_file(root.join("lib.rs"), Vec::new());
        let mut all_files = Vec::new();
        collect_paths(&root, &mut all_files);
        for path in all_files {
            if !corpus.loaded_files.contains(&path) {
                corpus.add_file(path.clone(), module_path_from_file(&root, &path));
            }
        }
        corpus
    }

    fn add_file(&mut self, path: PathBuf, module: Vec<String>) {
        let path = path.canonicalize().unwrap_or(path);
        if !self.loaded_modules.insert((path.clone(), module.clone())) {
            return;
        }
        self.loaded_files.insert(path.clone());
        if !self.active_files.insert(path.clone()) {
            return;
        }
        let source =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let syntax = syn::parse_file(&source)
            .unwrap_or_else(|error| panic!("{} is not parseable Rust: {error}", path.display()));
        self.sources.insert(path.clone(), source.clone());
        self.units.push(Unit {
            module: module.clone(),
            items: syntax.items.clone(),
        });
        for (child_module, child) in module_children(&path, &module, &syntax.items) {
            match child {
                ModuleChild::Inline(items) => self.add_inline(path.clone(), child_module, items),
                ModuleChild::External(path) => self.add_file(path, child_module),
            }
        }
        self.active_files.remove(&path);
    }

    fn add_inline(&mut self, file: PathBuf, module: Vec<String>, items: Vec<Item>) {
        self.units.push(Unit {
            module: module.clone(),
            items: items.clone(),
        });
        for (child_module, child) in module_children(&file, &module, &items) {
            match child {
                ModuleChild::Inline(items) => self.add_inline(file.clone(), child_module, items),
                ModuleChild::External(path) => self.add_file(path, child_module),
            }
        }
    }
}

enum ModuleChild {
    Inline(Vec<Item>),
    External(PathBuf),
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

fn module_children(
    file: &Path,
    module: &[String],
    items: &[Item],
) -> Vec<(Vec<String>, ModuleChild)> {
    items
        .iter()
        .filter_map(|item| {
            let Item::Mod(item_mod) = item else {
                return None;
            };
            let mut child_module = module.to_vec();
            child_module.push(identifier_name(&item_mod.ident));
            if let Some((_, inline_items)) = &item_mod.content {
                return Some((child_module, ModuleChild::Inline(inline_items.clone())));
            }
            Some((
                child_module,
                ModuleChild::External(external_module_file(file, item_mod)),
            ))
        })
        .collect()
}

fn external_module_file(parent: &Path, item_mod: &syn::ItemMod) -> PathBuf {
    let parent_directory = parent.parent().expect("module parent directory");
    if let Some(path) = module_path_attribute(item_mod) {
        return parent_directory.join(path);
    }
    let directory = match parent.file_stem().and_then(|stem| stem.to_str()) {
        Some("lib") | Some("main") | Some("mod") | None => parent_directory.to_path_buf(),
        Some(stem) => parent_directory.join(stem),
    };
    let name = identifier_name(&item_mod.ident);
    let flat = directory.join(format!("{name}.rs"));
    if flat.is_file() {
        return flat;
    }
    directory.join(name).join("mod.rs")
}

fn module_path_attribute(item_mod: &syn::ItemMod) -> Option<String> {
    item_mod.attrs.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }
        let Meta::NameValue(name_value) = &attribute.meta else {
            return None;
        };
        let Expr::Lit(ExprLit {
            lit: Lit::Str(path),
            ..
        }) = &name_value.value
        else {
            return None;
        };
        Some(path.value())
    })
}

fn path_ref(path: &syn::Path) -> PathRef {
    PathRef {
        segments: path
            .segments
            .iter()
            .map(|segment| identifier_name(&segment.ident))
            .collect(),
    }
}

fn collect_use_tree(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    named: &mut Vec<(String, PathRef)>,
    globs: &mut Vec<PathRef>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(identifier_name(&path.ident));
            collect_use_tree(&path.tree, prefix, named, globs);
            prefix.pop();
        }
        UseTree::Name(name) => {
            if identifier_name(&name.ident) == "self" && !prefix.is_empty() {
                named.push((
                    prefix.last().cloned().expect("nonempty use prefix"),
                    PathRef {
                        segments: prefix.clone(),
                    },
                ));
            } else {
                let mut path = prefix.clone();
                path.push(identifier_name(&name.ident));
                named.push((identifier_name(&name.ident), PathRef { segments: path }));
            }
        }
        UseTree::Rename(rename) => {
            let path = if identifier_name(&rename.ident) == "self" {
                if prefix.is_empty() {
                    vec!["self".into()]
                } else {
                    prefix.clone()
                }
            } else {
                let mut path = prefix.clone();
                path.push(identifier_name(&rename.ident));
                path
            };
            named.push((identifier_name(&rename.rename), PathRef { segments: path }));
        }
        UseTree::Glob(_) => globs.push(PathRef {
            segments: prefix.clone(),
        }),
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree(item, prefix, named, globs);
            }
        }
    }
}

impl<'a> Resolver<'a> {
    fn new(corpus: &'a Corpus) -> Self {
        let modules = corpus
            .units
            .iter()
            .map(|unit| unit.module.clone())
            .collect();
        let mut resolver = Self {
            corpus,
            type_defs: BTreeSet::new(),
            aliases: BTreeMap::new(),
            layouts: BTreeMap::new(),
            bindings: BTreeMap::new(),
            globs: BTreeMap::new(),
            modules,
        };
        for unit in &corpus.units {
            for item in &unit.items {
                match item {
                    Item::Struct(item_struct) => {
                        let key = TypeKey {
                            module: unit.module.clone(),
                            name: identifier_name(&item_struct.ident),
                        };
                        resolver.type_defs.insert(key.clone());
                        resolver.layouts.insert(
                            key,
                            GenericLayout {
                                parameters: generic_parameters(&item_struct.generics),
                                fields: struct_field_types(item_struct),
                            },
                        );
                    }
                    Item::Type(item_type) => {
                        let key = TypeKey {
                            module: unit.module.clone(),
                            name: identifier_name(&item_type.ident),
                        };
                        resolver.type_defs.insert(key.clone());
                        resolver.aliases.insert(
                            key,
                            GenericAlias {
                                parameters: generic_parameters(&item_type.generics),
                                target: (*item_type.ty).clone(),
                            },
                        );
                    }
                    Item::Enum(item_enum) => {
                        resolver.type_defs.insert(TypeKey {
                            module: unit.module.clone(),
                            name: identifier_name(&item_enum.ident),
                        });
                    }
                    Item::Union(item_union) => {
                        resolver.type_defs.insert(TypeKey {
                            module: unit.module.clone(),
                            name: identifier_name(&item_union.ident),
                        });
                    }
                    Item::Use(item_use) => {
                        let mut named = Vec::new();
                        let mut globs = Vec::new();
                        collect_use_tree(&item_use.tree, &mut Vec::new(), &mut named, &mut globs);
                        for (name, path) in named {
                            resolver.bindings.insert(
                                TypeKey {
                                    module: unit.module.clone(),
                                    name,
                                },
                                path,
                            );
                        }
                        resolver
                            .globs
                            .entry(unit.module.clone())
                            .or_default()
                            .extend(globs);
                    }
                    _ => {}
                }
            }
        }
        resolver
    }

    fn resolve_self_type(&self, module: &[String], ty: &Type) -> bool {
        self.type_is_zst(
            module,
            ty,
            &GenericSubstitutions::default(),
            &mut BTreeSet::new(),
        )
    }

    fn type_is_zst(
        &self,
        module: &[String],
        ty: &Type,
        substitutions: &GenericSubstitutions,
        visiting: &mut BTreeSet<TypeKey>,
    ) -> bool {
        let substituted = substitute_type(ty, substitutions);
        let ty = &substituted;
        match ty {
            Type::Path(TypePath {
                qself: None, path, ..
            }) => {
                let Some(reference) = type_ref(ty) else {
                    return false;
                };
                if reference.unsupported_arguments {
                    return false;
                }
                if self.is_canonical_phantom_data(module, &reference.path) {
                    return true;
                }
                self.resolve_path(module, &path_ref(path), &mut BTreeSet::new())
                    .is_some_and(|key| self.key_is_zst(&key, &reference.arguments, visiting))
            }
            Type::Paren(paren) => self.type_is_zst(module, &paren.elem, substitutions, visiting),
            Type::Group(group) => self.type_is_zst(module, &group.elem, substitutions, visiting),
            Type::Tuple(tuple) => tuple
                .elems
                .iter()
                .all(|field| self.type_is_zst(module, field, substitutions, visiting)),
            Type::Array(array) => {
                array_is_zero(&array.len, substitutions)
                    || self.type_is_zst(module, &array.elem, substitutions, visiting)
            }
            Type::Reference(_) | Type::Ptr(_) => false,
            _ => false,
        }
    }

    fn key_is_zst(
        &self,
        key: &TypeKey,
        arguments: &[GenericArgumentValue],
        visiting: &mut BTreeSet<TypeKey>,
    ) -> bool {
        if !visiting.insert(key.clone()) {
            return false;
        }
        let result = if let Some(layout) = self.layouts.get(key) {
            let Some(substitutions) = generic_substitutions(&layout.parameters, arguments) else {
                visiting.remove(key);
                return false;
            };
            layout
                .fields
                .iter()
                .all(|field| self.type_is_zst(&key.module, field, &substitutions, visiting))
        } else if let Some(alias) = self.aliases.get(key) {
            let Some(substitutions) = generic_substitutions(&alias.parameters, arguments) else {
                visiting.remove(key);
                return false;
            };
            let target = substitute_type(&alias.target, &substitutions);
            self.type_is_zst(&key.module, &target, &substitutions, visiting)
        } else {
            false
        };
        visiting.remove(key);
        result
    }

    fn is_canonical_phantom_data(&self, module: &[String], path: &PathRef) -> bool {
        self.canonical_external_path(module, path, &mut BTreeSet::new())
            .is_some_and(|resolved| {
                resolved == ["std", "marker", "PhantomData"]
                    || resolved == ["core", "marker", "PhantomData"]
            })
    }

    fn canonical_external_path(
        &self,
        module: &[String],
        path: &PathRef,
        visiting: &mut BTreeSet<(Vec<String>, Vec<String>)>,
    ) -> Option<Vec<String>> {
        if !visiting.insert((module.to_vec(), path.segments.clone())) {
            return None;
        }
        if path.segments.starts_with(&["std".into(), "marker".into()])
            || path.segments.starts_with(&["core".into(), "marker".into()])
        {
            return Some(path.segments.clone());
        }
        let relative = self.relative_module_bases(module, path)?;
        let tail = &path.segments[relative.tail_start..];
        let name = tail.last()?.clone();
        if tail.len() > 1 {
            let prefix = PathRef {
                segments: path.segments[..path.segments.len() - 1].to_vec(),
            };
            if let Some(mut resolved) = self.canonical_external_path(module, &prefix, visiting) {
                resolved.push(name);
                return Some(resolved);
            }
            if let Some(target_module) = self.resolve_module_path(module, &prefix)
                && let Some(resolved) = self.canonical_external_path(
                    &target_module,
                    &PathRef {
                        segments: vec![name],
                    },
                    visiting,
                )
            {
                return Some(resolved);
            }
            return None;
        }
        for base in relative.bases {
            let key = TypeKey {
                module: base.clone(),
                name: name.clone(),
            };
            if let Some(alias) = self.aliases.get(&key)
                && let Some(target) = type_ref(&alias.target)
                && !target.unsupported_arguments
                && let Some(resolved) = self.canonical_external_path(&base, &target.path, visiting)
            {
                return Some(resolved);
            }
            if self.layouts.contains_key(&key) {
                continue;
            }
            if let Some(target) = self.bindings.get(&key)
                && let Some(resolved) = self.canonical_external_path(&base, target, visiting)
            {
                return Some(resolved);
            }
            for glob in self.globs.get(&base).cloned().unwrap_or_default() {
                if let Some(mut resolved) = self.canonical_external_path(&base, &glob, visiting) {
                    resolved.push(name.clone());
                    return Some(resolved);
                }
                let Some(target_module) = self.resolve_module_path(&base, &glob) else {
                    continue;
                };
                if !self.public_names(&target_module, module).contains(&name) {
                    continue;
                }
                if let Some(resolved) = self.canonical_external_path(
                    &target_module,
                    &PathRef {
                        segments: vec![name.clone()],
                    },
                    visiting,
                ) {
                    return Some(resolved);
                }
            }
        }
        None
    }

    fn resolve_path(
        &self,
        module: &[String],
        path: &PathRef,
        visiting: &mut BTreeSet<TypeKey>,
    ) -> Option<TypeKey> {
        let relative = self.relative_module_bases(module, path)?;
        let tail = &path.segments[relative.tail_start..];
        let name = tail.last()?.clone();
        if tail.len() > 1 {
            let target_module = self.resolve_module_path(
                module,
                &PathRef {
                    segments: path.segments[..path.segments.len() - 1].to_vec(),
                },
            )?;
            return self.resolve_binding(&target_module, &name, visiting);
        }
        for base in relative.bases {
            if let Some(result) = self.resolve_binding(&base, &name, visiting) {
                return Some(result);
            }
        }
        None
    }

    fn resolve_binding(
        &self,
        module: &[String],
        name: &str,
        visiting: &mut BTreeSet<TypeKey>,
    ) -> Option<TypeKey> {
        let key = TypeKey {
            module: module.to_vec(),
            name: name.to_owned(),
        };
        if self.type_defs.contains(&key) {
            return Some(key);
        }
        if !visiting.insert(key.clone()) {
            return None;
        }
        if let Some(target) = self.bindings.get(&key).cloned() {
            let result = self.resolve_path(module, &target, visiting);
            visiting.remove(&key);
            return result;
        }
        if self.local_item_names(module).contains(name) {
            visiting.remove(&key);
            return None;
        }
        for glob in self.globs.get(module).cloned().unwrap_or_default() {
            let Some(target_module) = self.resolve_module_path(module, &glob) else {
                continue;
            };
            if !self.public_names(&target_module, module).contains(name) {
                continue;
            }
            if let Some(result) = self.resolve_binding(&target_module, name, visiting) {
                visiting.remove(&key);
                return Some(result);
            }
        }
        visiting.remove(&key);
        None
    }

    fn resolve_module_path(&self, module: &[String], path: &PathRef) -> Option<Vec<String>> {
        self.resolve_module_path_inner(module, path, &mut BTreeSet::new())
    }

    fn resolve_module_path_inner(
        &self,
        module: &[String],
        path: &PathRef,
        visiting: &mut BTreeSet<TypeKey>,
    ) -> Option<Vec<String>> {
        let relative = self.relative_module_bases(module, path)?;
        let tail = &path.segments[relative.tail_start..];
        for base in relative.bases {
            let mut candidate = base;
            let mut resolved = true;
            for segment in tail {
                let mut direct = candidate.clone();
                direct.push(segment.clone());
                if self.modules.contains(&direct) {
                    candidate = direct;
                    continue;
                }
                let key = TypeKey {
                    module: candidate.clone(),
                    name: segment.clone(),
                };
                if let Some(alias) = self.bindings.get(&key).cloned() {
                    if !visiting.insert(key.clone()) {
                        resolved = false;
                        break;
                    }
                    let Some(aliased) =
                        self.resolve_module_path_inner(&candidate, &alias, visiting)
                    else {
                        visiting.remove(&key);
                        resolved = false;
                        break;
                    };
                    visiting.remove(&key);
                    candidate = aliased;
                    continue;
                }
                if let Some(glob_module) = self.resolve_glob_module_component(&candidate, segment) {
                    candidate = glob_module;
                    continue;
                }
                resolved = false;
                break;
            }
            if resolved {
                return Some(candidate);
            }
        }
        None
    }

    fn resolve_glob_module_component(
        &self,
        importer: &[String],
        name: &str,
    ) -> Option<Vec<String>> {
        for glob in self.globs.get(importer).cloned().unwrap_or_default() {
            let Some(target_module) = self.resolve_module_path(importer, &glob) else {
                continue;
            };
            if let Some(module) = self
                .exported_modules(&target_module, importer, &mut BTreeSet::new())
                .get(name)
            {
                return Some(module.clone());
            }
        }
        None
    }

    fn relative_module_bases(&self, module: &[String], path: &PathRef) -> Option<RelativePath> {
        let first = path.segments.first()?.as_str();
        let (bases, tail_start) = match first {
            "crate" => (vec![Vec::new()], 1),
            "self" => (vec![module.to_vec()], 1),
            "super" => {
                let mut base = module.to_vec();
                let mut count = 0;
                while path
                    .segments
                    .get(count)
                    .is_some_and(|segment| segment == "super")
                {
                    base.pop();
                    count += 1;
                }
                (vec![base], count)
            }
            _ => (
                (0..=module.len())
                    .rev()
                    .map(|prefix| module[..prefix].to_vec())
                    .collect(),
                0,
            ),
        };
        Some(RelativePath { bases, tail_start })
    }

    fn public_names(&self, module: &[String], importer: &[String]) -> BTreeSet<String> {
        self.exported_names(module, importer, &mut BTreeSet::new())
    }

    fn exported_names(
        &self,
        module: &[String],
        importer: &[String],
        visiting: &mut BTreeSet<Vec<String>>,
    ) -> BTreeSet<String> {
        if !visiting.insert(module.to_vec()) {
            return BTreeSet::new();
        }
        let mut names = BTreeSet::new();
        for unit in self
            .corpus
            .units
            .iter()
            .filter(|unit| unit.module == module)
        {
            for item in &unit.items {
                match item {
                    Item::Struct(item) if is_visible_from(&item.vis, module, importer) => {
                        names.insert(identifier_name(&item.ident));
                    }
                    Item::Type(item) if is_visible_from(&item.vis, module, importer) => {
                        names.insert(identifier_name(&item.ident));
                    }
                    Item::Enum(item) if is_visible_from(&item.vis, module, importer) => {
                        names.insert(identifier_name(&item.ident));
                    }
                    Item::Union(item) if is_visible_from(&item.vis, module, importer) => {
                        names.insert(identifier_name(&item.ident));
                    }
                    Item::Mod(item) if is_visible_from(&item.vis, module, importer) => {
                        names.insert(identifier_name(&item.ident));
                    }
                    Item::Use(item) if is_visible_from(&item.vis, module, importer) => {
                        let mut named = Vec::new();
                        let mut globs = Vec::new();
                        collect_use_tree(&item.tree, &mut Vec::new(), &mut named, &mut globs);
                        names.extend(named.into_iter().map(|(name, _)| name));
                        for glob in globs {
                            if let Some(target) = self.resolve_module_path(module, &glob) {
                                names.extend(self.exported_names(&target, importer, visiting));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        visiting.remove(module);
        names
    }

    fn exported_modules(
        &self,
        module: &[String],
        importer: &[String],
        visiting: &mut BTreeSet<Vec<String>>,
    ) -> BTreeMap<String, Vec<String>> {
        if !visiting.insert(module.to_vec()) {
            return BTreeMap::new();
        }
        let mut modules = BTreeMap::new();
        for unit in self
            .corpus
            .units
            .iter()
            .filter(|unit| unit.module == module)
        {
            for item in &unit.items {
                match item {
                    Item::Mod(item) if is_visible_from(&item.vis, module, importer) => {
                        let mut child = module.to_vec();
                        child.push(identifier_name(&item.ident));
                        if self.modules.contains(&child) {
                            modules.insert(identifier_name(&item.ident), child);
                        }
                    }
                    Item::Use(item) if is_visible_from(&item.vis, module, importer) => {
                        let mut named = Vec::new();
                        let mut globs = Vec::new();
                        collect_use_tree(&item.tree, &mut Vec::new(), &mut named, &mut globs);
                        for (name, path) in named {
                            if let Some(target) = self.resolve_module_path(module, &path) {
                                modules.insert(name, target);
                            }
                        }
                        for glob in globs {
                            if let Some(target) = self.resolve_module_path(module, &glob) {
                                modules.extend(self.exported_modules(&target, importer, visiting));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        visiting.remove(module);
        modules
    }

    fn local_item_names(&self, module: &[String]) -> BTreeSet<String> {
        self.corpus
            .units
            .iter()
            .filter(|unit| unit.module == module)
            .flat_map(|unit| {
                unit.items.iter().filter_map(|item| match item {
                    Item::Enum(item) => Some(identifier_name(&item.ident)),
                    Item::Struct(item) => Some(identifier_name(&item.ident)),
                    Item::Trait(item) => Some(identifier_name(&item.ident)),
                    Item::TraitAlias(item) => Some(identifier_name(&item.ident)),
                    Item::Type(item) => Some(identifier_name(&item.ident)),
                    Item::Union(item) => Some(identifier_name(&item.ident)),
                    Item::Mod(item) => Some(identifier_name(&item.ident)),
                    Item::Use(item) => {
                        let mut named = Vec::new();
                        let mut globs = Vec::new();
                        collect_use_tree(&item.tree, &mut Vec::new(), &mut named, &mut globs);
                        named.into_iter().map(|(name, _)| name).next()
                    }
                    _ => None,
                })
            })
            .collect()
    }
}

fn struct_field_types(item: &syn::ItemStruct) -> Vec<Type> {
    match &item.fields {
        Fields::Named(fields) => fields.named.iter().map(|field| field.ty.clone()).collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .map(|field| field.ty.clone())
            .collect(),
        Fields::Unit => Vec::new(),
    }
}

fn generic_parameters(generics: &syn::Generics) -> Vec<GenericParameter> {
    generics
        .params
        .iter()
        .map(|parameter| match parameter {
            syn::GenericParam::Type(parameter) => GenericParameter {
                name: identifier_name(&parameter.ident),
                kind: GenericParameterKind::Type,
            },
            syn::GenericParam::Const(parameter) => GenericParameter {
                name: identifier_name(&parameter.ident),
                kind: GenericParameterKind::Const,
            },
            syn::GenericParam::Lifetime(parameter) => GenericParameter {
                name: parameter.lifetime.ident.to_string(),
                kind: GenericParameterKind::Lifetime,
            },
        })
        .collect()
}

fn generic_substitutions(
    parameters: &[GenericParameter],
    arguments: &[GenericArgumentValue],
) -> Option<GenericSubstitutions> {
    if parameters.len() != arguments.len() {
        return None;
    }
    let mut substitutions = GenericSubstitutions::default();
    for (parameter, argument) in parameters.iter().zip(arguments) {
        match (parameter.kind, argument) {
            (GenericParameterKind::Type, GenericArgumentValue::Type(argument)) => {
                substitutions.types.insert(
                    parameter.name.clone(),
                    substitute_type(argument, &GenericSubstitutions::default()),
                );
            }
            (GenericParameterKind::Const, GenericArgumentValue::Const(argument)) => {
                substitutions
                    .consts
                    .insert(parameter.name.clone(), argument.clone());
            }
            (GenericParameterKind::Lifetime, GenericArgumentValue::Lifetime(argument)) => {
                substitutions
                    .lifetimes
                    .insert(parameter.name.clone(), argument.clone());
            }
            _ => return None,
        }
    }
    Some(substitutions)
}

fn substitute_type(ty: &Type, substitutions: &GenericSubstitutions) -> Type {
    substitute_type_inner(ty, substitutions, &mut BTreeSet::new())
}

fn substitute_type_inner(
    ty: &Type,
    substitutions: &GenericSubstitutions,
    seen: &mut BTreeSet<String>,
) -> Type {
    if let Type::Path(TypePath {
        qself: None, path, ..
    }) = ty
        && path.segments.len() == 1
        && matches!(path.segments[0].arguments, PathArguments::None)
    {
        let name = identifier_name(&path.segments[0].ident);
        if let Some(replacement) = substitutions.types.get(&name) {
            if !seen.insert(name.clone()) {
                return ty.clone();
            }
            let substituted = substitute_type_inner(replacement, substitutions, seen);
            seen.remove(&name);
            return substituted;
        }
    }

    let mut substituted = ty.clone();
    match &mut substituted {
        Type::Path(type_path) => {
            for segment in &mut type_path.path.segments {
                match &mut segment.arguments {
                    PathArguments::AngleBracketed(arguments) => {
                        for argument in &mut arguments.args {
                            match argument {
                                GenericArgument::Type(argument) => {
                                    *argument =
                                        substitute_type_inner(argument, substitutions, seen);
                                }
                                GenericArgument::Const(argument) => {
                                    *argument = substitute_expr(argument, substitutions);
                                }
                                GenericArgument::Lifetime(_) => {}
                                _ => {}
                            }
                        }
                    }
                    PathArguments::Parenthesized(arguments) => {
                        for argument in &mut arguments.inputs {
                            argument.ty = substitute_type_inner(&argument.ty, substitutions, seen);
                        }
                        if let syn::ReturnType::Type(_, output) = &mut arguments.output {
                            **output = substitute_type_inner(output, substitutions, seen);
                        }
                    }
                    PathArguments::None => {}
                }
            }
        }
        Type::Array(array) => {
            *array.elem = substitute_type_inner(&array.elem, substitutions, seen);
            array.len = substitute_expr(&array.len, substitutions);
        }
        Type::Group(group) => {
            *group.elem = substitute_type_inner(&group.elem, substitutions, seen);
        }
        Type::Paren(paren) => {
            *paren.elem = substitute_type_inner(&paren.elem, substitutions, seen);
        }
        Type::Ptr(pointer) => {
            *pointer.elem = substitute_type_inner(&pointer.elem, substitutions, seen);
        }
        Type::Reference(reference) => {
            *reference.elem = substitute_type_inner(&reference.elem, substitutions, seen);
        }
        Type::Slice(slice) => {
            *slice.elem = substitute_type_inner(&slice.elem, substitutions, seen);
        }
        Type::Tuple(tuple) => {
            for element in &mut tuple.elems {
                *element = substitute_type_inner(element, substitutions, seen);
            }
        }
        _ => {}
    }
    substituted
}

fn substitute_expr(expr: &Expr, substitutions: &GenericSubstitutions) -> Expr {
    substitute_expr_inner(expr, substitutions, &mut BTreeSet::new())
}

fn substitute_expr_inner(
    expr: &Expr,
    substitutions: &GenericSubstitutions,
    seen: &mut BTreeSet<String>,
) -> Expr {
    if let Expr::Path(path) = expr
        && path.qself.is_none()
        && path.path.segments.len() == 1
        && matches!(path.path.segments[0].arguments, PathArguments::None)
    {
        let name = identifier_name(&path.path.segments[0].ident);
        if let Some(replacement) = substitutions.consts.get(&name) {
            if !seen.insert(name.clone()) {
                return expr.clone();
            }
            let substituted = substitute_expr_inner(replacement, substitutions, seen);
            seen.remove(&name);
            return substituted;
        }
    }

    let mut substituted = expr.clone();
    match &mut substituted {
        Expr::Path(path) => {
            for segment in &mut path.path.segments {
                if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
                    for argument in &mut arguments.args {
                        match argument {
                            GenericArgument::Type(argument) => {
                                *argument = substitute_type(argument, substitutions);
                            }
                            GenericArgument::Const(argument) => {
                                *argument = substitute_expr(argument, substitutions);
                            }
                            GenericArgument::Lifetime(_) => {}
                            _ => {}
                        }
                    }
                }
            }
        }
        Expr::Paren(paren) => {
            *paren.expr = substitute_expr_inner(&paren.expr, substitutions, seen);
        }
        Expr::Group(group) => {
            *group.expr = substitute_expr_inner(&group.expr, substitutions, seen);
        }
        Expr::Block(block) => {
            for statement in &mut block.block.stmts {
                if let syn::Stmt::Expr(expression, _) = statement {
                    *expression = substitute_expr_inner(expression, substitutions, seen);
                }
            }
        }
        Expr::Const(constant) => {
            for statement in &mut constant.block.stmts {
                if let syn::Stmt::Expr(expression, _) = statement {
                    *expression = substitute_expr_inner(expression, substitutions, seen);
                }
            }
        }
        _ => {}
    }
    substituted
}

fn type_ref(ty: &Type) -> Option<TypeRef> {
    match ty {
        Type::Path(TypePath {
            qself: None, path, ..
        }) => {
            let last = path.segments.last()?;
            let mut arguments = Vec::new();
            let mut unsupported_arguments = false;
            match &last.arguments {
                PathArguments::None => {}
                PathArguments::AngleBracketed(arguments_list) => {
                    for argument in &arguments_list.args {
                        match argument {
                            GenericArgument::Type(argument) => {
                                arguments.push(GenericArgumentValue::Type(argument.clone()));
                            }
                            GenericArgument::Const(argument) => {
                                arguments.push(GenericArgumentValue::Const(argument.clone()));
                            }
                            GenericArgument::Lifetime(argument) => {
                                arguments.push(GenericArgumentValue::Lifetime(argument.clone()));
                            }
                            _ => unsupported_arguments = true,
                        }
                    }
                }
                PathArguments::Parenthesized(_) => unsupported_arguments = true,
            }
            Some(TypeRef {
                path: path_ref(path),
                arguments,
                unsupported_arguments,
            })
        }
        Type::Paren(paren) => type_ref(&paren.elem),
        Type::Group(group) => type_ref(&group.elem),
        _ => None,
    }
}

fn type_name(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath {
            qself: None, path, ..
        }) => path
            .segments
            .iter()
            .map(|segment| {
                let name = identifier_name(&segment.ident);
                match &segment.arguments {
                    PathArguments::None => name,
                    PathArguments::AngleBracketed(arguments) => {
                        let arguments = arguments
                            .args
                            .iter()
                            .map(generic_argument_name)
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{name}<{arguments}>")
                    }
                    PathArguments::Parenthesized(_) => format!("{name}(?)"),
                }
            })
            .collect::<Vec<_>>()
            .join("::"),
        Type::Paren(paren) => format!("({})", type_name(&paren.elem)),
        Type::Group(group) => type_name(&group.elem),
        Type::Tuple(tuple) => format!(
            "({})",
            tuple
                .elems
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => "?".into(),
    }
}

fn generic_argument_name(argument: &GenericArgument) -> String {
    match argument {
        GenericArgument::Type(argument) => type_name(argument),
        GenericArgument::Const(argument) => expression_name(argument),
        GenericArgument::Lifetime(argument) => format!("'{}", argument.ident),
        _ => "?".into(),
    }
}

fn expression_name(expr: &Expr) -> String {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_digits().to_owned(),
        Expr::Path(path) => path
            .path
            .segments
            .iter()
            .map(|segment| identifier_name(&segment.ident))
            .collect::<Vec<_>>()
            .join("::"),
        Expr::Paren(paren) => format!("({})", expression_name(&paren.expr)),
        Expr::Group(group) => expression_name(&group.expr),
        Expr::Block(block) => block
            .block
            .stmts
            .iter()
            .find_map(|statement| match statement {
                syn::Stmt::Expr(expression, _) => Some(expression_name(expression)),
                _ => None,
            })
            .map_or_else(|| "{}".into(), |value| format!("{{ {value} }}")),
        Expr::Const(constant) => constant
            .block
            .stmts
            .iter()
            .find_map(|statement| match statement {
                syn::Stmt::Expr(expression, _) => Some(expression_name(expression)),
                _ => None,
            })
            .map_or_else(|| "const {}".into(), |value| format!("const {{ {value} }}")),
        _ => "?".into(),
    }
}

fn array_is_zero(expr: &Expr, substitutions: &GenericSubstitutions) -> bool {
    const_is_zero(expr, substitutions, &mut BTreeSet::new())
}

fn const_is_zero(
    expr: &Expr,
    substitutions: &GenericSubstitutions,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let substituted = substitute_expr(expr, substitutions);
    match &substituted {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_digits().chars().all(|digit| digit == '0'),
        Expr::Paren(paren) => const_is_zero(&paren.expr, substitutions, visiting),
        Expr::Group(group) => const_is_zero(&group.expr, substitutions, visiting),
        Expr::Block(block) => block
            .block
            .stmts
            .iter()
            .find_map(|statement| match statement {
                syn::Stmt::Expr(expression, _) => {
                    Some(const_is_zero(expression, substitutions, visiting))
                }
                _ => None,
            })
            .unwrap_or(false),
        Expr::Const(constant) => constant
            .block
            .stmts
            .iter()
            .find_map(|statement| match statement {
                syn::Stmt::Expr(expression, _) => {
                    Some(const_is_zero(expression, substitutions, visiting))
                }
                _ => None,
            })
            .unwrap_or(false),
        Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && matches!(path.path.segments[0].arguments, PathArguments::None) =>
        {
            let name = identifier_name(&path.path.segments[0].ident);
            let Some(replacement) = substitutions.consts.get(&name) else {
                return false;
            };
            if !visiting.insert(name.clone()) {
                return false;
            }
            let result = const_is_zero(replacement, substitutions, visiting);
            visiting.remove(&name);
            result
        }
        _ => false,
    }
}

fn is_visible_from(
    visibility: &syn::Visibility,
    defined_module: &[String],
    importing_module: &[String],
) -> bool {
    match visibility {
        syn::Visibility::Public(_) => true,
        syn::Visibility::Inherited => importing_module.starts_with(defined_module),
        syn::Visibility::Restricted(restricted) => {
            let segments = restricted
                .path
                .segments
                .iter()
                .map(|segment| identifier_name(&segment.ident))
                .collect::<Vec<_>>();
            let first = segments.first().map(String::as_str);
            let mut scope = match first {
                Some("crate") => Vec::new(),
                Some("self") => defined_module.to_vec(),
                Some("super") => {
                    let mut scope = defined_module.to_vec();
                    let mut index = 0;
                    while segments
                        .get(index)
                        .is_some_and(|segment| segment == "super")
                    {
                        scope.pop();
                        index += 1;
                    }
                    scope
                }
                _ => return false,
            };
            let start = if first == Some("super") {
                segments
                    .iter()
                    .take_while(|segment| *segment == "super")
                    .count()
            } else {
                1
            };
            scope.extend(segments.into_iter().skip(start));
            importing_module.starts_with(&scope)
        }
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
        if list.path.is_ident("all") {
            nested
                .iter()
                .any(|meta| matches!(meta, Meta::Path(path) if path.is_ident("test")))
        } else {
            nested.len() == 1
                && matches!(nested.first(), Some(Meta::Path(path)) if path.is_ident("test"))
        }
    })
}

fn test_attribute(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("test"))
}

fn allowed_item_attribute(item: &Item, attribute: &Attribute) -> bool {
    if attribute.path().is_ident("cfg") || attribute.path().is_ident("doc") {
        return true;
    }
    if attribute.path().is_ident("test") {
        return matches!(item, Item::Fn(_));
    }
    if attribute.path().is_ident("path") {
        return matches!(item, Item::Mod(_));
    }
    if !attribute.path().is_ident("derive") {
        return false;
    }
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    let derives = Punctuated::<syn::Path, Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .unwrap_or_default();
    derives.iter().all(|path| {
        let name = path
            .segments
            .iter()
            .map(|segment| identifier_name(&segment.ident))
            .collect::<Vec<_>>()
            .join("::");
        ALLOWED_DERIVES.contains(&name.as_str())
    })
}

fn item_has_unreviewed_attribute(item: &Item) -> bool {
    item_attrs(item)
        .iter()
        .any(|attribute| !allowed_item_attribute(item, attribute))
}

fn allowed_nested_attribute(attribute: &Attribute) -> bool {
    let path = attribute.path();
    path.is_ident("allow")
        || path.is_ident("cfg")
        || path.is_ident("cold")
        || path.is_ident("deny")
        || path.is_ident("deprecated")
        || path.is_ident("doc")
        || path.is_ident("forbid")
        || path.is_ident("inline")
        || path.is_ident("must_use")
        || path.is_ident("track_caller")
        || path.is_ident("warn")
}

fn nested_attribute_name(attribute: &Attribute) -> String {
    attribute
        .path()
        .segments
        .iter()
        .map(|segment| identifier_name(&segment.ident))
        .collect::<Vec<_>>()
        .join("::")
}

fn scan_nested_attributes(
    attributes: &[Attribute],
    kind: ViolationKind,
    violations: &mut Vec<Violation>,
) {
    if let Some(attribute) = attributes
        .iter()
        .find(|attribute| !allowed_nested_attribute(attribute))
    {
        violations.push(issue(
            kind,
            format!(
                "unreviewed nested item attribute: {}",
                nested_attribute_name(attribute)
            ),
        ));
    }
}

fn scan_impl_items(items: &[syn::ImplItem], violations: &mut Vec<Violation>) {
    for item in items {
        match item {
            syn::ImplItem::Const(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::ImplItem::Fn(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::ImplItem::Type(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::ImplItem::Macro(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations);
                violations.push(issue(
                    ViolationKind::MacroSurface,
                    "implementation item macro",
                ));
            }
            syn::ImplItem::Verbatim(_) => {
                violations.push(issue(
                    ViolationKind::MacroSurface,
                    "implementation item macro",
                ));
            }
            _ => {}
        }
    }
}

fn scan_trait_items(items: &[syn::TraitItem], violations: &mut Vec<Violation>) {
    for item in items {
        match item {
            syn::TraitItem::Const(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::TraitItem::Fn(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::TraitItem::Type(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations)
            }
            syn::TraitItem::Macro(item) => {
                scan_nested_attributes(&item.attrs, ViolationKind::MacroSurface, violations);
                violations.push(issue(ViolationKind::MacroSurface, "trait item macro"));
            }
            syn::TraitItem::Verbatim(_) => {
                violations.push(issue(ViolationKind::MacroSurface, "trait item macro"));
            }
            _ => {}
        }
    }
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn issue(kind: ViolationKind, detail: impl Into<String>) -> Violation {
    Violation {
        kind,
        detail: detail.into(),
    }
}

fn structural_violations(corpus: &Corpus) -> Vec<Violation> {
    let resolver = Resolver::new(corpus);
    let mut violations = Vec::new();
    for unit in &corpus.units {
        scan_items(&unit.items, &unit.module, &resolver, false, &mut violations);
    }
    violations
}

fn scan_items(
    items: &[Item],
    module: &[String],
    resolver: &Resolver<'_>,
    test_only: bool,
    violations: &mut Vec<Violation>,
) {
    for item in items {
        if !test_only && item_has_unreviewed_attribute(item) {
            violations.push(issue(
                ViolationKind::MacroSurface,
                format!(
                    "unreviewed item attribute: {}",
                    item_attrs(item)
                        .iter()
                        .find(|attribute| !allowed_item_attribute(item, attribute))
                        .map(|attribute| {
                            attribute
                                .path()
                                .segments
                                .iter()
                                .map(|segment| identifier_name(&segment.ident))
                                .collect::<Vec<_>>()
                                .join("::")
                        })
                        .unwrap_or_default()
                ),
            ));
        }
        match item {
            Item::Macro(_) | Item::Verbatim(_) if !test_only => {
                violations.push(issue(
                    ViolationKind::MacroSurface,
                    "item-position macro surface",
                ));
            }
            Item::Fn(function)
                if !(test_only
                    || test_attribute(&function.attrs)
                    || cfg_test_only(&function.attrs)
                    || (module.is_empty() && identifier_name(&function.sig.ident) == "main")) =>
            {
                violations.push(issue(
                    ViolationKind::FreeFunction,
                    "production free function",
                ));
            }
            Item::ForeignMod(foreign) if !test_only => {
                for foreign_item in &foreign.items {
                    if matches!(foreign_item, syn::ForeignItem::Fn(_)) {
                        violations.push(issue(
                            ViolationKind::FreeFunction,
                            "foreign function declaration",
                        ));
                    }
                    if matches!(foreign_item, syn::ForeignItem::Macro(_)) {
                        violations.push(issue(ViolationKind::MacroSurface, "foreign item macro"));
                    }
                }
            }
            Item::Impl(ItemImpl {
                trait_,
                self_ty,
                items: impl_items,
                ..
            }) if !test_only => {
                if trait_.is_none() {
                    violations.push(issue(
                        ViolationKind::InherentImpl,
                        "inherent implementation",
                    ));
                } else if resolver.resolve_self_type(module, self_ty) {
                    violations.push(issue(
                        ViolationKind::ZstBehavior,
                        format!(
                            "behavior attached to known zero-sized type `{}`",
                            type_name(self_ty)
                        ),
                    ));
                }
                scan_impl_items(impl_items, violations);
            }
            Item::Trait(item) if !test_only => {
                scan_trait_items(&item.items, violations);
            }
            Item::Mod(item) => {
                if let Some((_, nested)) = &item.content {
                    let mut nested_module = module.to_owned();
                    nested_module.push(identifier_name(&item.ident));
                    scan_items(
                        nested,
                        &nested_module,
                        resolver,
                        test_only || cfg_test_only(&item.attrs),
                        violations,
                    );
                }
            }
            _ => {}
        }
    }
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
        if source[..start]
            .chars()
            .next_back()
            .is_some_and(xid_continue)
            || source[end..].chars().next().is_some_and(xid_continue)
        {
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
    let corpus = Corpus::from_paths(vec![path.to_owned()]);
    let violations = structural_violations(&corpus);
    assert!(
        violations.is_empty(),
        "good fixture {} was rejected: {:?}",
        path.display(),
        violations
            .iter()
            .map(|violation| format!("{:?}: {}", violation.kind, violation.detail))
            .collect::<Vec<_>>()
    );
}

fn assert_has_kind(path: &Path, kind: ViolationKind) {
    let corpus = Corpus::from_paths(vec![path.to_owned()]);
    let violations = structural_violations(&corpus);
    assert!(
        violations.iter().any(|violation| violation.kind == kind),
        "bad fixture {} did not expose {kind:?}",
        path.display()
    );
}

fn assert_exact_zst_names(path: &Path, expected: &[&str]) {
    let corpus = Corpus::from_paths(vec![path.to_owned()]);
    let actual = structural_violations(&corpus)
        .into_iter()
        .filter(|violation| violation.kind == ViolationKind::ZstBehavior)
        .filter_map(|violation| {
            violation
                .detail
                .strip_prefix("behavior attached to known zero-sized type `")
                .and_then(|detail| detail.strip_suffix('`'))
                .map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "unexpected ZST behavior witness set");
}

fn assert_exact_nested_attribute_names(path: &Path, expected: &[&str]) {
    let corpus = Corpus::from_paths(vec![path.to_owned()]);
    let actual_names = structural_violations(&corpus)
        .into_iter()
        .filter(|violation| violation.kind == ViolationKind::MacroSurface)
        .filter_map(|violation| {
            violation
                .detail
                .strip_prefix("unreviewed nested item attribute: ")
                .map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    let actual_count = structural_violations(&corpus)
        .into_iter()
        .filter(|violation| violation.kind == ViolationKind::MacroSurface)
        .filter(|violation| {
            violation
                .detail
                .starts_with("unreviewed nested item attribute: ")
        })
        .count();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_count, 6);
    assert_eq!(
        actual_names, expected,
        "unexpected nested attribute witness set"
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
    assert_exact_zst_names(
        &root.join("zst-behavior-bad.rs"),
        &[
            "(Empty)",
            "Empty",
            "Generic<T>",
            "Tuple",
            "crate::namespace::Namespaced",
            "self::Namespaced",
        ],
    );
    let cross_file_corpus = Corpus::from_paths(vec![
        root.join("zst-cross-file-decl.rs"),
        root.join("zst-cross-file-impl.rs"),
    ]);
    let cross_file_actual = structural_violations(&cross_file_corpus)
        .into_iter()
        .filter(|violation| violation.kind == ViolationKind::ZstBehavior)
        .filter_map(|violation| {
            violation
                .detail
                .strip_prefix("behavior attached to known zero-sized type `")
                .and_then(|detail| detail.strip_suffix('`'))
                .map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        cross_file_actual,
        BTreeSet::from(["(crate::CrossFile)".to_owned()]),
        "unexpected cross-file ZST witness set"
    );
    assert_exact_zst_names(
        &root.join("zst-resolution-bad.rs"),
        &[
            "GlobZst",
            "GrandparentZst",
            "ImportedNamespace::AliasZst",
            "ImportedZst",
            "SecondAlias",
            "alias::AliasZst",
            "crate::path_forms::CrateZst",
            "nested::Unit",
            "self::SelfZst",
            "super::super::source::MultiSuperZst",
            "super::SuperZst",
        ],
    );
    assert_no_structural_violations(&root.join("zst-resolution-good.rs"));
    assert_exact_zst_names(
        &root.join("zst-layout-bad.rs"),
        &[
            "ArrayOfEmpty",
            "ArrayOuter<()>",
            "Braced",
            "CoreHolder",
            "DirectGlobHolder",
            "Empty",
            "GenericAlias<()>",
            "GenericAliasTransitive<()>",
            "GlobHolder",
            "Leaf<()>",
            "LeafUnit",
            "Nested",
            "NestedOuter<()>",
            "Outer<()>",
            "OuterAlias<()>",
            "OuterAliasTransitive<()>",
            "ParenOuter<()>",
            "Pair",
            "PairAlias",
            "PairAliasTransitive",
            "PhantomAlias<()>",
            "PhantomAliasTransitive<()>",
            "ReexportHolder",
            "StdHolder",
            "Tuple",
            "TupleAlias<()>",
            "UnitWrapper",
            "ArrayAlias<()>",
            "ParenAlias<()>",
            "Wrapper<()>",
            "ZeroArray",
        ],
    );
    assert_no_structural_violations(&root.join("zst-layout-good.rs"));
    assert_exact_zst_names(
        &root.join("zst-const-lifetime-bad.rs"),
        &[
            "Marker<{ 0 }>",
            "Marker<{ 1 }>",
            "Marker<1>",
            "LifetimeMarker<'static>",
            "PhantomMarker<'static, { 42 }>",
            "ByteArray<{ 0 }>",
            "ZstArray<(), { 1 }>",
            "Mixed<'static, (), { 3 }>",
            "MarkerAlias<{ 2 }>",
            "LifetimeAlias<'static>",
            "PhantomAlias<'static, { 7 }>",
            "MixedAlias<'static, (), { 5 }>",
            "ZstArrayAlias<(), { 4 }>",
            "ParenthesizedMarker<{ 4 }>",
            "ParenthesizedByteArray<{ 0 }>",
        ],
    );
    assert_no_structural_violations(&root.join("zst-const-lifetime-good.rs"));
    assert_has_kind(&root.join("macro-bad.rs"), ViolationKind::MacroSurface);
    assert_no_structural_violations(&root.join("macro-good.rs"));
    assert_exact_nested_attribute_names(
        &root.join("nested-items-bad.rs"),
        &[
            "const_attribute_macro",
            "evil::inline",
            "method_attribute_macro",
            "type_attribute_macro",
        ],
    );
    assert_no_structural_violations(&root.join("nested-items-good.rs"));
    assert_eq!(
        vocabulary_violations(&fs::read_to_string(root.join("vocabulary-good.rs")).unwrap()),
        0
    );
    assert!(
        vocabulary_violations(&fs::read_to_string(root.join("vocabulary-bad.rs")).unwrap()) >= 7
    );
}

#[test]
fn production_src_obeys_architecture_binding_law() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let corpus = Corpus::production(&root);
    let violations = structural_violations(&corpus);
    assert!(
        violations.is_empty(),
        "production structural violations: {:?}",
        violations
            .iter()
            .map(|violation| format!("{:?}: {}", violation.kind, violation.detail))
            .collect::<Vec<_>>()
    );
    let vocabulary = corpus
        .sources
        .values()
        .map(|source| vocabulary_violations(source))
        .sum::<usize>();
    assert_eq!(
        vocabulary, 0,
        "production forbidden vocabulary matches: {vocabulary}"
    );
}
