//! Datomization: the datomic machinery every declared type bears.
//!
//! Everything emitted here depends on the shape datom-codec 0.14.0
//! gives its own intrinsics, and on nothing else: a re-pin of
//! datom-codec that changes that shape touches this module alone. For
//! a declared type the machinery is `datom_codec::Datomic` with
//! `incorporate(site: Site<'_>)` and `conceive(&self) -> Datom`. A
//! variant carrying nothing is `Datom::Word`; one carrying data is
//! `Datom::Variant(name, Box<Datom>)`; a struct is `Datom::Struct`
//! of its positions in order.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::generation::{Parametrizing, Positioning, Tokening};
use crate::{Identity, Name, Reference, Scope, Variant};

// ---------------------------------------------------------------------------
// Positions: conceive and incorporate a struct body
// ---------------------------------------------------------------------------

/// The kind whose capabilities yield the datomic machinery of a struct body.
trait Positioned {
    fn conceived(&self) -> TokenStream;
    fn incorporated(&self, scope: &Scope, owner: &Name, constructor: TokenStream) -> TokenStream;
}

impl Positioned for [Reference] {
    fn conceived(&self) -> TokenStream {
        let mut fields = Vec::with_capacity(self.len());
        for index in 0..self.len() {
            let idx = syn::Index::from(index);
            fields.push(quote! { protos::Conceivable::conceive(&self.#idx).expect("infallible datom ascent").1 });
        }
        quote! { datom_codec::Datom::Struct(vec![ #( #fields ),* ]) }
    }

    fn incorporated(&self, scope: &Scope, owner: &Name, constructor: TokenStream) -> TokenStream {
        let arity = Literal::i64_unsuffixed(self.len() as protos::Integer);
        let mut types = Vec::with_capacity(self.len());
        let mut values = Vec::with_capacity(self.len());
        for (index, reference) in self.iter().enumerate() {
            types.push(reference.position(scope, owner));
            values.push(Ident::new(&format!("p{index}"), Span::call_site()));
        }
        quote! {
            let mut p = datom_codec::Sited::positions(site, #arity)?;
            #( let #values: #types = datom_codec::Positional::position(&mut p)?; )*
            Ok(#constructor( #( #values ),* ))
        }
    }
}

// ---------------------------------------------------------------------------
// Variants: conceive and incorporate arms
// ---------------------------------------------------------------------------

/// The kind whose capabilities yield the datomic arms of a variant.
trait Arming {
    fn conceive_arm(&self) -> TokenStream;
    fn incorporate_arm(&self, scope: &Scope, owner: &Name) -> TokenStream;
}

impl Arming for Variant {
    fn conceive_arm(&self) -> TokenStream {
        match self {
            Variant::Bare(name) => {
                let head: &str = &name.0;
                let name = name.tokens();
                quote! { Self::#name => datom_codec::Datom::Word(datom_codec::DatomWord::try_from(protos::Word::try_from(#head).expect("static variant")).expect("stable variant")), }
            }
            Variant::Typed(name, _) | Variant::Enum(name, _) => {
                let head: &str = &name.0;
                let name = name.tokens();
                quote! {
                    Self::#name(p0) => datom_codec::Datom::Variant(
                        protos::Symbol::try_from(#head).expect("static variant"),
                        Box::new(protos::Conceivable::conceive(p0).expect("infallible datom ascent").1),
                    ),
                }
            }
            Variant::Struct(name, positions) => {
                let head: &str = &name.0;
                let name = name.tokens();
                let mut bindings = Vec::with_capacity(positions.len());
                let mut conceived = Vec::with_capacity(positions.len());
                for index in 0..positions.len() {
                    let binding = Ident::new(&format!("p{index}"), Span::call_site());
                    conceived.push(quote! { protos::Conceivable::conceive(#binding).expect("infallible datom ascent").1 });
                    bindings.push(binding);
                }
                quote! {
                    Self::#name( #( #bindings ),* ) => datom_codec::Datom::Variant(
                        protos::Symbol::try_from(#head).expect("static variant"),
                        Box::new(datom_codec::Datom::Struct(vec![ #( #conceived ),* ])),
                    ),
                }
            }
        }
    }

    fn incorporate_arm(&self, scope: &Scope, owner: &Name) -> TokenStream {
        match self {
            Variant::Bare(name) => {
                let head: &str = &name.0;
                let name = name.tokens();
                quote! { #head => { datom_codec::Headed::nothing(v)?; Ok(Self::#name) } }
            }
            Variant::Typed(name, _) | Variant::Enum(name, _) => {
                let head: &str = &name.0;
                let name = name.tokens();
                quote! { #head => Ok(Self::#name(datom_codec::Carrying::body(v)?)), }
            }
            Variant::Struct(name, positions) => {
                let head: &str = &name.0;
                let name = name.tokens();
                let arity = Literal::i64_unsuffixed(positions.len() as protos::Integer);
                let mut types = Vec::with_capacity(positions.len());
                let mut values = Vec::with_capacity(positions.len());
                for (index, reference) in positions.iter().enumerate() {
                    types.push(reference.position(scope, owner));
                    values.push(Ident::new(&format!("p{index}"), Span::call_site()));
                }
                quote! {
                    #head => {
                        let mut p = datom_codec::Headed::positions(v, #arity)?;
                        #( let #values: #types = datom_codec::Positional::position(&mut p)?; )*
                        Ok(Self::#name( #( #values ),* ))
                    }
                }
            }
        }
    }
}

/// The kind whose capabilities tell whether every, or any, variant carries nothing.
pub(crate) trait Uniform {
    fn all_bare(&self) -> bool;
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
}

// ---------------------------------------------------------------------------
// The one interaction a declared type bears
// ---------------------------------------------------------------------------

/// The kind whose capability emits the datomic interaction of a declared type.
pub(crate) trait Datomizing {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream;
}

/// The kind whose capability wraps conceive and incorporate in the impl block.
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
            impl #corporate datom_codec::Datomic for #name #arguments {
                fn incorporate(site: datom_codec::Site<'_>) -> Result<Self, datom_codec::Fault> {
                    #incorporate
                }
            }
            impl #corporate protos::Conceivable<datom_codec::Datom> for #name #arguments {
                type Fault = std::convert::Infallible;
                fn conceive(&self) -> Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
                    Ok(protos::Situated(protos::Situation { extent: protos::Extent(0, 0), children: vec![] }, #conceive))
                }
            }
        }
    }
}

impl Datomizing for [Reference] {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let conceived = self.conceived();
        let incorporated = self.incorporated(scope, owner, quote! { Self });
        identity.wrapped(scope, conceived, incorporated)
    }
}

impl Datomizing for [Variant] {
    fn machinery(&self, scope: &Scope, owner: &Name, identity: &Identity) -> TokenStream {
        let mut conceive_arms = Vec::with_capacity(self.len());
        let mut incorporate_arms = Vec::with_capacity(self.len());
        for variant in self {
            conceive_arms.push(variant.conceive_arm());
            incorporate_arms.push(variant.incorporate_arm(scope, owner));
        }
        let conceive = if self.is_empty() {
            quote! { match *self {} }
        } else {
            quote! { match self { #( #conceive_arms )* } }
        };
        let incorporate = quote! {
            let v = datom_codec::Sited::variant(site)?;
            match v.name {
                #( #incorporate_arms )*
                _ => Err(datom_codec::Headed::reject(&v, datom_codec::Problem::UnknownVariant(protos::Word::try_from(v.name).expect("variant name")))),
            }
        };
        identity.wrapped(scope, conceive, incorporate)
    }
}
