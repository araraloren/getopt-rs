mod delay;
mod first;
mod invoke;
mod multi;
mod noa;
mod single;
mod style;

///
/// argument boolean/flag embedded equalwithvalue - generate one guess
///     - invoke
///         - first
///             - match first opt
///             - invoke the handler of first opt
///             - set first opt matched if handler return Ok(Some(_))
///         - all
///             - match all the opt
///             - invoke the handler of all matched opt
///             - set opt matched and return if any handler return Ok(Some(_))
///    - delay
///         - first
///             - match first opt
///             - return the inner ctx
///         - all
///             - match all the opt
///             - return the inner ctxs
///     
/// embeddedplus combined - generate multiple guess
///     - invoke
///         - first
///             - match first opt
///             - invoke the handler of first opt
///             - set first opt matched if handler return Ok(Some(_))
///         - all
///             - match all the opt
///             - invoke the handler of all matched opt
///             - set opt matched and return if any handler return Ok(Some(_))
///    - delay
///         - first
///             - match first opt
///             - return the inner ctx
///         - all
///             - match all the opt
///             - return the inner ctxs
/// main pos cmd - generate one guess
///     - invoke
///         - match all the opt
///         - invoke the handler of all matched opt
///         - set all the opt matched if handler return Ok(Some(_))
///     - delay mode
///         not support
///
use crate::Error;
use crate::Uid;
use crate::args::Args;
use crate::opt::Opt;
use crate::opt::Style;
use crate::set::Set;
use crate::set::SetOpt;
use crate::ARef;
use crate::RawVal;
use crate::Str;

// pub use self::delay::DelayGuess;
// pub use self::delay::InnerCtxSaver;
pub use self::first::FirstOpt;
pub use self::invoke::InvokeGuess;
pub use self::multi::MultiOpt;
pub use self::noa::SingleNonOpt;
pub use self::single::SingleOpt;

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleMatRes {
    pub matched: bool,

    pub consume: bool,
}

impl SimpleMatRes {
    pub fn new(matched: bool, consume: bool) -> Self {
        Self { matched, consume }
    }
}

pub trait GuessPolicy<S> {
    type All;
    type First;
    type Error: Into<Error>;

    fn guess_all(&mut self) -> Result<Option<Self::All>, Self::Error>;

    fn guess_first(&mut self) -> Result<Option<Self::First>, Self::Error>;
}

pub trait GuessOpt<T> {
    type Ret;
    type Policy;
    type Error: Into<Error>;

    fn guess_policy(&mut self) -> Result<Self::Policy, Self::Error>;

    fn guess_opt(&mut self, policy: &mut Self::Policy) -> Result<Self::Ret, Self::Error>;
}

pub trait Process<Policy> {
    type Ret;
    type Error: Into<Error>;

    fn match_all(&mut self, policy: &mut Policy) -> Result<bool, Self::Error>;

    fn invoke_handler(&mut self, policy: &mut Policy) -> Result<Self::Ret, Self::Error>;
}

pub trait MatchPolicy {
    type Set;
    type Ret;
    type Error: Into<Error>;

    fn reset(&mut self) -> &mut Self;

    fn matched(&self) -> bool;

    fn undo(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn apply(&mut self, uid: Uid, set: &mut Self::Set) -> Result<(), Self::Error>;

    fn filter(&mut self, uid: Uid, set: &mut Self::Set) -> bool;

    fn r#match(&mut self, uid: Uid, set: &mut Self::Set) -> Result<Self::Ret, Self::Error>;
}

pub trait PolicyBuild {
    fn with_name(self, name: Str) -> Self;

    fn with_style(self, style: Style) -> Self;

    fn with_idx(self, index: usize) -> Self;

    fn with_total(self, total: usize) -> Self;

    fn with_consume(self, consume: bool) -> Self;

    fn with_arg(self, argument: Option<ARef<RawVal>>) -> Self;

    fn with_args(self, args: ARef<Args>) -> Self;
}

pub fn process_handler_ret(
    ret: Result<bool, Error>,
    mut when_ret: impl FnMut(bool) -> Result<(), Error>,
    mut when_fail: impl FnMut(Error) -> Result<(), Error>,
) -> Result<bool, Error> {
    match ret {
        Ok(ret) => {
            (when_ret)(ret)?;
            Ok(ret)
        }
        Err(e) => {
            if e.is_failure() {
                (when_fail)(e)?;
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}
