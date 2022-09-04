use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::{parenthesized, parse2, token, Attribute, Expr, Ident, LitChar, LitStr, Result};

use crate::field::{as_long_name, as_short_name, fill_in_name, ConstrName, Doc, Name};
use crate::top::Inner;
use crate::utils::LineIter;

#[derive(Debug)]
pub struct ReqFlag {
    value: ConstrName,
    naming: Vec<Name>,
    env: Vec<Expr>,
    help: Option<String>,
    is_hidden: bool,
    is_default: bool,
}

impl ReqFlag {
    pub fn make2(value: ConstrName, mut inner: Inner) -> Result<Self> {
        if inner.longs.is_empty() && inner.shorts.is_empty() {
            if value.constr.to_string().chars().nth(1).is_some() {
                inner.longs.push(as_long_name(&value.constr));
            } else {
                inner.shorts.push(as_short_name(&value.constr));
            }
        }

        let longs = inner.longs.into_iter().map(Name::Long);
        let short = inner.shorts.into_iter().map(Name::Short);
        Ok(ReqFlag {
            value,
            naming: longs.chain(short).collect::<Vec<_>>(),
            env: inner.envs,
            help: inner.help.first().cloned(), // TODO
            is_hidden: inner.is_hidden,
            is_default: inner.is_default,
        })
    }
    /*
    pub fn make(value: ConstrName, attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = ReqFlag {
            value,
            naming: Vec::new(),
            env: Vec::new(),
            help: None,
            is_hidden: false,
            is_default: false,
        };
        let mut help = Vec::new();
        for attr in attrs {
            if attr.path.is_ident("doc") {
                help.push(parse2::<Doc>(attr.tokens)?.0);
            } else if attr.path.is_ident("bpaf") {
                attr.parse_args_with(|input: ParseStream| loop {
                    if input.is_empty() {
                        break Ok(());
                    }
                    let input_copy = input.fork();
                    let keyword = input.parse::<Ident>()?;

                    let content;
                    if keyword == "long" {
                        res.naming.push(if input.peek(token::Paren) {
                            let _ = parenthesized!(content in input);
                            Name::Long(content.parse::<LitStr>()?)
                        } else {
                            Name::Long(as_long_name(&res.value.constr))
                        })
                    } else if keyword == "short" {
                        res.naming.push(if input.peek(token::Paren) {
                            let _ = parenthesized!(content in input);
                            Name::Short(content.parse::<LitChar>()?)
                        } else {
                            Name::Short(as_short_name(&res.value.constr))
                        })
                    } else if keyword == "env" {
                        let _ = parenthesized!(content in input);
                        let env = content.parse::<Expr>()?;
                        res.env = vec![env];
                    } else if keyword == "hide" {
                        res.is_hidden = true;
                    } else if keyword == "default" {
                        res.is_default = true;
                    } else {
                        break Err(
                            input_copy.error("Not a valid enum singleton constructor attribute")
                        );
                    };
                    if !input.is_empty() {
                        input.parse::<token::Comma>()?;
                    }
                })?;
            } else {
                unreachable!("Shouldn't get any attributes other than bpaf and doc")
            }
        }
        res.help = LineIter::from(&help[..]).next();
        fill_in_name(&res.value.constr, &mut res.naming);
        Ok(res)
    } */
}

impl ToTokens for ReqFlag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut first = true;
        for naming in &self.naming {
            if first {
                quote!(::bpaf::).to_tokens(tokens);
            } else {
                quote!(.).to_tokens(tokens);
            }
            naming.to_tokens(tokens);
            first = false;
        }
        for env in &self.env {
            quote!(.env(#env)).to_tokens(tokens);
        }
        if let Some(help) = &self.help {
            // help only makes sense for named things
            if !first {
                quote!(.help(#help)).to_tokens(tokens);
            }
        }
        let value = &self.value;

        if self.is_default {
            quote!(.flag(#value, #value)).to_tokens(tokens);
        } else {
            quote!(.req_flag(#value)).to_tokens(tokens);
        }
        if self.is_hidden {
            quote!(.hide()).to_tokens(tokens);
        }
    }
}
