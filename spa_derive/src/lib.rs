use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index,
};

#[proc_macro_derive(PodSerialize)]
pub fn derive_podserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: PodSerialize` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to sum up the heap size of each field.
    let fields = field_serialize(&input.data);

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics spa::serialize::PodSerialize for #name #ty_generics #where_clause {
            fn serialize<O: std::io::Write + std::io::Seek>(
                &self,
                serializer: PodSerializer<O>,
            ) -> Result<spa::serialize::SerializeSuccess<O>, cookie_factory::GenError> {
                let mut struct_serializer = serializer.serialize_struct()?;
                #fields
                struct_serializer.end()
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(PodDeserialize)]
pub fn derive_poddeserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Add a bound `T: PodSerialize` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to sum up the heap size of each field.
    let fields = field_deserialize(&input.data);
    let visitor_name = proc_macro2::Ident::new(
        (name.to_string() + "Visitor").as_str(),
        proc_macro2::Span::call_site(),
    );
    let expanded = quote! {
        // The generated impl.
        impl<'de> #impl_generics spa::deserialize::PodDeserialize<'de> for #name #ty_generics #where_clause {
            fn deserialize(
                deserializer: spa::deserialize::PodDeserializer<'de>,
            ) -> Result<
                (Self, spa::deserialize::DeserializeSuccess<'de>),
                spa::deserialize::DeserializeError<&'de [u8]>,
            >
            where
                Self: Sized,
            {
                struct #visitor_name;

                impl<'de> spa::deserialize::Visitor<'de> for #visitor_name {
                    type Value = #name;
                    type ArrayElem = std::convert::Infallible;
                    fn visit_struct(
                        &self,
                        struct_deserializer: &mut spa::deserialize::StructPodDeserializer<'de>,
                    ) -> Result<Self::Value, spa::deserialize::DeserializeError<&'de [u8]>> {
                        Ok(#name {
                            #fields
                        })
                    }
                }
                deserializer.deserialize_struct(#visitor_name)
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: PodSerialize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(spa::serialize::PodSerialize));
        }
    }
    generics
}

// Generate an expression to serialize each field of a struct
fn field_serialize(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     struct_serializer.serialize_field(&self.name);
                    //     struct_serializer.serialize_field(&self.name2);
                    //
                    // but using fully qualified function call syntax.
                    let serialize_fields = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            struct_serializer.serialize_field(&self.#name)?;
                        }
                    });
                    quote! {
                        #(#serialize_fields)*
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //     struct_serializer.serialize_field(&self.0);
                    //     struct_serializer.serialize_field(&self.1);
                    //
                    // but using fully qualified function call syntax.
                    let serialize_fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        quote_spanned! {f.span()=>
                            struct_serializer.serialize_field(&self.#index)?;
                        }
                    });
                    quote! {
                        #(#serialize_fields)*
                    }
                }
                Fields::Unit => {
                    // Unit struct cannot be done currently
                    unimplemented!()
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Generate an expression to serialize each field of a struct
fn field_deserialize(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //     struct_serializer.serialize_field(&self.name);
                    //     struct_serializer.serialize_field(&self.name2);
                    //
                    // but using fully qualified function call syntax.
                    let serialize_fields = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            #name: struct_deserializer
                                .deserialize_field()?
                                .expect("Input has too few fields"),
                        }
                    });
                    quote! {
                        #(#serialize_fields)*
                    }
                }
                Fields::Unnamed(ref _fields) => {
                    // Expands to an expression like
                    //
                    //     struct_serializer.serialize_field(&self.0);
                    //     struct_serializer.serialize_field(&self.1);
                    //
                    // but using fully qualified function call syntax.
                    unimplemented!("Cannot do unnamed fields yet")
                    // let serialize_fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    //     let index = Index::from(i);
                    //     quote_spanned! {f.span()=>
                    //         #name: struct_deserializer
                    //             .deserialize_field()?
                    //             .expect("Input has too few fields"),
                    //     }
                    // });
                    // quote! {
                    //     #(#serialize_fields)*
                    // }
                }
                Fields::Unit => {
                    // Unit struct cannot be done currently
                    unimplemented!()
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
