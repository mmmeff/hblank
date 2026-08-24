//! Procedural macros for Hblank.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Fields, FnArg, ItemFn, Lit, LitStr, Meta,
    Path, Type, parse_macro_input,
};

#[proc_macro_derive(HblankProps, attributes(hblank))]
pub fn derive_hblank_props(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_hblank_props(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_hblank_props(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let Data::Struct(data) = input.data else {
        return Err(Error::new_spanned(
            name,
            "HblankProps can only be derived for structs with named fields",
        ));
    };
    let Fields::Named(fields) = data.fields else {
        return Err(Error::new_spanned(
            name,
            "HblankProps requires named fields",
        ));
    };

    let mut definitions = Vec::with_capacity(fields.named.len());
    let mut readers = Vec::with_capacity(fields.named.len());
    let mut writers = Vec::with_capacity(fields.named.len());

    for field in fields.named {
        let ident = field
            .ident
            .ok_or_else(|| Error::new_spanned(&field.ty, "HblankProps requires named fields"))?;
        let ty = field.ty;
        let id = ident.to_string();
        let label = field_label(&field.attrs)?.unwrap_or_else(|| humanize(&id));
        let docs = docs(&field.attrs);

        definitions.push(quote! {
            ::hblank::ControlDefinition {
                id: #id,
                label: #label,
                docs: #docs,
                kind: <#ty as ::hblank::__private::ControlField>::KIND,
            }
        });
        readers.push(quote! {
            #id => Some(<#ty as ::hblank::__private::ControlField>::to_control_value(&self.#ident))
        });
        writers.push(quote! {
            #id => <#ty as ::hblank::__private::ControlField>::set_control_value(
                &mut self.#ident,
                #id,
                value,
            )
        });
    }

    Ok(quote! {
        impl ::hblank::HblankProps for #name {
            fn definitions(&self) -> &'static [::hblank::ControlDefinition] {
                const DEFINITIONS: &[::hblank::ControlDefinition] = &[
                    #(#definitions),*
                ];
                DEFINITIONS
            }

            fn control_value(&self, id: &str) -> Option<::hblank::ControlValue> {
                match id {
                    #(#readers,)*
                    _ => None,
                }
            }

            fn set_control(
                &mut self,
                id: &str,
                value: ::hblank::ControlValue,
            ) -> Result<(), ::hblank::ControlError> {
                match id {
                    #(#writers,)*
                    _ => Err(::hblank::ControlError::UnknownControl(id.to_owned())),
                }
            }

            fn clone_box(&self) -> Box<dyn ::hblank::HblankProps> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    })
}

#[proc_macro_derive(HblankEnum, attributes(hblank))]
pub fn derive_hblank_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_hblank_enum(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_hblank_enum(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let Data::Enum(data) = input.data else {
        return Err(Error::new_spanned(
            name,
            "HblankEnum can only be derived for enums",
        ));
    };

    let mut variants = Vec::with_capacity(data.variants.len());
    let mut names = Vec::with_capacity(data.variants.len());
    for variant in data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                variant,
                "HblankEnum variants cannot contain data",
            ));
        }
        let ident = variant.ident;
        let label = field_label(&variant.attrs)?.unwrap_or_else(|| humanize(&ident.to_string()));
        variants.push(ident);
        names.push(label);
    }

    Ok(quote! {
        impl ::hblank::HblankEnum for #name {
            const VARIANTS: &'static [&'static str] = &[#(#names),*];

            fn variant_name(&self) -> &'static str {
                match self {
                    #(Self::#variants => #names),*
                }
            }

            fn from_variant_name(value: &str) -> Option<Self> {
                match value {
                    #(#names => Some(Self::#variants),)*
                    _ => None,
                }
            }
        }
    })
}

#[derive(Default)]
struct ComponentArgs {
    title: Option<LitStr>,
    group: Option<LitStr>,
}

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut component_args = ComponentArgs::default();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("title") {
            component_args.title = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("group") {
            component_args.group = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error("expected one of: title, group"));
        }
        Ok(())
    });
    syn::parse_macro_input!(args with parser);
    let function = parse_macro_input!(input as ItemFn);
    expand_component(component_args, &function)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_component(
    args: ComponentArgs,
    function: &ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    validate_synchronous_non_generic(function, "components")?;
    let props_type = render_props_type(function, "components")?;
    let function_name = &function.sig.ident;
    let module_name = format_ident!("__hblank_component_{}", function_name);
    let function_docs = docs(&function.attrs);
    let title = args.title.unwrap_or_else(|| {
        LitStr::new(&humanize(&function_name.to_string()), function_name.span())
    });
    let group = args
        .group
        .map_or_else(|| quote!(module_path!()), |group| quote!(#group));

    Ok(quote! {
        #function

        #[doc(hidden)]
        pub(crate) mod #module_name {
            use super::*;

            pub(crate) fn id() -> ::std::string::String {
                ::hblank::canonical_source_id(file!(), stringify!(#function_name))
            }

            pub(crate) fn assert_props(_: &#props_type) {}

            pub(crate) fn build() -> ::hblank::ComponentDefinition {
                fn render(
                    props: &dyn ::hblank::HblankProps,
                    window: &mut ::hblank::gpui::Window,
                    cx: &mut ::hblank::gpui::App,
                ) -> ::hblank::gpui::AnyElement {
                    let props = props
                        .as_any()
                        .downcast_ref::<#props_type>()
                        .expect("Hblank component received the wrong props type");
                    ::hblank::gpui::IntoElement::into_any_element(
                        super::#function_name(props, window, cx),
                    )
                }

                ::hblank::ComponentDefinition::new::<#props_type>(
                    ::hblank::ComponentMetadata {
                        id: id(),
                        title: #title,
                        group: #group,
                        docs: #function_docs,
                        source: file!(),
                        line: line!(),
                    },
                    render,
                )
            }
        }

        ::hblank::__private::inventory::submit! {
            ::hblank::ComponentRegistration { build: #module_name::build }
        }
    })
}

#[derive(Default)]
struct FixtureArgs {
    component: Option<Path>,
    title: Option<LitStr>,
}

#[proc_macro_attribute]
pub fn fixture(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut fixture_args = FixtureArgs::default();
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("component") {
            fixture_args.component = Some(meta.value()?.parse()?);
        } else if meta.path.is_ident("title") {
            fixture_args.title = Some(meta.value()?.parse()?);
        } else {
            return Err(meta.error("expected one of: component, title"));
        }
        Ok(())
    });
    syn::parse_macro_input!(args with parser);
    let function = parse_macro_input!(input as ItemFn);
    expand_fixture(fixture_args, &function)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_fixture(args: FixtureArgs, function: &ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    validate_synchronous_non_generic(function, "fixtures")?;
    if !function.sig.inputs.is_empty() {
        return Err(Error::new_spanned(
            &function.sig.inputs,
            "Hblank fixture variants take no arguments and return component props",
        ));
    }
    let component = args.component.ok_or_else(|| {
        Error::new_spanned(
            &function.sig.ident,
            "Hblank fixture variants require component = path::to::component",
        )
    })?;
    let component_module = component_module_path(&component)?;
    let function_name = &function.sig.ident;
    let builder_name = format_ident!("__hblank_build_{}", function_name);
    let function_docs = docs(&function.attrs);
    let title = args.title.unwrap_or_else(|| {
        LitStr::new(&humanize(&function_name.to_string()), function_name.span())
    });

    Ok(quote! {
        #function

        #[doc(hidden)]
        fn #builder_name() -> ::hblank::FixtureRegistrationData {
            let defaults = #function_name();
            #component_module::assert_props(&defaults);
            ::hblank::FixtureRegistrationData::new(
                ::hblank::FixtureRegistrationMetadata {
                    id: ::hblank::canonical_source_id(file!(), stringify!(#function_name)),
                    title: #title,
                    docs: #function_docs,
                    source: file!(),
                    line: line!(),
                },
                #component_module::id(),
                ::std::boxed::Box::new(defaults),
            )
        }

        ::hblank::__private::inventory::submit! {
            ::hblank::FixtureRegistration { build: #builder_name }
        }
    })
}

fn validate_synchronous_non_generic(function: &ItemFn, subject: &str) -> syn::Result<()> {
    if function.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            &function.sig,
            format!("Hblank {subject} must be synchronous"),
        ));
    }
    if !function.sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &function.sig.generics,
            format!("Hblank {subject} cannot be generic"),
        ));
    }
    Ok(())
}

fn render_props_type<'a>(function: &'a ItemFn, subject: &str) -> syn::Result<&'a Type> {
    if function.sig.inputs.len() != 3 {
        return Err(Error::new_spanned(
            &function.sig.inputs,
            format!("Hblank {subject} take exactly (&Props, &mut gpui::Window, &mut gpui::App)"),
        ));
    }
    let first = function
        .sig
        .inputs
        .first()
        .ok_or_else(|| Error::new_spanned(&function.sig, "missing props argument"))?;
    let FnArg::Typed(first) = first else {
        return Err(Error::new_spanned(
            first,
            format!("the first Hblank {subject} argument must be &Props"),
        ));
    };
    let Type::Reference(props_reference) = first.ty.as_ref() else {
        return Err(Error::new_spanned(
            &first.ty,
            format!("the first Hblank {subject} argument must be &Props"),
        ));
    };
    if props_reference.mutability.is_some() {
        return Err(Error::new_spanned(
            &first.ty,
            "component props are immutable; mutate them through harness controls",
        ));
    }
    Ok(props_reference.elem.as_ref())
}

fn component_module_path(component: &Path) -> syn::Result<Path> {
    let mut module = component.clone();
    let Some(last) = module.segments.last_mut() else {
        return Err(Error::new_spanned(
            component,
            "component path cannot be empty",
        ));
    };
    last.ident = format_ident!("__hblank_component_{}", last.ident);
    Ok(module)
}

fn docs(attributes: &[Attribute]) -> String {
    attributes
        .iter()
        .filter_map(|attribute| {
            if !attribute.path().is_ident("doc") {
                return None;
            }
            let Meta::NameValue(name_value) = &attribute.meta else {
                return None;
            };
            let Expr::Lit(ExprLit {
                lit: Lit::Str(value),
                ..
            }) = &name_value.value
            else {
                return None;
            };
            Some(value.value().trim().to_owned())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn field_label(attributes: &[Attribute]) -> syn::Result<Option<String>> {
    let mut label = None;
    for attribute in attributes {
        if !attribute.path().is_ident("hblank") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                let value: LitStr = meta.value()?.parse()?;
                label = Some(value.value());
                Ok(())
            } else {
                Err(meta.error("expected label = \"…\""))
            }
        })?;
    }
    Ok(label)
}

fn humanize(identifier: &str) -> String {
    let mut output = String::with_capacity(identifier.len() + 4);
    let mut previous_lowercase = false;
    for (index, character) in identifier.chars().enumerate() {
        if character == '_' || character == '-' {
            if !output.ends_with(' ') {
                output.push(' ');
            }
            previous_lowercase = false;
            continue;
        }
        if character.is_uppercase() && previous_lowercase {
            output.push(' ');
        }
        if index == 0 {
            output.extend(character.to_uppercase());
        } else {
            output.push(character);
        }
        previous_lowercase = character.is_lowercase();
    }
    output
}
