use proc_macro2::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Field;
use syn::Index;
use syn::Lifetime;
use syn::Lit;
use syn::Type;

use crate::config::Configs;
use crate::config::SubKind;

use super::filter_comment_doc;
use super::gen_default_policy_ty;
use super::gen_option_ident;
use super::gen_option_uid_ident;
use super::gen_subapp_without_option;
use super::gen_ty_without_option;
use super::OptUpdate;
use super::HELP_OPTION_NAME;
use super::POLICY_FWD;

#[derive(Debug)]
pub struct SubGenerator<'a> {
    sub_id: usize,

    #[allow(unused)]
    field_ty: &'a Type,

    name: TokenStream,

    ident: Option<&'a Ident>,

    docs: Vec<Lit>,

    configs: Configs<SubKind>,

    without_option_ty: Type,
}

impl<'a> SubGenerator<'a> {
    pub fn new(field: &'a Field) -> syn::Result<Self> {
        let field_ty = &field.ty;
        let ident = field.ident.as_ref();
        let attrs = &field.attrs;
        let docs = filter_comment_doc(attrs);
        let configs = Configs::parse_attrs("sub", attrs);
        let without_option_ty = gen_ty_without_option(field_ty)?;
        let name = {
            if let Some(cfg) = configs.find_cfg(SubKind::Name) {
                cfg.value().to_token_stream()
            } else {
                ident
                    .unwrap_or_else(|| {
                        abort! {
                            ident,
                            "`arg` or `sub` not support empty field name"
                        }
                    })
                    .to_string()
                    .to_token_stream()
            }
        };

        Ok(Self {
            sub_id: 0,
            field_ty,
            name,
            ident,
            docs,
            configs,
            without_option_ty,
        })
    }

    pub fn with_sub_id(mut self, id: usize) -> Self {
        self.sub_id = id;
        self
    }

    pub fn name(&self) -> &TokenStream {
        &self.name
    }

    pub fn get_sub_id(&self) -> usize {
        self.sub_id
    }

    pub fn get_without_option_type(&self) -> &Type {
        &self.without_option_ty
    }

    pub fn gen_policy_type(&self) -> syn::Result<TokenStream> {
        let policy_ty = self.configs.find_cfg(SubKind::Policy);

        Ok(if let Some(policy_ty) = policy_ty {
            let policy_name = policy_ty.value().to_token_stream().to_string();
            let policy = gen_default_policy_ty(&policy_name);

            if let Some(policy) = policy {
                policy
            } else {
                policy_ty.value().to_token_stream()
            }
        } else {
            gen_default_policy_ty(POLICY_FWD).unwrap()
        })
    }

    pub fn gen_app_type(
        &self,
        lifetime: Option<Lifetime>,
        policy_ty: &TokenStream,
    ) -> syn::Result<TokenStream> {
        let sub_struct_app_ty = self.gen_struct_app_type()?;

        if let Some(lifetime) = lifetime {
            Ok(quote! {
                #sub_struct_app_ty<#lifetime, #policy_ty>
            })
        } else {
            Ok(quote! {
                #sub_struct_app_ty<'_, #policy_ty>
            })
        }
    }

    pub fn gen_field_extract(&self) -> syn::Result<(bool, TokenStream)> {
        let is_refopt = self.configs.find_cfg(SubKind::Ref).is_some();
        let is_mutopt = self.configs.find_cfg(SubKind::Mut).is_some();
        let ident = self.ident;
        let name = &self.name;

        if is_refopt && is_mutopt {
            abort! {
                ident,
                "can not set both mut and ref on arg"
            }
        } else if is_refopt {
            Ok((
                true,
                quote! {
                    #ident: set.find_val(#name).ok(),
                },
            ))
        } else {
            Ok((
                false,
                quote! {
                    #ident: set.take_val(#name).ok(),
                },
            ))
        }
    }

    pub fn gen_option_update(
        &self,
        idx: usize,
        sub_parser_tuple_ty: &TokenStream,
        is_process_help: bool,
        help_uid: Option<&Ident>,
    ) -> syn::Result<OptUpdate> {
        let ident = gen_option_ident(idx, self.ident.span());
        let uid = gen_option_uid_ident(idx, self.ident.span());

        Ok((
            Some(self.gen_option_config_new(&ident)?),
            Some(self.gen_option_config_insert(&uid, &ident)),
            Some(self.gen_option_handler_insert(
                &uid,
                sub_parser_tuple_ty,
                is_process_help,
                help_uid,
            )?),
        ))
    }

    pub fn gen_option_config_insert(&self, uid: &Ident, ident: &Ident) -> TokenStream {
        quote! {
            let #uid = set.insert(#ident);
        }
    }

    pub fn gen_option_config_new(&self, ident: &Ident) -> syn::Result<TokenStream> {
        let name = &self.name;
        let mut codes = vec![];
        let mut config = quote! {
            let mut config = aopt::prelude::SetCfg::<P::Set>::default();
            config.set_name(#name);
        };

        for cfg in self.configs.iter() {
            codes.push(match cfg.kind() {
                SubKind::Alias => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.add_alias(#token);
                    }
                }
                SubKind::Hint => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.set_hint(#token);
                    }
                }
                SubKind::Help => {
                    let token = cfg.value().to_token_stream();

                    quote! {
                        config.set_help(#token);
                    }
                }
                _ => {
                    quote! {}
                }
            })
        }
        if !self.configs.has_cfg(SubKind::Help) && !self.docs.is_empty() {
            let mut code = quote! {
                let mut message = String::default();
            };
            let mut iter = self.docs.iter();

            if let Some(doc) = iter.next() {
                code.extend(quote! {
                    message.push_str(#doc.trim());
                });
            }
            for doc in iter {
                code.extend(quote! {
                    message.push_str(" ");
                    message.push_str(#doc.trim());
                });
            }
            codes.push(quote! {
                config.set_help({ #code message });
            })
        }
        codes.push(quote! {
            aopt::opt::Cmd::infer_fill_info(&mut config, true);
            config
        });
        config.extend(codes.into_iter());

        Ok(quote! {
            let #ident = {
                ctor.new_with({ #config }).map_err(Into::into)?
            };
        })
    }

    pub fn gen_option_handler_insert(
        &self,
        uid: &Ident,
        sub_parser_tuple_ty: &TokenStream,
        is_process_help: bool,
        help_uid: Option<&Ident>,
    ) -> syn::Result<TokenStream> {
        let without_option_ty = &self.without_option_ty;
        let sub_id = self.get_sub_id();
        let sub_id = Index::from(sub_id);
        let pass_help_to_next = if is_process_help {
            let help_uid = help_uid.unwrap_or_else(|| {
                abort! {
                    uid,
                    "Failed generate help handler, found None of help uid"
                }
            });
            quote! {
                if let Ok(value) = set.opt(#help_uid)?.val::<bool>() {
                    if *value {
                        // pass a fake flag to next sub command
                        args.push(aopt::RawVal::from(#HELP_OPTION_NAME));
                    }
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            parser.entry(#uid)?.on(
                move |set: &mut P::Set, ser: &mut P::Ser, args: aopt::prelude::ctx::Args, index: aopt::prelude::ctx::Index| {
                    use std::ops::Deref;

                    let mut args = args.deref().clone().into_inner();
                    let mut next_ctx = cote::AppRunningCtx::default();
                    let current_cmd = args.remove(*index.deref());
                    let current_cmd = current_cmd.get_str();

                    next_ctx.add_name(current_cmd.ok_or_else(||
                        aopt::Error::raise_error(format!("can not convert `{:?}` to str", current_cmd)))?.to_owned()
                    );
                    #pass_help_to_next

                    let args = aopt::ARef::new(aopt::prelude::Args::from_vec(args));
                    let mut sub_app = &mut ser.sve_val_mut::<#sub_parser_tuple_ty>()?.#sub_id;

                    sub_app.set_running_ctx(next_ctx)?;
                    let parser = sub_app.inner_parser_mut();

                    // initialize the option value
                    parser.init()?;
                    let ret = parser.parse(args).map_err(Into::into);

                    sub_app.sync_running_ctx(&ret, true)?;
                    let ret = ret?;
                    let ret_ctx = ret.ctx();
                    let ret_args = ret_ctx.args();
                    let ret_inner_ctx = ret_ctx.inner_ctx().ok();
                    let ret_e = ret.failure();

                    if ret.status() {
                        let running_ctx = sub_app.take_running_ctx()?;

                        ser.sve_val_mut::<cote::AppRunningCtx>()?.append_ctx(running_ctx);
                        let mut sub_app = &mut ser.sve_val_mut::<#sub_parser_tuple_ty>()?.#sub_id;

                        Ok(<#without_option_ty>::try_extract(sub_app.inner_parser_mut().optset_mut()).ok())
                    }
                    else {
                        // return failure with more detail error message
                        Err(aopt::Error::raise_failure(
                            format!("Failed at command `{}` with `{}`: {}, inner_ctx = {}",
                            stringify!(#without_option_ty), ret_args, ret_e.display(),
                            if let Some(inner_ctx) = ret_inner_ctx {
                                format!("{}", inner_ctx)
                            } else {
                                format!("None")
                            }
                        )))
                    }
                }
            );
        })
    }

    pub fn gen_struct_app_type(&self) -> syn::Result<Ident> {
        let ident = gen_subapp_without_option(&self.without_option_ty)?;

        Ok(Ident::new(&format!("{}App", ident), ident.span()))
    }

    pub fn gen_sub_help_context(&self) -> syn::Result<TokenStream> {
        let idx = self.get_sub_id();
        let idx = Index::from(idx);
        let mut ret = quote! { let mut context = sub_parser_tuple.#idx.gen_help_display_ctx(); };

        if let Some(head_cfg) = self.configs.find_cfg(SubKind::Head) {
            let value = head_cfg.value();

            ret.extend(quote! {
                context = context.with_head(String::from(#value));
            })
        }
        if let Some(foot_cfg) = self.configs.find_cfg(SubKind::Foot) {
            let value = foot_cfg.value();

            ret.extend(quote! {
                context = context.with_foot(String::from(#value));
            })
        }
        ret.extend(quote! { context });
        Ok(ret)
    }
}
