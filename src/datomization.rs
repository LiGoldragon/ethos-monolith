//! Datomization: the datomic machinery every declared type bears.
//!
//! Everything emitted here depends on the shape datomic 0.12.0 gives
//! its own intrinsics, and on nothing else: a re-pin of datomic that
//! changes that shape touches this module alone. For a declared type
//! the machinery is `protos::Conceivable<datomic::Datom>` with an
//! infallible fault, `datomic::Datomic::incorporate_from` prepending
//! every position's index to a nested fault (`datomic::Prepending`),
//! and `protos::Incorporable<Type>` for `datomic::Datom`. A variant
//! carrying nothing is `Datom::Bare`; one carrying data is
//! `Datom::Variant(Head::Bare, Box<Datom>)`, its body at child 0; a
//! struct is `Datom::Struct` of its positions in order.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::generation::{Emitting, Parametrizing, Positioning, Tokening};
use crate::{Identity, Name, Reference, Scope, Variant};

// ---------------------------------------------------------------------------
// Positions: conceive and incorporate a struct body
// ---------------------------------------------------------------------------

/// How a position is borrowed for conceiving: as it is, and through its box.
struct Borrow {
    /// An expression of type `&Position`: `&self.0`, or a binding `p0`.
    plain: TokenStream,
    /// The same through the box: `&*self.0`, or `&**p0`.
    unboxed: TokenStream,
}

/// The kind whose capabilities yield the datomic machinery of a struct body: its positions in order.
trait Positioned {
    /// The conceive of every position, each borrowed as given.
    fn conceived(&self, scope: &Scope, owner: &Name, borrows: &[Borrow]) -> TokenStream;
    /// An expression over `fields: Vec<Datom>` incorporating every position and constructing.
    fn incorporated(&self, scope: &Scope, owner: &Name, constructor: TokenStream) -> TokenStream;
}

impl Positioned for [Reference] {
    fn conceived(&self, scope: &Scope, owner: &Name, borrows: &[Borrow]) -> TokenStream {
        let mut conceived = Vec::with_capacity(self.len());
        for (reference, borrow) in self.iter().zip(borrows) {
            let borrowed = if reference.boxed(scope, owner) {
                &borrow.unboxed
            } else {
                &borrow.plain
            };
            conceived.push(quote! {
                protos::Conceivable::<datomic::Datom>::conceive(#borrowed)?
            });
        }
        quote! { datomic::Datom::Struct(Vec::from([ #( #conceived ),* ])) }
    }

    fn incorporated(&self, scope: &Scope, owner: &Name, constructor: TokenStream) -> TokenStream {
        let arity = Literal::usize_unsuffixed(self.len());
        let arity_integer = Literal::i64_unsuffixed(self.len() as protos::Integer);
        let mut datoms = Vec::with_capacity(self.len());
        let mut values = Vec::with_capacity(self.len());
        for index in 0..self.len() {
            datoms.push(Ident::new(&format!("d{index}"), Span::call_site()));
            values.push(Ident::new(&format!("p{index}"), Span::call_site()));
        }
        // Built from the last position inward: each position's incorporation
        // nests the rest in its Ok arm, so a fault returns placed under its index.
        let mut inner = quote! { Ok(#constructor( #( #values ),* )) };
        for index in (0..self.len()).rev() {
            let ty = self[index].emit(scope);
            let datom = &datoms[index];
            let value = &values[index];
            let index_integer = Literal::i64_unsuffixed(index as protos::Integer);
            let bound = if self[index].boxed(scope, owner) {
                quote! { let #value = Box::new(#value); }
            } else {
                TokenStream::new()
            };
            inner = quote! {
                match <#ty as datomic::Datomic>::incorporate_from(#datom) {
                    Err(fault) => Err(datomic::Prepending::prepend(fault, #index_integer)),
                    Ok(#value) => {
                        #bound
                        #inner
                    }
                }
            };
        }
        quote! {
            match <[datomic::Datom; #arity]>::try_from(fields) {
                Ok([ #( #datoms ),* ]) => #inner,
                Err(fields) => Err(datomic::Fault::Corporate(
                    vec![],
                    datomic::Problem::Arity(#arity_integer, fields.len() as protos::Integer),
                )),
            }
        }
    }
}

/// The kind whose capabilities yield the datomic arms of a variant.
trait Arming {
    fn conceive_arm(&self, scope: &Scope, owner: &Name) -> TokenStream;
    fn incorporate_arm(&self, scope: &Scope, owner: &Name) -> Option<TokenStream>;
}

impl Arming for Variant {
    fn conceive_arm(&self, scope: &Scope, owner: &Name) -> TokenStream {
        match self {
            Variant::Bare(name) => {
                let head = &name.0;
                let name = name.tokens();
                quote! { Self::#name => datomic::Datom::Bare(#head.to_owned()), }
            }
            Variant::Typed(name, reference) => {
                let head = &name.0;
                let name = name.tokens();
                let conceived = if reference.boxed(scope, owner) {
                    quote! { protos::Conceivable::<datomic::Datom>::conceive(&**p0)? }
                } else {
                    quote! { protos::Conceivable::<datomic::Datom>::conceive(p0)? }
                };
                quote! {
                    Self::#name(p0) => datomic::Datom::Variant(
                        protos::Head::Bare(#head.to_owned()),
                        Box::new(#conceived),
                    ),
                }
            }
            Variant::Enum(name, _) => {
                let head = &name.0;
                let name = name.tokens();
                quote! {
                    Self::#name(p0) => datomic::Datom::Variant(
                        protos::Head::Bare(#head.to_owned()),
                        Box::new(protos::Conceivable::<datomic::Datom>::conceive(p0)?),
                    ),
                }
            }
            Variant::Struct(name, positions) => {
                let head = &name.0;
                let name = name.tokens();
                let mut bindings = Vec::with_capacity(positions.len());
                let mut borrows = Vec::with_capacity(positions.len());
                for index in 0..positions.len() {
                    let binding = Ident::new(&format!("p{index}"), Span::call_site());
                    borrows.push(Borrow {
                        plain: quote! { #binding },
                        unboxed: quote! { &**#binding },
                    });
                    bindings.push(binding);
                }
                let conceived = positions.conceived(scope, owner, &borrows);
                quote! {
                    Self::#name( #( #bindings ),* ) => datomic::Datom::Variant(
                        protos::Head::Bare(#head.to_owned()),
                        Box::new(#conceived),
                    ),
                }
            }
        }
    }

    fn incorporate_arm(&self, scope: &Scope, owner: &Name) -> Option<TokenStream> {
        match self {
            Variant::Bare(_) => None,
            Variant::Typed(name, reference) => {
                let head = &name.0;
                let name = name.tokens();
                let ty = reference.emit(scope);
                let value = if reference.boxed(scope, owner) {
                    quote! { Box::new(value) }
                } else {
                    quote! { value }
                };
                Some(quote! {
                    #head => match <#ty as datomic::Datomic>::incorporate_from(*body) {
                        Ok(value) => Ok(Self::#name(#value)),
                        Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                    },
                })
            }
            Variant::Enum(name, _) => {
                let head = &name.0;
                let name = name.tokens();
                Some(quote! {
                    #head => match datomic::Datomic::incorporate_from(*body) {
                        Ok(value) => Ok(Self::#name(value)),
                        Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                    },
                })
            }
            Variant::Struct(name, positions) => {
                let head = &name.0;
                let name = name.tokens();
                let incorporated = positions.incorporated(scope, owner, quote! { Self::#name });
                Some(quote! {
                    #head => match *body {
                        datomic::Datom::Struct(fields) => {
                            let incorporated = #incorporated;
                            match incorporated {
                                Ok(value) => Ok(value),
                                Err(fault) => Err(datomic::Prepending::prepend(fault, 0)),
                            }
                        }
                        other => Err(datomic::Fault::Corporate(
                            vec![0],
                            datomic::Problem::Shape(datomic::Expected::Struct, other),
                        )),
                    },
                })
            }
        }
    }
}

/// The kind whose capabilities tell whether every, or any, variant carries nothing.
pub(crate) trait Uniform {
    fn all_bare(&self) -> bool;
    fn any_bare(&self) -> bool;
}

impl Uniform for [Variant] {
    fn all_bare(&self) -> bool {
        for variant in self {
            if !matches!(variant, Variant::Bare(_)) {
                return false;
            }
        }
        !self.is_empty()
    }

    fn any_bare(&self) -> bool {
        for variant in self {
            if matches!(variant, Variant::Bare(_)) {
                return true;
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// The three interactions a declared type bears
// ---------------------------------------------------------------------------

/// The kind whose capability emits the three datomic interactions of a declared type.
pub(crate) trait Datomizing {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream;
}

/// The kind whose capability wraps a type's conceive and incorporate bodies in the three impl blocks.
trait Wrapping {
    fn wrapped(
        &self,
        scope: &Scope,
        conceive: TokenStream,
        incorporate: TokenStream,
    ) -> TokenStream;
}

impl Wrapping for Identity {
    fn wrapped(
        &self,
        scope: &Scope,
        conceive: TokenStream,
        incorporate: TokenStream,
    ) -> TokenStream {
        let name = self.name.tokens();
        let corporate = self.parameters(scope, true);
        let arguments = self.arguments();
        quote! {
            impl #corporate protos::Conceivable<datomic::Datom> for #name #arguments {
                type Fault = std::convert::Infallible;
                fn conceive(&self) -> Result<datomic::Datom, std::convert::Infallible> {
                    #conceive
                }
            }
            impl #corporate datomic::Datomic for #name #arguments {
                fn incorporate_from(datom: datomic::Datom) -> Result<Self, datomic::Fault> {
                    #incorporate
                }
            }
            impl #corporate protos::Incorporable<#name #arguments> for datomic::Datom {
                type Fault = datomic::Fault;
                fn incorporate(self) -> Result<#name #arguments, datomic::Fault> {
                    <#name #arguments as datomic::Datomic>::incorporate_from(self)
                }
            }
        }
    }
}

impl Datomizing for [Reference] {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let mut borrows = Vec::with_capacity(self.len());
        for index in 0..self.len() {
            let index = syn::Index::from(index);
            borrows.push(Borrow {
                plain: quote! { &self.#index },
                unboxed: quote! { &*self.#index },
            });
        }
        let conceived = self.conceived(scope, owner, &borrows);
        let incorporated = self.incorporated(scope, owner, quote! { Self });
        identity.wrapped(
            scope,
            quote! { Ok(#conceived) },
            quote! {
                match datom {
                    datomic::Datom::Struct(fields) => #incorporated,
                    other => Err(datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Struct, other),
                    )),
                }
            },
        )
    }
}

impl Datomizing for [Variant] {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let mut conceive_arms = Vec::with_capacity(self.len());
        let mut bare_arms = Vec::new();
        let mut headed_arms = Vec::new();
        for variant in self {
            conceive_arms.push(variant.conceive_arm(scope, owner));
            if let Variant::Bare(bare) = variant {
                let head = &bare.0;
                let bare = bare.tokens();
                bare_arms.push(quote! { #head => Ok(Self::#bare), });
            }
            if let Some(arm) = variant.incorporate_arm(scope, owner) {
                headed_arms.push(arm);
            }
        }
        let bare_match = if self.any_bare() {
            quote! {
                datomic::Datom::Bare(symbol) => match symbol.as_str() {
                    #( #bare_arms )*
                    _ => Err(datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::UnknownVariant(symbol),
                    )),
                },
            }
        } else {
            TokenStream::new()
        };
        let headed_match = if headed_arms.is_empty() {
            TokenStream::new()
        } else {
            quote! {
                datomic::Datom::Variant(protos::Head::Bare(head), body) => match head.as_str() {
                    #( #headed_arms )*
                    _ => Err(datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::UnknownVariant(head),
                    )),
                },
            }
        };
        let conceive = if self.is_empty() {
            quote! { match *self {} }
        } else {
            quote! { Ok(match self { #( #conceive_arms )* }) }
        };
        identity.wrapped(
            scope,
            conceive,
            quote! {
                match datom {
                    #bare_match
                    #headed_match
                    other => Err(datomic::Fault::Corporate(
                        vec![],
                        datomic::Problem::Shape(datomic::Expected::Variant, other),
                    )),
                }
            },
        )
    }
}
