//! Generation: File to Rust text (cannot fault, the file having been checked).
//!
//! Each declaration emits itself: a struct declaration its struct and
//! its datomic machinery, an enum declaration its enum and its
//! machinery, a kind declaration its trait, an association its
//! assertion; the file emits by walking its variant's sections. Names
//! are resolved through the scope, never a table; the generated Rust
//! carries no `use` and writes every foreign name fully qualified.
//!
//! One rule decides boxing: a position whose type reaches the type
//! that declares it, walking through declared types, aliases, `Option`
//! and `Result` but not through `Vector`, is boxed as a whole
//! (`std::boxed::Box<std::option::Option<Tree>>`), and the datom machinery
//! never sees the box.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::checking::Declaring;
use crate::datomization::{Datomizing, Uniform};
use crate::{
    AssociatedConstant, AssociatedType, Association, Capability, Constraint, File, Generating,
    Identity, Intrinsic, KindBody, KindDeclaration, Name, Receiver, Reference, Resolution,
    Resolving, Scope, Signature, Source, TypeDeclaration, Variant,
};

// ---------------------------------------------------------------------------
// Tokens of names and sources
// ---------------------------------------------------------------------------

/// The kind whose capability yields a value's Rust tokens without any scope.
pub(crate) trait Tokening {
    fn tokens(&self) -> TokenStream;
}

impl Tokening for Name {
    fn tokens(&self) -> TokenStream {
        let ident = Ident::new(&self.0, Span::call_site());
        quote! { #ident }
    }
}

impl Tokening for Source {
    fn tokens(&self) -> TokenStream {
        let path: syn::Path = syn::parse_str(&self.0).expect("a source was validated as a path");
        quote! { #path }
    }
}

impl Tokening for Intrinsic {
    fn tokens(&self) -> TokenStream {
        match self {
            Intrinsic::Text => quote! { protos::Text },
            Intrinsic::Integer => quote! { protos::Integer },
            Intrinsic::Decimal => quote! { protos::Decimal },
            Intrinsic::Boolean => quote! { protos::Boolean },
            Intrinsic::Meaning => quote! { datom_codec::Meaning },
            Intrinsic::Vector => quote! { std::vec::Vec },
            Intrinsic::Option => quote! { std::option::Option },
            Intrinsic::Result => quote! { std::result::Result },
            Intrinsic::Itself => quote! { Self },
            Intrinsic::Sized => quote! { Sized },
        }
    }
}

/// The kind whose capability yields the name of the parameter at an index: A, B, C.
trait Lettering {
    fn letter(&self) -> Ident;
}

impl Lettering for usize {
    fn letter(&self) -> Ident {
        let letter = char::from(b'A' + *self as u8);
        Ident::new(&letter.to_string(), Span::call_site())
    }
}

/// The kind whose capability lowercases a name for an assertion function.
trait Lowering {
    fn lowered(&self) -> String;
}

impl Lowering for Reference {
    fn lowered(&self) -> String {
        let mut lowered = self.name.0.to_lowercase();
        for argument in &self.arguments {
            lowered.push('_');
            lowered.push_str(&argument.lowered());
        }
        lowered
    }
}

// ---------------------------------------------------------------------------
// Emitting: Rust tokens in a scope
// ---------------------------------------------------------------------------

/// The kind whose capability yields a value's Rust tokens in a scope.
pub(crate) trait Emitting {
    fn emit(&self, scope: &Scope) -> TokenStream;
}

impl Emitting for Reference {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let mut arguments = Vec::with_capacity(self.arguments.len());
        for argument in &self.arguments {
            arguments.push(argument.emit(scope));
        }
        let applied = if arguments.is_empty() {
            TokenStream::new()
        } else {
            quote! { < #( #arguments ),* > }
        };
        let name = self.name.tokens();
        if let Some(source) = &self.source {
            let source = source.tokens();
            return quote! { #source :: #name #applied };
        }
        match scope.resolve(&self.name) {
            Resolution::Intrinsic(intrinsic) => {
                let intrinsic = intrinsic.tokens();
                quote! { #intrinsic #applied }
            }
            Resolution::Imported(source, emitted) => {
                let source = source.tokens();
                let emitted = emitted.tokens();
                quote! { #source :: #emitted #applied }
            }
            Resolution::Type(_)
            | Resolution::Kind(_)
            | Resolution::Ambiguous(_)
            | Resolution::Undeclared => {
                quote! { #name #applied }
            }
            Resolution::Parameter(index) => {
                let letter = (index as usize).letter();
                quote! { #letter }
            }
            Resolution::Associated(name) => {
                let name = name.tokens();
                quote! { Self::#name }
            }
        }
    }
}

/// The kind whose capability yields the bounds of a constraint: `A + B`.
trait Bounding {
    fn bounds(&self, scope: &Scope) -> TokenStream;
}

impl Bounding for Constraint {
    fn bounds(&self, scope: &Scope) -> TokenStream {
        let references = match self {
            Constraint::One(reference) => std::slice::from_ref(reference),
            Constraint::Many(references) => references,
        };
        references.bounds(scope)
    }
}

impl Bounding for [Reference] {
    fn bounds(&self, scope: &Scope) -> TokenStream {
        let mut bounds = Vec::with_capacity(self.len());
        for reference in self {
            bounds.push(reference.emit(scope));
        }
        quote! { #( #bounds )+* }
    }
}

/// The kind whose capabilities yield an identity's generics: the parameters with their bounds, and the arguments.
pub(crate) trait Parametrizing {
    fn parameters(&self, scope: &Scope, corporate: bool) -> TokenStream;
    fn arguments(&self) -> TokenStream;
}

impl Parametrizing for Identity {
    fn parameters(&self, scope: &Scope, corporate: bool) -> TokenStream {
        if self.constraints.is_empty() {
            return TokenStream::new();
        }
        // The bounds name kinds outside the identity they bound.
        let outer = Scope {
            file: scope.file,
            identity: None,
            associated: scope.associated,
        };
        let mut parameters = Vec::with_capacity(self.constraints.len());
        for (index, constraint) in self.constraints.iter().enumerate() {
            let letter = index.letter();
            let bounds = constraint.bounds(&outer);
            if corporate {
                parameters.push(quote! { #letter: #bounds + datom_codec::Datomic });
            } else {
                parameters.push(quote! { #letter: #bounds });
            }
        }
        quote! { < #( #parameters ),* > }
    }

    fn arguments(&self) -> TokenStream {
        if self.constraints.is_empty() {
            return TokenStream::new();
        }
        let mut letters = Vec::with_capacity(self.constraints.len());
        for index in 0..self.constraints.len() {
            letters.push(index.letter());
        }
        quote! { < #( #letters ),* > }
    }
}

// ---------------------------------------------------------------------------
// Reaching: the boxing rule
// ---------------------------------------------------------------------------

/// The kind whose capability tells whether a value holds the target type by value, through declared types, aliases, Option and Result.
trait Reaching {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool;
}

impl Reaching for Reference {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        if self.source.is_none() {
            if &self.name == target || self.name.0 == "Self" {
                return true;
            }
            match file.resolve(&self.name) {
                Resolution::Intrinsic(Intrinsic::Vector) => return false,
                Resolution::Type(name) if !visited.contains(&name) => {
                    visited.push(name.clone());
                    if let Some(declaration) = file.declaration(&name)
                        && declaration.reaches(target, file, visited)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        for argument in &self.arguments {
            if argument.reaches(target, file, visited) {
                return true;
            }
        }
        false
    }
}

impl Reaching for [Reference] {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        for reference in self {
            if reference.reaches(target, file, visited) {
                return true;
            }
        }
        false
    }
}

impl Reaching for TypeDeclaration {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        match self {
            TypeDeclaration::Struct(_, positions) => positions.reaches(target, file, visited),
            TypeDeclaration::Enum(_, variants) => variants.reaches(target, file, visited),
            TypeDeclaration::Alias(_, aliased) => aliased.reaches(target, file, visited),
        }
    }
}

impl Reaching for Variant {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        match self {
            Variant::Bare(_) => false,
            Variant::Typed(_, reference) => reference.reaches(target, file, visited),
            Variant::Struct(_, positions) => positions.reaches(target, file, visited),
            Variant::Enum(_, variants) => variants.reaches(target, file, visited),
        }
    }
}

impl Reaching for [Variant] {
    fn reaches(&self, target: &Name, file: &File, visited: &mut Vec<Name>) -> bool {
        for variant in self {
            if variant.reaches(target, file, visited) {
                return true;
            }
        }
        false
    }
}

/// The kind whose capability yields the derives of a declared type: Eq unless it reaches Decimal, Copy when every variant is bare.
trait Deriving {
    fn derives(&self, file: &File, copy: bool) -> TokenStream;
}

impl<R: Reaching + ?Sized> Deriving for R {
    fn derives(&self, file: &File, copy: bool) -> TokenStream {
        let equatable = !self.reaches(&Name("Decimal".to_owned()), file, &mut vec![]);
        match (copy, equatable) {
            (true, true) => quote! { #[derive(Clone, Copy, Debug, PartialEq, Eq)] },
            (true, false) => quote! { #[derive(Clone, Copy, Debug, PartialEq)] },
            (false, true) => quote! { #[derive(Clone, Debug, PartialEq, Eq)] },
            (false, false) => quote! { #[derive(Clone, Debug, PartialEq)] },
        }
    }
}

/// The kind whose capabilities yield a position's Rust type, boxed when it reaches its owner.
pub(crate) trait Positioning {
    fn boxed(&self, scope: &Scope, owner: &Name) -> bool;
    fn position(&self, scope: &Scope, owner: &Name) -> TokenStream;
}

impl Positioning for Reference {
    fn boxed(&self, scope: &Scope, owner: &Name) -> bool {
        self.reaches(owner, scope.file, &mut vec![])
    }

    fn position(&self, scope: &Scope, owner: &Name) -> TokenStream {
        let ty = self.emit(scope);
        if self.boxed(scope, owner) {
            quote! { std::boxed::Box<#ty> }
        } else {
            ty
        }
    }
}

/// The kind whose capabilities yield a variant's definition and the items its inline enum needs.
trait Varianted {
    fn definition(&self, scope: &Scope, owner: &Name, enclosing: &Identity) -> TokenStream;
    fn nested(&self, scope: &Scope, owner: &Name, enclosing: &Identity) -> TokenStream;
}

/// The kind whose capability names the enum type an inline enum variant declares.
trait Nesting {
    fn nested_identity(&self, name: &Name) -> Identity;
}

impl Nesting for Identity {
    fn nested_identity(&self, name: &Name) -> Identity {
        Identity {
            name: Name(format!("{}{}", self.name.0, name.0)),
            constraints: self.constraints.clone(),
        }
    }
}

impl Varianted for Variant {
    fn definition(&self, scope: &Scope, owner: &Name, enclosing: &Identity) -> TokenStream {
        match self {
            Variant::Bare(name) => name.tokens(),
            Variant::Typed(name, reference) => {
                let name = name.tokens();
                let ty = reference.position(scope, owner);
                quote! { #name(#ty) }
            }
            Variant::Struct(name, positions) => {
                let name = name.tokens();
                let mut types = Vec::with_capacity(positions.len());
                for position in positions {
                    types.push(position.position(scope, owner));
                }
                quote! { #name( #( #types ),* ) }
            }
            Variant::Enum(name, _) => {
                let nested = enclosing.nested_identity(name);
                let ty = nested.name.tokens();
                let arguments = nested.arguments();
                let name = name.tokens();
                quote! { #name(#ty #arguments) }
            }
        }
    }

    fn nested(&self, scope: &Scope, owner: &Name, enclosing: &Identity) -> TokenStream {
        match self {
            Variant::Enum(name, variants) => {
                let nested = enclosing.nested_identity(name);
                variants.enumeration(scope, owner, &nested)
            }
            Variant::Bare(_) | Variant::Typed(_, _) | Variant::Struct(_, _) => TokenStream::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Declaring: the items a type declaration emits
// ---------------------------------------------------------------------------

/// The kind whose capability emits a struct of these positions with its datomic machinery.
trait Structuring {
    fn structure(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream;
}

/// The kind whose capability emits an enum of these variants with its datomic machinery.
trait Enumerating {
    fn enumeration(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream;
}

impl Structuring for [Reference] {
    fn structure(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let name = identity.name.tokens();
        let parameters = identity.parameters(scope, false);
        let mut types = Vec::with_capacity(self.len());
        for position in self {
            types.push(position.position(scope, owner));
        }
        let derive = self.derives(scope.file, false);
        let machinery = self.machinery(scope, owner, identity);
        quote! {
            #derive
            pub struct #name #parameters ( #( pub #types ),* );
            #machinery
        }
    }
}

impl Enumerating for [Variant] {
    fn enumeration(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let name = identity.name.tokens();
        let parameters = identity.parameters(scope, false);
        let derive = self.derives(scope.file, self.all_bare());
        let mut definitions = Vec::with_capacity(self.len());
        let mut nested = Vec::new();
        for variant in self {
            definitions.push(variant.definition(scope, owner, identity));
            nested.push(variant.nested(scope, owner, identity));
        }
        let machinery = self.machinery(scope, owner, identity);
        quote! {
            #( #nested )*
            #derive
            pub enum #name #parameters { #( #definitions ),* }
            #machinery
        }
    }
}

impl Emitting for TypeDeclaration {
    fn emit(&self, scope: &Scope) -> TokenStream {
        match self {
            TypeDeclaration::Struct(identity, positions) => {
                let inner = Scope {
                    file: scope.file,
                    identity: Some(identity),
                    associated: scope.associated,
                };
                positions.structure(&inner, &identity.name, identity)
            }
            TypeDeclaration::Enum(identity, variants) => {
                let inner = Scope {
                    file: scope.file,
                    identity: Some(identity),
                    associated: scope.associated,
                };
                variants.enumeration(&inner, &identity.name, identity)
            }
            TypeDeclaration::Alias(identity, aliased) => {
                let inner = Scope {
                    file: scope.file,
                    identity: Some(identity),
                    associated: scope.associated,
                };
                let name = identity.name.tokens();
                let parameters = identity.parameters(&inner, false);
                let aliased = aliased.emit(&inner);
                quote! { pub type #name #parameters = #aliased; }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Kinds: traits
// ---------------------------------------------------------------------------

/// The named sections that compose a kind declaration.
struct KindContents<'a> {
    superkinds: &'a [Reference],
    types: &'a [AssociatedType],
    constants: &'a [AssociatedConstant],
    capabilities: &'a [Capability],
}

/// The kind whose capability exposes a declaration's named kind sections.
trait Containing {
    fn contents(&self) -> KindContents<'_>;
}

impl Containing for KindDeclaration {
    fn contents(&self) -> KindContents<'_> {
        match &self.body {
            KindBody::Simple(capabilities) => KindContents {
                superkinds: &[],
                types: &[],
                constants: &[],
                capabilities,
            },
            KindBody::Complex {
                superkinds,
                types,
                constants,
                capabilities,
            } => KindContents {
                superkinds,
                types,
                constants,
                capabilities,
            },
        }
    }
}

impl Emitting for AssociatedType {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let name = self.name.tokens();
        if self.bounds.is_empty() {
            quote! { type #name; }
        } else {
            let bounds = self.bounds.bounds(scope);
            quote! { type #name: #bounds; }
        }
    }
}

impl Emitting for AssociatedConstant {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let name = self.name.tokens();
        let ty = self.ty.emit(scope);
        quote! { const #name: #ty; }
    }
}

impl Emitting for Capability {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let name = self.name.tokens();
        let mut parameters = Vec::new();
        match self.receiver {
            Receiver::Shared => parameters.push(quote! { &self }),
            Receiver::Mutable => parameters.push(quote! { &mut self }),
            Receiver::Static => {}
        }
        let yields = match &self.signature {
            Signature::Yielding(yields) => yields,
            Signature::Taking(inputs, yields) => {
                if let [input] = inputs.as_slice() {
                    let ty = input.emit(scope);
                    parameters.push(quote! { input: #ty });
                } else {
                    for (index, input) in inputs.iter().enumerate() {
                        let input_name = Ident::new(&format!("input_{index}"), Span::call_site());
                        let ty = input.emit(scope);
                        parameters.push(quote! { #input_name: #ty });
                    }
                }
                yields
            }
        };
        let yields = yields.emit(scope);
        quote! { fn #name( #( #parameters ),* ) -> #yields; }
    }
}

impl Emitting for KindDeclaration {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let KindContents {
            superkinds,
            types,
            constants,
            capabilities,
        } = self.contents();
        let inner = Scope {
            file: scope.file,
            identity: Some(&self.identity),
            associated: types,
        };
        let name = self.identity.name.tokens();
        let parameters = self.identity.parameters(&inner, false);
        let extends = if superkinds.is_empty() {
            TokenStream::new()
        } else {
            let bounds = superkinds.bounds(&inner);
            quote! { : #bounds }
        };
        let mut items = Vec::new();
        for associated in types {
            items.push(associated.emit(&inner));
        }
        for constant in constants {
            items.push(constant.emit(&inner));
        }
        for capability in capabilities {
            items.push(capability.emit(&inner));
        }
        quote! {
            pub trait #name #parameters #extends { #( #items )* }
        }
    }
}

// ---------------------------------------------------------------------------
// Associations: compile-time assertions
// ---------------------------------------------------------------------------

impl Emitting for Association {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let inner = Scope {
            file: scope.file,
            identity: Some(&self.identity),
            associated: scope.associated,
        };
        let subject = Reference {
            source: None,
            name: self.identity.name.clone(),
            arguments: vec![],
        };
        let ty = subject.emit(scope);
        let arguments = self.identity.arguments();
        let parameters = self.identity.parameters(&inner, false);
        let mut assertions = Vec::with_capacity(self.kinds.len());
        for kind in &self.kinds {
            let assertion = Ident::new(
                &format!(
                    "assert_{}_{}",
                    self.identity.name.0.to_lowercase(),
                    kind.lowered()
                ),
                Span::call_site(),
            );
            let bound = kind.emit(scope);
            if self.identity.constraints.is_empty() {
                assertions.push(quote! {
                    fn #assertion<T: #bound>() {}
                    let _ = #assertion::<#ty>;
                });
            } else {
                assertions.push(quote! {
                    fn #assertion #parameters () {
                        fn assertion<T: #bound>() {}
                        let _ = assertion::<#ty #arguments>;
                    }
                });
            }
        }
        quote! { const _: () = { #( #assertions )* }; }
    }
}

// ---------------------------------------------------------------------------
// The file: its sections walked
// ---------------------------------------------------------------------------

impl Emitting for File {
    fn emit(&self, scope: &Scope) -> TokenStream {
        let mut items = vec![quote! { #![allow(dead_code)] }];
        match self {
            File::Types(types) => {
                for declaration in &types.types {
                    items.push(declaration.emit(scope));
                }
                for association in &types.associations {
                    items.push(association.emit(scope));
                }
            }
            File::Kinds(kinds) => {
                for declaration in &kinds.kinds {
                    items.push(declaration.emit(scope));
                }
            }
            File::Signal(signal) => {
                for declaration in &signal.types {
                    items.push(declaration.emit(scope));
                }
                let request = TypeDeclaration::Enum(
                    Identity {
                        name: Name("Request".to_owned()),
                        constraints: vec![],
                    },
                    signal.requests.clone(),
                );
                let response = TypeDeclaration::Enum(
                    Identity {
                        name: Name("Response".to_owned()),
                        constraints: vec![],
                    },
                    signal.responses.clone(),
                );
                items.push(request.emit(scope));
                items.push(response.emit(scope));
            }
            File::Sema(sema) => {
                let record = TypeDeclaration::Struct(
                    Identity {
                        name: Name("Record".to_owned()),
                        constraints: vec![],
                    },
                    sema.record.clone(),
                );
                items.push(record.emit(scope));
                for declaration in &sema.types {
                    items.push(declaration.emit(scope));
                }
            }
        }
        quote! { #( #items )* }
    }
}

impl Generating for File {
    fn generate(&self) -> String {
        let scope = Scope {
            file: self,
            identity: None,
            associated: &[],
        };
        let tokens = self.emit(&scope);
        let file: syn::File = syn::parse2(tokens).expect("generated tokens are a Rust file");
        prettyplease::unparse(&file)
    }
}
