use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ctx::wrap_handler;
use crate::ctx::wrap_handler_action;
use crate::ctx::wrap_handler_fallback;
use crate::ctx::Ctx;
use crate::ctx::Extract;
use crate::ctx::Handler;
use crate::ctx::Store;
use crate::map::ErasedTy;
use crate::opt::Opt;
use crate::set::SetExt;
use crate::set::SetOpt;
use crate::trace_log;
use crate::Error;
use crate::HashMap;
use crate::Uid;

/// Keep the variable length arguments handler in [`HashMap`] with key [`Uid`].
///
/// # Example
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::ARef;
/// # use aopt::Error;
/// #
/// # fn main() -> Result<(), Error> {
///  pub struct Count(usize);
///
///  // implement Extract for your type
///  impl Extract<ASet, ASer> for Count {
///      type Error = Error;
///
///      fn extract(_set: &ASet, _ser: &ASer, ctx: &Ctx) -> Result<Self, Self::Error> {
///          Ok(Self(ctx.args().len()))
///      }
///  }
///  let mut ser = ASer::default();
///  let mut is = Invoker::new();
///  let mut set = ASet::default();
///  let args = ARef::new(Args::from_array(["--foo", "bar", "doo"]));
///  let mut ctx = Ctx::default().with_args(args);
///
///  ser.sve_insert(ser::Value::new(42i64));
///  // you can register callback into Invoker
///  is.entry(0)
///      .on(
///          |_set: &mut ASet, _: &mut ASer| -> Result<Option<()>, Error> {
///              println!("Calling the handler of {{0}}");
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(1)
///      .on(
///          |_set: &mut ASet, _: &mut ASer, cnt: Count| -> Result<Option<()>, Error> {
///              println!("Calling the handler of {{1}}");
///              assert_eq!(cnt.0, 3);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///  is.entry(2)
///      .on(
///          |_set: &mut ASet, _: &mut ASer, data: ser::Value<i64>| -> Result<Option<()>, Error> {
///              println!("Calling the handler of {{2}}");
///              assert_eq!(data.as_ref(), &42);
///              Ok(None)
///          },
///      )
///      .then(NullStore);
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(0)));
///  is.invoke(&mut set, &mut ser, &ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(1)));
///  is.invoke(&mut set, &mut ser, &ctx)?;
///
///  ctx.set_inner_ctx(Some(InnerCtx::default().with_uid(2)));
///  is.invoke(&mut set, &mut ser, &ctx)?;
/// #
/// #    Ok(())
/// # }
/// ```
pub struct Invoker<Set, Ser> {
    callbacks: HashMap<Uid, InvokeHandler<Set, Ser, Error>>,
}

pub type InvokeHandler<Set, Ser, Error> =
    Box<dyn FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error> + Send + Sync>;

impl<Set, Ser> Debug for Invoker<Set, Ser> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Invoker")
            .field("callbacks", &"{ ... }")
            .finish()
    }
}

impl<Set, Ser> Default for Invoker<Set, Ser> {
    fn default() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Ser> Invoker<Set, Ser> {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::default(),
        }
    }
}

impl<Set, Ser> Invoker<Set, Ser>
where
    Ser: 'static,
    Set: crate::set::Set + 'static,
{
    pub fn set_raw<
        H: FnMut(&mut Set, &mut Ser, &Ctx) -> Result<bool, Error> + Send + Sync + 'static,
    >(
        &mut self,
        uid: Uid,
        handler: H,
    ) -> &mut Self {
        self.callbacks.insert(uid, Box::new(handler));
        self
    }

    /// Register a callback that will called by [`Policy`](crate::parser::Policy) when option set.
    ///
    /// The [`Invoker`] first call the [`invoke`](crate::ctx::Handler::invoke), then
    /// call the [`process`](crate::ctx::Store::process) with the return value.
    /// # Note
    /// ```txt
    /// |   handler: |&mut Set, &mut Ser, { Other Args }| -> Result<Option<Value>, Error>
    /// |   storer: |&mut Set, &mut Ser, Option<&RawVal>, Option<Value>| -> Result<bool, Error>
    ///         |
    ///      wrapped
    ///         |
    ///         v
    /// |   |&mut Set, &mut Ser, &Ctx| -> Option<Value>
    ///         |
    ///      invoked
    ///         |
    ///         v
    /// |   call Callbacks::invoke(&mut self, &mut Set, &mut Ser, &Ctx)
    /// |       call Handler::invoke(&mut self, &mut Set, &mut Ser, Args)
    /// |           call Args::extract(&Set, &Ser, &Ctx) -> Args
    /// |           -> Result<Option<Value>, Error>
    /// |       -> call Store::process(&Set, Option<&RawVal>, Option<Value>)
    /// |           -> Result<bool, Error>
    /// ```
    pub fn set_handler<A, O, H, T>(&mut self, uid: Uid, handler: H, store: T) -> &mut Self
    where
        O: ErasedTy,
        A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
        T: Store<Set, Ser, O, Ret = bool, Error = Error> + Send + Sync + 'static,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
    {
        self.set_raw(uid, wrap_handler(handler, store));
        self
    }

    pub fn has(&self, uid: Uid) -> bool {
        self.callbacks.contains_key(&uid)
    }

    /// Invoke the handler saved in [`Invoker`], it will panic if the handler not exist.
    pub fn invoke(&mut self, set: &mut Set, ser: &mut Ser, ctx: &Ctx) -> Result<bool, Error> {
        let uid = ctx.uid()?;

        crate::trace_log!("Invoking callback of {}({:?})", uid, ctx);
        if let Some(callback) = self.callbacks.get_mut(&uid) {
            return (callback)(set, ser, ctx);
        }
        unreachable!(
            "There is no callback of {}, call `invoke_default` instead",
            set.opt(uid)?.name()
        )
    }
}

impl<Set, Ser> Invoker<Set, Ser>
where
    SetOpt<Set>: Opt,
    Set: crate::set::Set,
{
    pub fn entry<A, O, H>(&mut self, uid: Uid) -> HandlerEntry<'_, Set, Ser, H, A, O>
    where
        O: ErasedTy,
        H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
        A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
    {
        HandlerEntry::new(self, uid)
    }

    /// The default handler for all option.
    ///
    /// If there no handler for a option, then default handler will be called.
    /// It will parsing [`RawVal`](crate::RawVal)(using [`RawValParser`]) into associated type, then call the action
    /// of option save the value to [`AnyValService`](crate::ser::AnyValService).
    /// For value type, reference documents of [`Assoc`].
    pub fn fallback(set: &mut Set, _: &mut Ser, ctx: &Ctx) -> Result<bool, Error> {
        let uid = ctx.uid()?;
        let opt = set.get_mut(uid).unwrap();
        let arg = ctx.arg()?;
        let raw = arg.as_ref().map(|v| v.as_ref());
        let act = *opt.action();

        trace_log!("Invoke fallback for {}({act}) {{{ctx:?}}}", opt.name());
        opt.accessor_mut().store_all(raw, ctx, &act)
    }

    pub fn invoke_default(
        &mut self,
        set: &mut Set,
        ser: &mut Ser,
        ctx: &Ctx,
    ) -> Result<bool, Error> {
        Self::fallback(set, ser, ctx)
    }
}

pub struct HandlerEntry<'a, Set, Ser, H, A, O>
where
    O: ErasedTy,
    Ser: 'static,
    Set: crate::set::Set + 'static,
    SetOpt<Set>: Opt,
    H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
    A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
{
    ser: &'a mut Invoker<Set, Ser>,

    handler: Option<H>,

    register: bool,

    uid: Uid,

    marker: PhantomData<(A, O)>,
}

impl<'a, Set, Ser, H, A, O> HandlerEntry<'a, Set, Ser, H, A, O>
where
    O: ErasedTy,
    Ser: 'static,
    Set: crate::set::Set + 'static,
    SetOpt<Set>: Opt,
    H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
    A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
{
    pub fn new(inv_ser: &'a mut Invoker<Set, Ser>, uid: Uid) -> Self {
        Self {
            ser: inv_ser,
            handler: None,
            register: false,
            uid,
            marker: PhantomData::default(),
        }
    }

    /// Register the handler which will be called when option is set.
    pub fn on(mut self, handler: H) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Register the handler which will be called when option is set.
    /// And the [`fallback`](crate::ctx::Invoker::fallback) will be called if
    /// the handler return None.
    pub fn fallback(mut self, handler: H) -> Self {
        if !self.register {
            self.ser.set_raw(self.uid, wrap_handler_fallback(handler));
            self.register = true;
        }
        self
    }

    /// Register the handler with given store.
    /// The store will be used save the return value of option handler.
    pub fn then(
        mut self,
        store: impl Store<Set, Ser, O, Ret = bool, Error = Error> + Send + Sync + 'static,
    ) -> Self {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                self.ser.set_raw(self.uid, wrap_handler(handler, store));
            }
            self.register = true;
        }
        self
    }

    pub fn submit(mut self) -> Uid {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                self.ser.set_raw(self.uid, wrap_handler_action(handler));
            }
            self.register = true;
        }
        self.uid
    }
}

impl<'a, Set, Ser, H, A, O> Drop for HandlerEntry<'a, Set, Ser, H, A, O>
where
    O: ErasedTy,
    Ser: 'static,
    Set: crate::set::Set + 'static,
    SetOpt<Set>: Opt,
    H: Handler<Set, Ser, A, Output = Option<O>, Error = Error> + Send + Sync + 'static,
    A: Extract<Set, Ser, Error = Error> + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if !self.register {
            if let Some(handler) = self.handler.take() {
                self.ser.set_raw(self.uid, wrap_handler_action(handler));
            }
            self.register = true;
        }
    }
}