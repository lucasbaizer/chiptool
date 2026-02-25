use std::fmt::Write as _;

use anyhow::Result;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::ir::*;
use crate::util::{self, StringExt};

use super::{sorted, with_defmt_cfg_attr};

pub fn render_device_x(_ir: &IR, d: &Device) -> Result<String> {
    let mut device_x = String::new();
    for i in sorted(&d.interrupts, |i| i.value) {
        writeln!(&mut device_x, "PROVIDE({} = DefaultHandler);", i.name).unwrap();
    }
    Ok(device_x)
}

pub fn render(opts: &super::Options, ir: &IR, d: &Device, path: &str) -> Result<TokenStream> {
    let mut out = TokenStream::new();
    let span = Span::call_site();

    let mut interrupts = TokenStream::new();
    let mut peripherals = TokenStream::new();
    let mut vectors = TokenStream::new();
    let mut names = vec![];

    let mut pos = 0;
    for i in sorted(&d.interrupts, |i| i.value) {
        while pos < i.value {
            vectors.extend(quote!(Vector { _reserved: 0 },));
            pos += 1;
        }
        pos += 1;

        let name_uc = Ident::new(&i.name.to_sanitized_upper_case(), span);
        let description = format!(
            "{} - {}",
            i.value,
            i.description
                .as_ref()
                .map(|s| util::respace(s))
                .as_ref()
                .map(|s| util::escape_brackets(s))
                .unwrap_or_else(|| i.name.clone())
        );

        let value = util::unsuffixed(i.value as u64);

        interrupts.extend(quote! {
            #[doc = #description]
            #name_uc = #value,
        });
        vectors.extend(quote!(Vector { _handler: #name_uc },));
        names.push(name_uc);
    }

    for p in sorted(&d.peripherals, |p| p.base_address) {
        let name = Ident::new(&p.name, span);
        let address = util::hex_usize(p.base_address);
        let doc = util::doc(&p.description);

        if let Some(block_name) = &p.block {
            let _b = ir.blocks.get(block_name);
            let path = util::relative_path(block_name, path);

            peripherals.extend(quote! {
                #doc
                pub const #name: #path = unsafe { #path::from_ptr(#address as _) };
            });
        } else {
            peripherals.extend(quote! {
                #doc
                pub const #name: *mut () = #address as _;
            });
        }
    }

    out.extend(quote!(
        #peripherals
    ));

    Ok(out)
}
