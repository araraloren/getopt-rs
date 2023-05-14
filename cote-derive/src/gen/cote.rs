use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::DeriveInput;
use syn::GenericParam;
use syn::Generics;
use syn::Token;
use syn::WherePredicate;

use crate::config::Configs;
use crate::config::CoteKind;

use super::gen_default_policy_ty;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::OptUpdate;
use super::APP_POSTFIX;
use super::HELP_OPTION_HELP;
use super::HELP_OPTION_NAME;
use super::HELP_OPTION_SHORT;
use super::POLICY_FWD;
use super::POLICY_PRE;

#[derive(Debug)]
pub struct CoteGenerator<'a> {
    name: TokenStream,

    ident: &'a Ident,

    configs: Configs<CoteKind>,

    generics: &'a Generics,

    has_sub_command: bool,
}

impl<'a> CoteGenerator<'a> {
    pub fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let ident = &input.ident;
        let generics = &input.generics;
        let params = &generics.params;
        let configs = Configs::<CoteKind>::parse_attrs("cote", &input.attrs);
        let name = if let Some(cfg) = configs.find_cfg(CoteKind::Name) {
            let value = cfg.value();

            quote! {
                String::from(#value)
            }
        } else {
            quote! {
                String::from(env!("CARGO_PKG_NAME"))
            }
        };
        // Check the lifetime in type parameters
        for param in params {
            match param {
                GenericParam::Type(_) => {}
                GenericParam::Lifetime(lifetime) => {
                    abort! {
                        input,
                        "Cote not support struct with lifetime `{}`",
                        lifetime.to_token_stream().to_string()
                    }
                }
                GenericParam::Const(const_param) => {
                    abort! {
                        input,
                        "Parsing struct failed: Cote not support const parameter `{:?}`",
                        const_param,
                    }
                }
            }
        }

        Ok(Self {
            name,
            ident,
            configs,
            generics,
            has_sub_command: false,
        })
    }

    pub fn split_for_impl(
        &self,
    ) -> (
        &Punctuated<GenericParam, Token![,]>,
        Option<&Punctuated<WherePredicate, Token![,]>>,
    ) {
        let params = &self.generics.params;
        let where_predicate = self.generics.where_clause.as_ref().map(|v| &v.predicates);

        (params, where_predicate)
    }

    pub fn set_has_sub_command(&mut self, sub_command: bool) -> &mut Self {
        self.has_sub_command = sub_command;
        self
    }

    pub fn has_sub_command(&self) -> bool {
        self.has_sub_command
    }

    pub fn is_process_help(&self) -> bool {
        self.configs.has_cfg(CoteKind::Help) || self.configs.has_cfg(CoteKind::AbortHelp)
    }

    pub fn get_ident(&self) -> &Ident {
        self.ident
    }

    pub fn get_name(&self) -> &TokenStream {
        &self.name
    }

    pub fn gen_new_app_define(&self, ident: &Ident) -> TokenStream {
        quote! {
            #[doc=concat!("Automatic generated by cote-derive for [`", stringify!(#ident), "`].")]
            pub struct #ident<'a, P: Policy>(cote::CoteApp<'a, P>);

            impl<'a, P: Policy> std::ops::Deref for #ident<'a, P> {
                type Target = cote::CoteApp<'a, P>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<'a, P: Policy> std::ops::DerefMut for #ident<'a, P> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        }
    }

    pub fn gen_struct_app_type(&self) -> Ident {
        let ident = self.ident;

        Ident::new(&format!("{}{}", ident, APP_POSTFIX), ident.span())
    }

    pub fn gen_style_settings_for_parser(&self) -> Option<TokenStream> {
        let has_combine = self.configs.has_cfg(CoteKind::Combine);
        let has_embedded = self.configs.has_cfg(CoteKind::EmbeddedPlus);
        let for_combine = has_combine.then_some(quote! {parser.enable_combined();});
        let for_embedded_plus = has_embedded.then_some(quote! {parser.enable_embedded_plus();});
        let for_strict = self.configs.find_cfg(CoteKind::Strict).map(|v| {
            let value = v.value();
            quote! {
                parser.set_strict(#value);
            }
        });

        if for_combine.is_none() && for_embedded_plus.is_none() && for_strict.is_none() {
            None
        } else {
            let mut ret = quote! {};

            ret.extend(for_combine.into_iter());
            ret.extend(for_embedded_plus.into_iter());
            ret.extend(for_strict.into_iter());
            Some(ret)
        }
    }

    pub fn gen_tweak_on_app(&self) -> TokenStream {
        let mut ret = quote! {};

        for config in self.configs.iter() {
            match config.kind() {
                CoteKind::RawCall(method) => {
                    let method = Ident::new(&method, self.ident.span());
                    let args = config.value();

                    ret.extend(quote! {
                        app.#method(#args);
                    });
                }
                _ => {}
            }
        }
        ret
    }

    pub fn gen_sync_running_ctx(&self) -> TokenStream {
        let mut ret = quote! {};

        if self.configs.has_cfg(CoteKind::AbortHelp) {
            ret.extend(quote! {
                if ret.is_err() ||
                    !ret.as_ref().map(|v|v.status()).unwrap_or(true) {
                    let running_ctx = self.inner_parser_mut().app_data_mut::<cote::AppRunningCtx>()?;
                    if sub_parser {
                        running_ctx.set_display_sub_help(true);
                        running_ctx.set_exit_sub(false);
                    }
                    else {
                        running_ctx.set_display_help(true);
                        running_ctx.set_exit(false);
                    }
                }
            })
        }
        if self.configs.has_cfg(CoteKind::Help) {
            ret.extend(quote! {
                let parser = self.inner_parser();
                if parser.find_val::<bool>(#HELP_OPTION_NAME)? == &true {
                    let running_ctx = self.inner_parser_mut().app_data_mut::<cote::AppRunningCtx>()?;
                    if sub_parser {
                        running_ctx.set_display_sub_help(true);
                        running_ctx.set_exit_sub(true);
                    }
                    else {
                        running_ctx.set_display_help(true);
                        running_ctx.set_exit(true);
                    }
                }
            })
        }
        ret
    }

    pub fn gen_help_display_ctx(&self) -> TokenStream {
        let head = if let Some(head_cfg) = self.configs.find_cfg(CoteKind::Head) {
            let value = head_cfg.value();

            quote! {
                String::from(#value)
            }
        } else {
            quote! {
                String::from(env!("CARGO_PKG_DESCRIPTION"))
            }
        };
        let foot = if let Some(foot_cfg) = self.configs.find_cfg(CoteKind::Foot) {
            let value = foot_cfg.value();

            quote! {
                String::from(#value)
            }
        } else {
            quote! {
                format!("Create by {} v{}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"))
            }
        };
        let width = if let Some(head_cfg) = self.configs.find_cfg(CoteKind::HelpWidth) {
            let value = head_cfg.value();

            quote! {
                #value
            }
        } else {
            quote! { 40 }
        };
        let usage_width = if let Some(head_cfg) = self.configs.find_cfg(CoteKind::UsageWidth) {
            let value = head_cfg.value();

            quote! {
                #value
            }
        } else {
            quote! { 10 }
        };
        let name = &self.name;

        quote! {
            cote::HelpDisplayCtx::default()
                .with_name(#name)
                .with_head(#head)
                .with_foot(#foot)
                .with_width(#width)
                .with_usagew(#usage_width)
        }
    }

    pub fn gen_policy_type(&self) -> syn::Result<TokenStream> {
        let policy_ty = self.configs.find_cfg(CoteKind::Policy);

        Ok(if let Some(policy_ty) = policy_ty {
            let policy_name = policy_ty.value().to_token_stream().to_string();
            let policy = gen_default_policy_ty(&policy_name);

            if let Some(policy) = policy {
                policy
            } else {
                policy_ty.value().to_token_stream()
            }
        } else if self.has_sub_command() {
            gen_default_policy_ty(POLICY_PRE).unwrap()
        } else {
            gen_default_policy_ty(POLICY_FWD).unwrap()
        })
    }

    pub fn gen_main_option_update(&self, idx: usize) -> Option<OptUpdate> {
        let ident = self.ident;
        let then = self.configs.find_cfg(CoteKind::Then);
        let on = self.configs.find_cfg(CoteKind::On);
        let fallback = self.configs.find_cfg(CoteKind::Fallback);

        if on.is_some() || fallback.is_some() {
            let ident = gen_option_ident(idx, ident.span());
            let uid = gen_option_uid_ident(idx, ident.span());

            Some((
                Some(quote! {
                    let #ident = {
                        ctor.new_with({
                            let mut config = aopt::prelude::SetCfg::<Set>::default();
                            config.set_name(format!("main_option_{}", #idx));
                            <aopt::opt::Main>::infer_fill_info(&mut config, true);
                            config
                        }).map_err(Into::into)?
                    };
                }),
                Some(quote! {
                    let #uid = set.insert(#ident);
                }),
                Some({
                    if let Some(on_config) = on {
                        let value = on_config.value();

                        if let Some(then_config) = then {
                            let then = then_config.value();

                            quote! {
                                parser.entry(#uid)?.on(#value).then(#then);
                            }
                        } else {
                            quote! {
                                parser.entry(#uid)?.on(#value);
                            }
                        }
                    } else if let Some(fallback_config) = fallback {
                        let value = fallback_config.value();

                        if let Some(then_config) = then {
                            let then = then_config.value();

                            quote! {
                                parser.entry(#uid)?.fallback(#value).then(#then);
                            }
                        } else {
                            quote! {
                                parser.entry(#uid)?.fallback(#value);
                            }
                        }
                    } else {
                        panic!("can not go here")
                    }
                }),
            ))
        } else {
            None
        }
    }

    pub fn gen_help_option_update(&self, idx: usize) -> Option<(Ident, OptUpdate)> {
        let ident = self.ident;
        self.configs.find_cfg(CoteKind::Help).map(|_| {
            let ident = gen_option_ident(idx, ident.span());
            let uid = gen_option_uid_ident(idx, ident.span());

            (
                uid.clone(),
                (
                    Some(quote! {
                        let #ident = {
                            ctor.new_with({
                                let mut config = aopt::prelude::SetCfg::<Set>::default();
                                config.set_name(#HELP_OPTION_NAME);
                                config.add_alias(#HELP_OPTION_SHORT);
                                config.set_help(#HELP_OPTION_HELP);
                                <bool>::infer_fill_info(&mut config, true);
                                config
                            }).map_err(Into::into)?
                        };
                    }),
                    Some(quote! {
                        #[allow(unused)]
                        let #uid = set.insert(#ident);
                    }),
                    None,
                ),
            )
        })
    }
}
