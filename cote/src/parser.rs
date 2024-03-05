use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;

use aopt::ctx::HandlerEntry;
use aopt::prelude::Args;
use aopt::prelude::ConfigBuild;
use aopt::prelude::ConfigValue;
use aopt::prelude::ErasedTy;
use aopt::prelude::Extract;
use aopt::prelude::Handler;
use aopt::prelude::Information;
use aopt::prelude::Invoker;
use aopt::prelude::Opt;
use aopt::prelude::OptParser;
use aopt::prelude::OptValidator;
use aopt::prelude::Policy;
use aopt::prelude::PolicyParser;
use aopt::prelude::SetCfg;
use aopt::prelude::SetOpt;
use aopt::raise_error;
use aopt::ser::ServicesValExt;
use aopt::set::SetValueFindExt;
use aopt::ARef;
use aopt::Error;
use aopt::RawVal;
use aopt::Uid;

use crate::prelude::RunningCtx;
use crate::ExtractFromSetDerive;
use crate::HelpContext;

#[derive(Debug)]
pub struct Parser<'a, Set, Ser> {
    name: String,
    set: Set,
    ser: Option<Ser>,
    inv: Option<Invoker<'a, Self, Ser>>,
    sub_parsers: Vec<Self>,
}

impl<'a, Set, Ser> Default for Parser<'a, Set, Ser>
where
    Set: Default,
    Ser: Default,
{
    fn default() -> Self {
        Self {
            name: String::from("CoteParser"),
            set: Default::default(),
            ser: Some(Ser::default()),
            inv: Some(Invoker::default()),
            sub_parsers: Default::default(),
        }
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser> {
    pub fn new(name: impl Into<String>, set: Set) -> Self {
        Self {
            name: name.into(),
            set,
            ser: None,
            inv: None,
            sub_parsers: vec![],
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn optset(&self) -> &Set {
        &self.set
    }

    pub fn optset_mut(&mut self) -> &mut Set {
        &mut self.set
    }

    pub fn set_optset(&mut self, set: Set) -> &mut Self {
        self.set = set;
        self
    }

    pub fn service(&self) -> &Ser {
        assert!(self.ser.is_some());
        self.ser.as_ref().unwrap()
    }

    pub fn service_mut(&mut self) -> &mut Ser {
        assert!(self.ser.is_some());
        self.ser.as_mut().unwrap()
    }

    pub fn set_service(&mut self, ser: Ser) -> &mut Self {
        self.ser = Some(ser);
        self
    }

    pub fn invoker(&self) -> &Invoker<'a, Self, Ser> {
        assert!(self.inv.is_some());
        self.inv.as_ref().unwrap()
    }

    pub fn invoker_mut(&mut self) -> &mut Invoker<'a, Self, Ser> {
        assert!(self.inv.is_some());
        self.inv.as_mut().unwrap()
    }

    pub fn set_invoker(&mut self, inv: Invoker<'a, Self, Ser>) -> &mut Self {
        self.inv = Some(inv);
        self
    }

    pub fn parsers(&self) -> &[Self] {
        &self.sub_parsers
    }

    pub fn parsers_mut(&mut self) -> &mut [Self] {
        &mut self.sub_parsers
    }

    pub fn set_parsers(&mut self, parsers: Vec<Self>) -> &mut Self {
        self.sub_parsers = parsers;
        self
    }

    pub fn parser(&self, id: usize) -> Result<&Self, Error> {
        self.sub_parsers
            .get(id)
            .ok_or_else(|| aopt::raise_error!("Can not find parser at index {}", id))
    }

    pub fn parser_mut(&mut self, id: usize) -> Result<&mut Self, Error> {
        self.sub_parsers
            .get_mut(id)
            .ok_or_else(|| aopt::raise_error!("Can not find parser at index {}", id))
    }

    pub fn find_parser(&self, name: &str) -> Result<&Self, Error> {
        self.sub_parsers
            .iter()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::raise_error!("Can not find parser named {}", name))
    }

    pub fn find_parser_mut(&mut self, name: &str) -> Result<&mut Self, Error> {
        self.sub_parsers
            .iter_mut()
            .find(|v| v.name() == name)
            .ok_or_else(|| aopt::raise_error!("Can not find parser named {}", name))
    }

    pub fn add_parser(&mut self, parser: Self) -> &mut Self {
        self.sub_parsers.push(parser);
        self
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Ser: ServicesValExt,
{
    pub fn rctx(&self) -> Result<&RunningCtx, aopt::Error> {
        self.service().sve_val()
    }

    pub fn rctx_mut(&mut self) -> Result<&mut RunningCtx, aopt::Error> {
        self.service_mut().sve_val_mut()
    }

    pub fn set_rctx(&mut self, ctx: RunningCtx) -> &mut Self {
        self.service_mut().sve_insert(ctx);
        self
    }

    pub fn take_rctx(&mut self) -> Result<RunningCtx, aopt::Error> {
        Ok(std::mem::take(self.rctx_mut()?))
    }
}

impl<'a, Set, Ser> Deref for Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    type Target = Set;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<'a, Set, Ser> DerefMut for Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    /// Reset the option set.
    pub fn reset(&mut self) -> Result<&mut Self, Error> {
        self.optset_mut().reset();
        Ok(self)
    }

    /// Call the [`init`](crate::Opt::init) of [`Opt`](crate::Opt) initialize the option value.
    pub fn init(&mut self) -> Result<(), Error> {
        let optset = self.optset_mut();

        for opt in optset.iter_mut() {
            opt.init()?;
        }
        Ok(())
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    #[cfg(feature = "sync")]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self, Ser>, Self, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Self, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'a,
        A: Extract<Self, Ser, Error = Error> + Send + Sync + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }

    #[cfg(not(feature = "sync"))]
    #[allow(clippy::type_complexity)]
    pub fn entry<A, O, H>(
        &mut self,
        uid: Uid,
    ) -> Result<HandlerEntry<'a, '_, Invoker<'a, Self, Ser>, Self, Ser, H, A, O>, Error>
    where
        O: ErasedTy,
        H: Handler<Self, Ser, A, Output = Option<O>, Error = Error> + 'a,
        A: Extract<Self, Ser, Error = Error> + 'a,
    {
        Ok(HandlerEntry::new(self.inv.as_mut().unwrap(), uid))
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: crate::Set,
    Ser: ServicesValExt,
{
    pub fn app_data<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.service().sve_val()
    }

    pub fn app_data_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.service_mut().sve_val_mut()
    }

    pub fn set_app_data<T: ErasedTy>(&mut self, val: T) -> Result<Option<T>, Error> {
        Ok(self.service_mut().sve_insert(val))
    }
}

impl<'a, 'b, Set, Ser> Parser<'a, Set, Ser>
where
    'a: 'b,
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn extract_type<T>(&'b mut self) -> Result<T, Error>
    where
        T: ExtractFromSetDerive<'b, Set>,
    {
        let set = self.optset_mut();

        T::try_extract(set)
    }
}

impl<'a, Set, Ser> aopt::set::Set for Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    type Ctor = Set::Ctor;

    fn register(&mut self, ctor: Self::Ctor) -> Option<Self::Ctor> {
        Set::register(&mut self.set, ctor)
    }

    fn ctor_iter(&self) -> std::slice::Iter<'_, Self::Ctor> {
        Set::ctor_iter(&self.set)
    }

    fn ctor_iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Ctor> {
        Set::ctor_iter_mut(&mut self.set)
    }

    fn reset(&mut self) {
        Set::reset(&mut self.set)
    }

    fn len(&self) -> usize {
        Set::len(&self.set)
    }

    fn iter(&self) -> std::slice::Iter<'_, SetOpt<Self>> {
        Set::iter(&self.set)
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, SetOpt<Self>> {
        Set::iter_mut(&mut self.set)
    }

    fn insert(&mut self, opt: SetOpt<Self>) -> Uid {
        Set::insert(&mut self.set, opt)
    }
}

impl<'a, Set, Ser> OptParser for Parser<'a, Set, Ser>
where
    Set: OptParser,
{
    type Output = Set::Output;

    type Error = Set::Error;

    fn parse_opt(&self, pattern: &str) -> Result<Self::Output, Self::Error> {
        OptParser::parse_opt(&self.set, pattern)
    }
}

impl<'a, Set, Ser> OptValidator for Parser<'a, Set, Ser>
where
    Set: OptValidator,
{
    type Error = Set::Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        OptValidator::check(&mut self.set, name)
    }

    fn split<'b>(&self, name: &'b str) -> Result<(&'b str, &'b str), Self::Error> {
        OptValidator::split(&self.set, name)
    }
}

impl<'a, Set, Ser> SetValueFindExt for Parser<'a, Set, Ser>
where
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    fn find_uid(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<Uid, Error> {
        SetValueFindExt::find_uid(&self.set, cb)
    }

    fn find_opt(&self, cb: impl ConfigBuild<SetCfg<Self>>) -> Result<&SetOpt<Self>, Error> {
        SetValueFindExt::find_opt(&self.set, cb)
    }

    fn find_opt_mut(
        &mut self,
        cb: impl ConfigBuild<SetCfg<Self>>,
    ) -> Result<&mut SetOpt<Self>, Error> {
        SetValueFindExt::find_opt_mut(&mut self.set, cb)
    }
}

impl<'a, P, Set, Ser> PolicyParser<P> for Parser<'a, Set, Ser>
where
    Set: aopt::set::Set + OptParser + OptValidator,
    P: Policy<Set = Self, Ser = Ser, Inv<'a> = Invoker<'a, Self, Ser>>,
{
    type Error = Error;

    fn parse_policy(
        &mut self,
        args: ARef<Args>,
        policy: &mut P,
    ) -> Result<<P as Policy>::Ret, Self::Error> {
        assert!(self.inv.is_some());
        assert!(self.ser.is_some());

        self.init()?;

        let mut inv = self.inv.take().unwrap();
        let mut ser = self.ser.take().unwrap();

        let ret = policy
            .parse(self, &mut inv, &mut ser, args)
            .map_err(Into::into);

        self.inv = Some(inv);
        self.ser = Some(ser);

        ret
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: aopt::set::Set + OptValidator + OptParser,
    <Set as OptParser>::Output: Information,
    SetCfg<Set>: ConfigValue + Default,
{
    /// Running function after parsing.
    ///
    /// # Example
    ///
    ///```rust
    /// # use aopt::Error;
    /// # use cote::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<ASet, ASer>::default().with_name("example");
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser.run_mut_with(
    ///         ["-a", "-b", "42"].into_iter(),
    ///         &mut policy,
    ///         |ret, parser| {
    ///             if ret.status() {
    ///                 assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                 assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///     println!("{} running over!", parser.name());
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn run_mut_with<'c, 'b, I, R, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let ret = self.parse_policy(aopt::ARef::new(Args::from(args)), policy)?;

        r(ret, self)
    }

    /// Call [`run_mut_with`](Parser::run_mut_with) with default arguments [`args()`](std::env::args).
    pub fn run_mut<'c, 'b, R, F, P>(&'c mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        'c: 'b,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &'b mut Self) -> Result<R, Error>,
    {
        let args: Vec<aopt::raw::RawVal> = Args::from_env().into();
        self.run_mut_with(args.into_iter(), policy, r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use cote::*;
    /// #
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<ASet, ASer>::default().with_name("example");
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser
    ///         .run_async_mut_with(
    ///             ["-a", "-b", "42"].into_iter(),
    ///             &mut policy,
    ///             |ret, parser| async move {
    ///                 if ret.status() {
    ///                     assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                     assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///                 }
    ///                 Ok(())
    ///             },
    ///         )
    ///         .await?;
    ///     println!("{} running over!", parser.name());
    /// # Ok(())
    /// # }
    ///```
    pub async fn run_async_mut_with<'c, 'b, I, R, FUT, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        let args = iter.map(|v| v.into());
        let async_ret;

        match self.parse_policy(aopt::ARef::new(Args::from(args)), policy) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e);
            }
        }
        async_ret
    }

    /// Call [`run_async_mut_with`](Self::run_async_mut_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async_mut<'c, 'b, R, FUT, F, P>(
        &'c mut self,
        policy: &mut P,
        r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b mut Self) -> FUT,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        let args: Vec<aopt::raw::RawVal> = Args::from_env().into();
        self.run_async_mut_with(args.into_iter(), policy, r).await
    }

    /// Running function after parsing.
    ///
    /// # Example
    ///
    ///```rust
    /// # use aopt::Error;
    /// # use cote::*;
    /// #
    /// # fn main() -> Result<(), Error> {
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<ASet, ASer>::default().with_name("example");
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser.run_with(
    ///         ["-a", "-b", "42"].into_iter(),
    ///         &mut policy,
    ///         |ret, parser| {
    ///             if ret.status() {
    ///                 assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                 assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///     println!("{} running over!", parser.name());
    /// #
    /// # Ok(())
    /// # }
    ///```
    pub fn run_with<'c, 'b, I, R, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args = iter.map(|v| v.into());
        let ret = self.parse_policy(aopt::ARef::new(Args::from(args)), policy)?;

        r(ret, self)
    }

    /// Call [`run_with`](Self::run_with) with default arguments [`args()`](std::env::args).
    pub fn run<'c, 'b, R, F, P>(&'c mut self, policy: &mut P, r: F) -> Result<R, Error>
    where
        'c: 'b,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
        F: FnMut(P::Ret, &'b Self) -> Result<R, Error>,
    {
        let args: Vec<aopt::raw::RawVal> = Args::from_env().into();
        self.run_with(args.into_iter(), policy, r)
    }

    /// Running async function after parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aopt::Error;
    /// # use cote::*;
    /// #
    /// #[tokio::main]
    /// # async fn main() -> Result<(), Error> {
    ///     let mut policy = FwdPolicy::default();
    ///     let mut parser = Parser::<ASet, ASer>::default().with_name("example");
    ///
    ///     parser.add_opt_i::<bool>("-a!")?;
    ///     parser.add_opt_i::<i64>("-b")?;
    ///
    ///     parser
    ///         .run_async_with(
    ///             ["-a", "-b", "42"].into_iter(),
    ///             &mut policy,
    ///             |ret, parser| async move {
    ///                 if ret.status() {
    ///                     assert_eq!(parser.find_val::<bool>("-a")?, &true);
    ///                     assert_eq!(parser.find_val::<i64>("-b")?, &42);
    ///                 }
    ///                 Ok(())
    ///             },
    ///         )
    ///         .await?;
    ///     println!("{} running over!", parser.name());
    /// # Ok(())
    /// # }
    ///```
    pub async fn run_async_with<'c, 'b, I, R, FUT, F, P>(
        &'c mut self,
        iter: impl Iterator<Item = I>,
        policy: &mut P,
        mut r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        I: Into<RawVal>,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b Self) -> FUT,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        let args = iter.map(|v| v.into());
        let async_ret;

        match self.parse_policy(aopt::ARef::new(Args::from(args)), policy) {
            Ok(ret) => {
                let ret = r(ret, self).await;

                async_ret = ret;
            }
            Err(e) => {
                async_ret = Err(e);
            }
        }
        async_ret
    }

    /// Call [`run_async_with`](Self::run_async_with) with default arguments [`args()`](std::env::args).
    pub async fn run_async<'c, 'b, R, FUT, F, P>(
        &'c mut self,
        policy: &mut P,
        r: F,
    ) -> Result<R, Error>
    where
        'c: 'b,
        FUT: Future<Output = Result<R, Error>>,
        F: FnMut(P::Ret, &'b Self) -> FUT,
        P: Policy<Set = Self, Inv<'a> = Invoker<'a, Self, Ser>, Ser = Ser>,
    {
        let args: Vec<aopt::raw::RawVal> = Args::from_env().into();
        self.run_async_with(args.into_iter(), policy, r).await
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: aopt::set::Set,
{
    pub const DEFAULT_OPTION_WIDTH: usize = 40;
    pub const DEFAULT_USAGE_WIDTH: usize = 10;

    pub fn display_help(
        &self,
        author: &str,
        version: &str,
        description: &str,
    ) -> Result<(), Error> {
        let set = self.optset();
        let name = self.name.as_str();

        crate::display_help!(
            set,
            name,
            author,
            version,
            description,
            Self::DEFAULT_OPTION_WIDTH,
            Self::DEFAULT_USAGE_WIDTH
        )
    }

    pub fn display_sub_help(&self, ctx: Vec<HelpContext>) -> Result<(), Error> {
        self.display_sub_help_impl(ctx, 0)
    }

    fn display_sub_help_impl(&self, hcs: Vec<HelpContext>, i: usize) -> Result<(), Error> {
        if !hcs.is_empty() {
            let max = hcs.len() - 1;

            if let Some(hc) = hcs.get(i) {
                if i == max && (i > 0 || hc.name() == self.name()) {
                    let names: Vec<&str> = hcs.iter().map(|v| v.name().as_str()).collect();
                    let name = names.join(" ");
                    let optset = self.optset();

                    return crate::display_help!(
                        optset,
                        &name,
                        hc.head(),
                        hc.foot(),
                        hc.width(),
                        hc.usagew()
                    );
                } else if i < max && hc.name() == self.name() {
                    if let Some(hc) = hcs.get(i + 1) {
                        let sub_parsers = self.parsers();

                        for sub_parser in sub_parsers {
                            if sub_parser.name() == hc.name() {
                                return sub_parser.display_sub_help_impl(hcs, i + 1);
                            }
                        }
                    }
                }
            }
        }
        Err(raise_error!(
            "Can not display help message for ctxs: {hcs:?}"
        ))
    }

    pub fn display_help_ctx(&self, ctx: HelpContext) -> Result<(), Error> {
        let name = ctx.generate_name();
        let set = self.optset();

        crate::display_help!(
            set,
            &name,
            ctx.head(),
            ctx.foot(),
            ctx.width(),
            ctx.usagew()
        )
    }
}

impl<'a, Set, Ser> Parser<'a, Set, Ser>
where
    Set: SetValueFindExt,
    SetCfg<Set>: ConfigValue + Default,
{
    pub fn display_help_if(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        author: &str,
        version: &str,
        description: &str,
    ) -> Result<bool, Error> {
        self.display_help_if_width(
            option,
            author,
            version,
            description,
            Self::DEFAULT_OPTION_WIDTH,
            Self::DEFAULT_USAGE_WIDTH,
        )
    }

    pub fn display_help_if_ctx(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        ctx: &HelpContext,
    ) -> Result<bool, Error> {
        let set = self.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let name = ctx.generate_name();
                let set = self.optset();

                crate::help::display_set_help(
                    set,
                    name,
                    ctx.head(),
                    ctx.foot(),
                    ctx.width(),
                    ctx.usagew(),
                )
                .map_err(|e| aopt::raise_error!("Can not show help message: {:?}", e))?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn display_help_if_width(
        &self,
        option: impl ConfigBuild<SetCfg<Set>>,
        author: &str,
        version: &str,
        description: &str,
        option_width: usize,
        usage_width: usize,
    ) -> Result<bool, Error> {
        let set = self.optset();

        if let Ok(help_option) = set.find_val::<bool>(option) {
            if *help_option {
                let name = self.name.as_str();

                crate::display_help!(
                    set,
                    name,
                    author,
                    version,
                    description,
                    option_width,
                    usage_width
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
