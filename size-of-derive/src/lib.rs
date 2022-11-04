use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::{mem::replace, ops::Not};
use syn::{
    parse_macro_input, parse_quote_spanned, Attribute, Data, DeriveInput, Error, Fields, Generics,
    Index, Lit, Meta, NestedMeta, Path, Result, ReturnType, Type, TypeArray, TypeBareFn, TypePtr,
    TypeReference, TypeSlice, TypeTuple, Variant, WherePredicate,
};

/// Derives the `SizeOf` trait for the given item
///
/// Works on structs and enums, `SizeOf` must be implemented manually for unions
/// unless the `#[size_of(skip_all)]` top-level attribute is used
///
/// Accepted attributes:
/// - `#[size_of(crate = "<crate_path>")]` allows setting the path to the
///   `size_of` crate, this is only allowed at the top level
/// - `#[size_of(skip_all)]` allows skipping all fields or variants of the item,
///   this is only allowed at the top level
/// - `#[size_of(skip)]` skips the current variant or field
/// - `#[size_of(skip_bounds)]` skips emitting trait bounds for the current
///   variant/field, allows compiling things like `struct Foo { bar: Box<Self>
///   }`
///
/// # Examples
///
/// Normal derive usage
///
/// ```rust,ignore
/// #[derive(SizeOf)]
/// struct Foo {
///     bar: Vec<u8>,
///     baz: String,
/// }
/// ```
///
/// Skips collecting the sizes of all children
///
/// ```rust,ignore
/// #[derive(SizeOf)]
/// #[size_of(skip_all)]
/// struct Foo {
///     bar: Vec<u8>,
///     baz: String,
/// }
/// ```
///
/// Skip collecting the size of a specific field/variant
///
/// ```rust,ignore
/// #[derive(SizeOf)]
/// struct Foo {
///     bar: Vec<u8>,
///     #[size_of(skip)]
///     baz: String,
/// }
///
/// #[derive(SizeOf)]
/// enum Bar {
///     #[size_of(skip)]
///     Baz,
///     #[size_of(skip)]
///     Bing(u8),
///     Bong {
///         #[size_of(skip)]
///         boo: String,
///     },
/// }
/// ```
///
/// Skip the bounds of a specific field/variant. The size of the field and all
/// of its children will still be collected, however a `FieldType: SizeOf` bound
/// will not be emitted. This will cause compile errors if `FieldType` doesn't
/// implement `SizeOf`, but it's necessary for recursive types.
///
/// ```rust,ignore
/// #[derive(SizeOf)]
/// struct Foo {
///     #[size_of(skip_bounds)]
///     baz: Vec<Self>,
/// }
/// ```
#[proc_macro_derive(SizeOf, attributes(size_of))]
pub fn size_of_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    size_of_derive_inner(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

// TODO: Emit an error on unrecognized or invalid `#[size_of]` attributes
// TODO: Add `#[size_of(size = <expr>, excess = <expr>, shared = <expr>,
// allocations = <expr>)]`
fn size_of_derive_inner(input: DeriveInput) -> Result<TokenStream> {
    // If the user specified a crate name to replace the default of `::size_of`,
    // use it and otherwise just fall back to `::size_of`
    let crate_path = find_crate_name(&input.attrs)?
        .unwrap_or_else(|| parse_quote_spanned!(Span::mixed_site() => ::size_of));

    // We allow skipping the entire struct for any type (structs, enums and unions)
    if has_skip_all(&input.attrs) {
        let type_name = &input.ident;
        let (intro_generics, fwd_generics, where_clause) = input.generics.split_for_impl();

        return Ok(quote! {
            #[automatically_derived]
            impl #intro_generics #crate_path::SizeOf for #type_name #fwd_generics
                #where_clause
            {
                #[inline]
                fn size_of_children(&self, context: &mut #crate_path::Context) {}
            }
        });

    // `#[size_of(skip)]` isn't accepted at the top level
    } else if let Some(attr) = find_skip(&input.attrs) {
        return Err(Error::new_spanned(
            attr,
            "#[size_of(skip)] is not supported on top-level items, use `#[size_of(skip_all)] instead",
        ));

    // `#[size_of(skip_bounds)]` isn't currently supported at the top level
    // FIXME: Implement this
    } else if let Some(attr) = find_skip_bounds(&input.attrs) {
        return Err(Error::new_spanned(
            attr,
            "#[size_of(skip_bounds)] is not currently supported on top-level items",
        ));
    }

    match &input.data {
        Data::Struct(structure) => {
            let struct_name = &input.ident;

            // Collect info for the fields
            let (field_types, field_sizes) = collect_field_info(&crate_path, &structure.fields);

            // Build the generic bounds for the impl
            let generics = make_generic_bounds(&input, &crate_path, &field_types);
            let (intro_generics, fwd_generics, where_clause) = generics.split_for_impl();

            // Inline empty bodies
            let attr = if field_sizes.is_empty() {
                quote! { #[inline] }
            } else {
                TokenStream::new()
            };

            Ok(quote! {
                #[automatically_derived]
                impl #intro_generics #crate_path::SizeOf for #struct_name #fwd_generics
                    #where_clause
                {
                    #attr
                    fn size_of_children(&self, context: &mut #crate_path::Context) {
                        #(#field_sizes;)*
                    }
                }
            })
        }

        Data::Enum(enumeration) => {
            // Flatten and dedup all types used within all unskipped variants
            let mut variant_types: Vec<_> = enumeration
                .variants
                .iter()
                .flat_map(|variant| {
                    // Do nothing for skipped variants
                    if has_skip(&variant.attrs) {
                        Vec::new()
                    } else {
                        collect_field_info(&crate_path, &variant.fields).0
                    }
                })
                .collect();
            dedup_types(&mut variant_types);

            let match_arms = enumeration.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match &variant.fields {
                    Fields::Named(named) => {
                        let mut needs_ellipses = false;
                        let idents: Vec<_> = named
                            .named
                            .iter()
                            .filter_map(|field| {
                                if has_skip(&field.attrs) {
                                    needs_ellipses = true;
                                    None
                                } else {
                                    Some(field.ident.as_ref().unwrap())
                                }
                            })
                            .collect();
                        let ellipses = needs_ellipses.then(|| quote!(..));

                        let body = named.named.iter().map(|field| {
                            if has_skip(&field.attrs) {
                                TokenStream::new()
                            } else {
                                let ident = &field.ident;
                                let ty = normalize_type(field.ty.clone());
                                quote! {
                                    <#ty as #crate_path::SizeOf>::size_of_children(#ident, context);
                                }
                            }
                        });

                        quote! {
                            Self::#variant_name { #(#idents,)* #ellipses } => {
                                #(#body)*
                            }
                        }
                    }

                    Fields::Unnamed(unnamed) => {
                        let idents = unnamed.unnamed.iter().enumerate().map(|(idx, field)| {
                            if has_skip(&field.attrs) {
                                quote! { _ }
                            } else {
                                let idx = format_ident!("_{idx}");
                                quote! { #idx }
                            }
                        });

                        let body = unnamed.unnamed.iter().enumerate().map(|(idx, field)| {
                            if has_skip(&field.attrs) {
                                TokenStream::new()
                            } else {
                                let ident = format_ident!("_{idx}");
                                let ty = normalize_type(field.ty.clone());
                                quote! {
                                    <#ty as #crate_path::SizeOf>::size_of_children(#ident, context);
                                }
                            }
                        });

                        quote! {
                            Self::#variant_name(#(#idents),*) => {
                                #(#body)*
                            }
                        }
                    }

                    // Do nothing for unit variants
                    Fields::Unit => quote! { Self::#variant_name => {} },
                }
            });

            // Build the generic bounds for the impl
            let generics = make_generic_bounds(&input, &crate_path, &variant_types);
            let (intro_generics, fwd_generics, where_clause) = generics.split_for_impl();
            let tuple_name = &input.ident;

            // Special casing uninhabited enums, references are always considered inhabited
            // so we just dereference the value and match on that
            let body = if enumeration.variants.is_empty() {
                quote! {
                    match *self {}
                }

            // If all variants are empty, skipped or have no unskipped fields
            // then we can emit nothing as the body
            } else if all_variants_empty(enumeration.variants.iter()) {
                TokenStream::new()

            // For inhabited enums we still match on the referenced value
            } else {
                quote! {
                    match self {
                        #(#match_arms)*
                    }
                }
            };

            Ok(quote! {
                #[automatically_derived]
                impl #intro_generics #crate_path::SizeOf for #tuple_name #fwd_generics
                    #where_clause
                {
                    fn size_of_children(&self, context: &mut #crate_path::Context) {
                        #body
                    }
                }
            })
        }

        // We can't automatically derive for unions
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "cannot derive TotalSize on unions, try manually implementing it",
        )),
    }
}

/// Returns `true` if the number of given variants that:
/// - Are not skipped
/// - Have at least one unskipped field
/// Is greater than zero
fn all_variants_empty<'a, I>(mut variants: I) -> bool
where
    I: Iterator<Item = &'a Variant> + 'a,
{
    variants.all(|variant| {
        has_skip(&variant.attrs)
            || variant
                .fields
                .iter()
                .filter(|field| !has_skip(&field.attrs))
                .count()
                == 0
    })
}

fn make_generic_bounds(input: &DeriveInput, crate_path: &Path, field_types: &[Type]) -> Generics {
    let mut generics = input.generics.clone();

    // Add SizeOf bounds to all all unskipped fields of the type
    generics
        .make_where_clause()
        .predicates
        .extend(field_types.iter().map(|field_ty| -> WherePredicate {
            parse_quote_spanned!(Span::mixed_site() => #field_ty: #crate_path::SizeOf)
        }));

    generics
}

fn collect_field_info(crate_path: &Path, fields: &Fields) -> (Vec<Type>, Vec<TokenStream>) {
    let (mut field_types, field_sizes): (Vec<_>, Vec<_>) = match fields {
        // Collect all field types that aren't annotated with a skip attribute
        Fields::Named(fields) => {
            fields
                .named
                .iter()
                .filter_map(|field| {
                    let field_type = normalize_type(field.ty.clone());

                    if has_skip(&field.attrs) || has_skip_bounds(&field.attrs) || is_trivial_bound(&field_type) {
                        None
                    } else {
                        let field_ident = &field.ident;
                        let field_size = quote! {
                            <#field_type as #crate_path::SizeOf>::size_of_children(&self.#field_ident, context)
                        };

                        Some((field_type, field_size))
                    }
                })
                .unzip()
        }

        // We need field indices for tuple structs
        Fields::Unnamed(fields) => {
            fields
                .unnamed
                .iter()
                .enumerate()
                .flat_map(|(idx, field)| has_skip(&field.attrs).not().then(|| {
                    let field_type = normalize_type(field.ty.clone());
                    let idx = Index::from(idx);
                    let field_size = quote! {
                        <#field_type as #crate_path::SizeOf>::size_of_children(&self.#idx, context)
                    };

                    (field_type, field_size)
                }))
                .unzip()
        }

        // We give no (extra) bounds on unit structs
        Fields::Unit => (Vec::new(), Vec::new()),
    };

    // Deduplicate the types we add bounds to so that we reduce the amount of code
    // we generate
    dedup_types(&mut field_types);

    (field_types, field_sizes)
}

/// Returns `true` if the type is trivial elidible
///
/// Currently returns `true` for the following types:
/// - `fn` types, e.g. `fn()`
/// - Raw pointers, e.g. `*const T`
///
// TODO: Can probably extend this to break down nested types like `&T`,
// `&mut T`, `[T; N]` and `[T]` into just `T`, we know that arrays and
// slices implement `SizeOf` when the inner type does. We can probably
// also decompose tuples into their constituent element types
fn is_trivial_bound(ty: &Type) -> bool {
    matches!(ty, Type::BareFn(_) | Type::Ptr(_))
}

/// Normalizes a type
///
/// Currently just removes all parenthesis from around the type
fn normalize_type(mut ty: Type) -> Type {
    normalize_type_mut(&mut ty);
    ty
}

/// Normalizes a type
///
/// Currently just removes all parenthesis from around the type
fn normalize_type_mut(ty: &mut Type) {
    // Unwrap any parenthesises around the type
    while let Type::Paren(inner) = ty {
        *ty = replace(&mut *inner.elem, Type::Verbatim(TokenStream::new()));
    }

    match ty {
        Type::Array(TypeArray { elem, .. })
        | Type::Slice(TypeSlice { elem, .. })
        | Type::Ptr(TypePtr { elem, .. })
        | Type::Reference(TypeReference { elem, .. }) => {
            normalize_type_mut(elem);
        }

        Type::BareFn(TypeBareFn { inputs, output, .. }) => {
            for input in inputs {
                normalize_type_mut(&mut input.ty);
            }

            if let ReturnType::Type(_, ret) = output {
                normalize_type_mut(ret);
            }
        }

        Type::Tuple(TypeTuple { elems, .. }) => {
            for elem in elems {
                normalize_type_mut(elem);
            }
        }

        _ => {}
    }
}

/// Deduplicate all types within the given vec
fn dedup_types(field_types: &mut Vec<Type>) {
    // Ensure that all types are normalized before we start messing with them
    for ty in &mut *field_types {
        normalize_type_mut(ty);
    }

    let mut idx = 0;
    while idx < field_types.len() {
        let current = field_types[idx].clone();

        let mut field_idx = 0;
        field_types.retain(|ty| {
            let should_retain = if field_idx <= idx {
                true
            } else {
                ty != &current
            };
            field_idx += 1;

            should_retain
        });

        idx += 1;
    }
}

/// Returns the path set in the top-level `#[size_of(crate_path = "<path>")]`
/// attribute (if present)
fn find_crate_name(attrs: &[Attribute]) -> Result<Option<Path>> {
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("size_of") {
                for nested in &meta.nested {
                    if let NestedMeta::Meta(Meta::NameValue(pair)) = nested {
                        if pair.path.is_ident("crate") {
                            if let Lit::Str(path) = &pair.lit {
                                return path.parse().map(Some);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Returns `true` if any of the given attributes has an `ident` meta, e.g.
/// `#[size_of(<ident>)]`
fn has_ident_attr(ident: &str, attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("size_of") {
                for nested in &meta.nested {
                    if matches!(nested, NestedMeta::Meta(Meta::Path(path)) if path.is_ident(ident))
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn find_ident_attr(ident: &str, attrs: &[Attribute]) -> Option<NestedMeta> {
    for attr in attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if meta.path.is_ident("size_of") {
                for nested in &meta.nested {
                    if matches!(nested, NestedMeta::Meta(Meta::Path(path)) if path.is_ident(ident))
                    {
                        return Some(nested.clone());
                    }
                }
            }
        }
    }

    None
}

/// Returns `true` if any of the given attributes is `#[size_of(skip)]`
fn has_skip_all(attrs: &[Attribute]) -> bool {
    has_ident_attr("skip_all", attrs)
}

/// Returns `true` if any of the given attributes is `#[size_of(skip)]`
fn has_skip(attrs: &[Attribute]) -> bool {
    has_ident_attr("skip", attrs)
}

/// Returns the `#[size_of(skip)]` meta if it exists
fn find_skip(attrs: &[Attribute]) -> Option<NestedMeta> {
    find_ident_attr("skip", attrs)
}

/// Returns `true` if any of the given attributes is `#[size_of(skip_bounds)]`
fn has_skip_bounds(attrs: &[Attribute]) -> bool {
    has_ident_attr("skip_bounds", attrs)
}

/// Returns the `#[size_of(skip_bounds)]` meta if it exists
fn find_skip_bounds(attrs: &[Attribute]) -> Option<NestedMeta> {
    find_ident_attr("skip_bounds", attrs)
}
