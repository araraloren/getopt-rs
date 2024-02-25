//!
//! # Documentation: Cote Tutorial
//!
//! 1. [Quick Start](#quick-start)
//!     1. [Help message generate](#help-message-generate)
//!     2. [Running](#running)
//! 2. [Configurating Struct](#configurating-struct)
//!     1. [Configurating Policy](#configurating-policy)
//!     2. [Configurating Help](#configurating-help)
//!     3. [Configurating User Style](#configurating-user-style)
//! 3. [Configurating Field](#configurating-field)
//!     1. [Options](#options)
//!     2. [Positionals](#positionals)
//!     3. [Command Flags](#command-flags)
//!     4. [Sub Commands](#sub-commands)
//! 4. [Configurating Options, Command flags and Positionals](#configurating-options-command-flags-and-positionals)
//!     1. [Configurating the name and alias](#configurating-the-name-and-alias)
//!     2. [Configurating the hint, help and default value](#configurating-the-hint-help-and-default-value)
//!     3. [Configurating the index](#configurating-the-index)
//!     4. [Force required Positionals and Options](#force-required-positionals-and-options)
//!     5. [Configurating action](#configurating-action)
//!     6. [Configurating handler](#configurating-handler)
//!     7. [Validate values](#validate-values)
//!     8. [Add "no delay" option](#add-no-delay-option)
//! 5. [Configurating Sub Commands](#configurating-sub-commands)
//!     1. [Configurating Policy](#configurating-policy)
//!     2. [Configurating name and alias](#configurating-name-and-alias)
//!     3. [Configurating help message](#configurating-help-message)
//!     4. [Optional Sub commands](#optional-sub-commands)
//! 6. [How it works](#how-it-works)
//!     1. [Traits](#traits)
//!     2. [`Cote` Configurations list](#cote-configurations-list)
//!     2. [`CoteOpt` Configurations list](#copt-configurations-list)
//!     2. [`CoteVal` Configurations list](#coteval-configurations-list)
//!
//! ## Quick Start
//!
//! Using [`Cote`](crate::cote_derive::Cote) derive you can quick setup a application.
//!
//! ```no_run
#![doc = include_str!("../examples/01_quick_start.rs")]
//! ```
//!
//! ### Help message generate
//!
//! - Output of `cli --help`:
//!
//! ```!
//! Usage: cli [-h,-?,--help] [-d,--debug] [-c,--config [CFG]] <COMMAND>
//!
//! Generate help message for command line program
//!
//! Commands:
//!  se@1       Search the given directory
//!  ls@1       List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! - Output of `cli ls --help`:
//!
//! ```!
//! Usage: cli ls [-h,-?,--help] [--recursive] [ARGS]
//!
//! List the given directory
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   --recursive       Enable recursive mode
//!
//! Args:
//!   dest@1      Set the list directory ["."]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! ### Running
//!
//! Output of `cli se --depth 2`:
//!
//! ```!
//! loading config from "default.json"
//! search the file under directory `Some(".")` with depth 2
//! ```
//!
//! #### `aborthelp`
//!
//! If code generate with cote configuration `aborthelp`.
//! When the option match failed, program will first
//! print help message, then display the error message.
//!
//! Output of `cli se --depth www` or `cli se --depth`:
//!
//! ```!
//! Usage: cli [-h,-?,--help] [-d,--debug] [-c,--config [CFG]] <COMMAND>
//!
//! Generate help message for command line program
//!
//! Commands:
//!   se@1      Search the given directory
//!   ls@1      List the given directory
//!
//! Options:
//!   -h,-?,--help           Display help message
//!   -d,--debug             Print debug message
//!   -c,--config [CFG]      Set the configuration path ["default.json"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: Parsing command `se` failed: InnerCtx { uid: 1, name: Some(--depth), style: Style::Argument, arg: Some("www"), index: 1, total: 3 }
//!    1: Can not find option `--depth`
//!    2: Can not convert value `www` to usize
//!    3: invalid digit found in string
//!
//! Location:
//!    src\main.rs:82
//!
//! Backtrace omitted.
//! Run with RUST_BACKTRACE=1 environment variable to display it.
//! Run with RUST_BACKTRACE=full to include source snippets.
//! ```
//!
//! ## Configurating Struct
//!
//! ### Configurating Policy
//!
//! Cote has three policy types built-in: [`fwd`](crate::FwdPolicy)、[`pre`](crate::PrePolicy)
//! and [`delay`](crate::DelayPolicy).
//! If no `policy` configuration specific, [`fwd`](crate::FwdPolicy) will be using if no sub command.
//! Otherwise [`pre`](crate::PrePolicy) will be used.
//!
//! ```rust
#![doc = include_str!("../examples/02_config_policy.rs")]
//! ```
//!
//! ### Configurating Help
//!
//! Specify `help` in `cote` attribute will automate generate help message for current application.
//! And `aborthelp` will automate display the help message if any error raised.
//!
//! The default name of the application is the name of the current package, use `name` custom it,
//! i.e., the result of `String::from(env!("CARGO_PKG_NAME"))`.
//!
//! The default maximum length of the option help message is 40, use `width` custom it.
//! The default maximum count of usage option item is 10, use `usagew` custom it.
//!
//! The text set by `head` will display after usage, in default it is description of package,
//! i.e., the result of `String::from(env!("CARGO_PKG_DESCRIPTION"))`.
//!
//! The text set by `foot` will display at the bottom, in default it is result of
//! `format!("Create by {} v{}", env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_VERSION"))`.
//!
//! #### Example
//!
//! ```rust
#![doc = include_str!("../examples/03_config_help.rs")]
//! ```
//!
//! The help message output like this:
//!
//! ```!
//! Usage: app [-h,-?,--help] [--debug] <--name>
//!        <COMMAND> [ARGS]
//!
//! The head message display in help message
//!
//! Commands:
//!   foo@1      Switch to foo sub command.
//!   bar@1      Switch to bar sub command.
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   --debug           Print debug message.
//!   --name            Set the name of client.
//!
//! Args:
//!   arg@2         The second position argument.
//!   args@3..      Collection of arguments start from position 3.
//!
//! The foot message display in help message
//! ```
//!
//! ### Configurating User Style
//!
//! The option styles support by default are:
//!
//! - [`EqualWithValue`](crate::UserStyle::EqualWithValue)
//!
//! Options such as `--opt=value`, the value of option is set after `=`.
//!
//! - [`Argument`](crate::UserStyle::Argument)
//!
//! Options such as `--opt value`, the value of option is next argument.
//!
//! - [`EmbeddedValue`](crate::UserStyle::EmbeddedValue)
//!
//! Options such as `-o42`, the value `42` of option is embedded in string.
//! The style only support one letter option.
//!
//! - [`Boolean`](crate::UserStyle::Boolean)
//!
//! Options such as `--opt`, in general, it is named flag, the value type of option is always `bool`.
//!
//! - `combine` - Add support for [`CombinedOption`](crate::UserStyle::CombinedOption).
//!
//! Options such as `-abcd`, thus set both boolean options `-a`, `-b`, `-c` and `-d`.
//!
//! ```rust
#![doc = include_str!("../examples/04_config_style.rs")]
//! ```
//!
//! - `embedded` - Add support for [`EmbeddedValuePlus`](crate::UserStyle::EmbeddedValuePlus).
//!
//! Options such as `--opt42`, the value `42` of option is embedded in string.
//! The style only supports options which name lengths bigger than 2.
//!
//! ```rust
#![doc = include_str!("../examples/05_embedded_value_plus.rs")]
//! ```
//!
//! - `flag` - Add support for [`Flag`](crate::UserStyle::Flag).
//!
//! Options such as `--opt`, in general, it is named flag, the value type of option is always `bool`.
//! Unlike [`Boolean`](crate::UserStyle::Boolean) pass [`TRUE`](crate::aopt::opt::BOOL_TRUE) to [`parse`](crate::RawValParser::parse),
//! [`Flag`](crate::UserStyle::Flag) pass [`None`] to [`parse`](crate::RawValParser::parse).
//!
//! ```rust
#![doc = include_str!("../examples/25_flag.rs")]
//! ```
//!
//! ## Configurating Field
//!
//! ### Options
//!
//! In default or specific the attribute `arg`, the fields of struct are generated into options.
//!
//! ```rust
#![doc = include_str!("../examples/06_option_demo.rs")]
//! ```
//!
//! ### Positionals
//!
//! Specific the attribute `pos` if you want to match the command line arguments by position.
//!
//! ```
#![doc = include_str!("../examples/07_positional_demo.rs")]
//! ```
//!
//! ### Command Flags
//!
//! Specific the attribute `cmd` will let you create a sub command flag.
//!
//! ```rust
#![doc = include_str!("../examples/08_command_flag_demo.rs")]
//! ```
//!
//! ### Sub Commands
//!
//! Specific the attribute `sub` will let you create a sub commands.
//!
//! ```rust
#![doc = include_str!("../examples/09_sub_command_demo.rs")]
//! ```
//!
//! ## Configurating Options, Command flags and Positionals
//!
//! ### Configurating the name and alias
//!
//! The default name of positionals and command flags is the name of the field.
//!
//! The default name of options consists of prefixs and identifiers of the field.
//! The default prefix is `--` if count of characters bigger than 1, otherwise `-` is using.
//! You can use `name` or `alias` configure the name and alias of the option.
//! For prefix information reference [`PrefixOptValidator`](crate::PrefixOptValidator).
//!
//! ```rust
#![doc = include_str!("../examples/10_arg_name_alias.rs")]
//! ```
//!
//! ### Configurating the hint, help and default value
//!
//! Hint is displayed on usage or the left side of item information.
//! In default, hint message is generated from the name and alias of item,
//! use `hint` custom the hint information of item.
//! Help is displayed on the right side of item information.
//! Use `help` configure the help information of item.
//! The default values will be display in help message if it is set.
//!
//!
//! ```rust
#![doc = include_str!("../examples/11_arg_hint_help.rs")]
//! ```
//!
//! Running the code, it's output should be:
//!
//! ```!
//! Usage: cli [-h,-?,--help] <-b,--baz> <COMMAND> [ARGS]
//!
//! Generate help message for command line program
//!
//! Commands:
//!   foo@1      Switch the mode to foo command
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   -b,--baz          Set the string value of baz
//!
//! Args:
//!   [BAR]         Set the value of bar [42usize]
//!   quux@3..
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! ### Configurating the index
//!
//! Index is only support positions and command flags.
//! For command flags, the index is fixed position `@1` by default.
//! For more informations about index, reference [`Index`](crate::Index).
//!
//! #### Example1
//!
//! ```rust
#![doc = include_str!("../examples/12_arg_index.rs")]
//! ```
//!
//! #### Example2
//!
//! For the item configured by `pos`, the index is automating generated start form 1
//! if no index set.
//!
//! ```rust
#![doc = include_str!("../examples/13_arg_index.rs")]
//! ```
//!
//! ### Force required Positionals and Options
//!
//! In default, options, positionals and command flags is force required.
//! Wrap the type with `Option` can make the item optional.
//! Using `force` you can configure the positionals and options force required.
//!
//! ```rust
#![doc = include_str!("../examples/14_arg_force.rs")]
//! ```
//!
//! ### Configurating action
//!
//! The type that implements [`Infer`](crate::Infer) has different [`Action`](crate::Action).
//! The [`Action`](crate::Action) defines the behavior when saving the value.
//! For more information, see [`Action::process`](crate::Action#method.process) and [`AOpt`](crate::AOpt).
//!
//! ```rust
#![doc = include_str!("../examples/15_arg_action.rs")]
//! ```
//!
//! ### Configurating handler
//!
//! Using `on`, `fallback` attribute configure the handler which will be called when
//! option set.
//! Using `then` attribute configure the store behavior when saving value.
//!
//! - `on`
//!
//!     + `cote`
//!
//!     Will be invoked if struct parsed successfully.
//!     Because the name of [`Main`](aopt::opt::Main) option will be generate automate.
//!     So you can't get the return value currently.
//!
//!     + `arg` or `pos`
//!
//!     Will be invoked if option set by user.
//!     The return value will be saved as value of option.
//!
//!     + `sub`
//!
//!     Not support, set the handler on struct type using `cote`.
//!
//! - `fallback`
//!     
//! Same as `on` except if the handler returns `Ok(None)`, the default handler will be invoked.
//!
//! - `then`
//!
//! Using with `on` and `fallback`, do nothing without `on` and `fallback`.
//! It will responded for saving the raw value and value.
//!
//! ```no_run
#![doc = include_str!("../examples/16_arg_handler.rs")]
//! ```
//!
//! - Output of command line `cli --foo 6`:
//!
//! ```!
//! Saving the value of `--foo` to 7
//! Got client: Cli { foo: 7, bar: None, qux: None }
//! ```
//!
//! - Output of command line `cli --foo 8 bar a2i`:
//!
//! ```!
//! Saving the value of `--foo` to 9
//! Got client: Cli { foo: 9, bar: Some(Bar { debug: false, quux: "a2i" }), qux: None }
//! ```
//!
//! - Output of command line `cli --foo 8 bar a2i --debug`:
//!
//! ```!
//! Saving the value of `--foo` to 9
//! Got value of `--debug`: RawVal("true") --> true
//! Got client: Cli { foo: 9, bar: Some(Bar { debug: false, quux: "a2i" }), qux: None }
//! ```
//!
//! - Output of command line `cli --foo 9 qux c`:
//!
//! ```!
//! Saving the value of `--foo` to 10
//! return Ok(None) call the default handler of Qux
//! Got client: Cli { foo: 9, bar: None, qux: Some(Qux { corge: true, grault: None }) }
//! ```
//!
//! - Output of command line `cli --foo 9 qux c --grault=42`:
//!
//! ```!
//! Saving the value of `--foo` to 10
//! return Ok(None) call the default handler of Qux
//! Got client: Cli { foo: 9, bar: None, qux: Some(Qux { corge: true, grault: Some(42) }) }
//! ```
//!
//! ### Validate values
//!
//! You can using `valid` check the value inside attribute.
//! Using [`valid!`](crate::valid!) generate struct implemented [`Validate`](crate::valid::Validate)
//! for the valid attribute.
//!
//! ```rust
#![doc = include_str!("../examples/17_arg_validator.rs")]
//! ```
//!
//! ### Add "no delay" option
//!
//! When using [`DelayPolicy`](crate::DelayPolicy), the option process(invoke handler)
//! after [`Cmd`](crate::UserStyle::Cmd) and [`Pos`](crate::UserStyle::Pos) style.
//! Sometimes we need the option process like [`FwdPolicy`](crate::FwdPolicy) does,
//! that is process before [`Cmd`](crate::UserStyle::Cmd) and [`Pos`](crate::UserStyle::Pos).
//!
//!```rust
#![doc = include_str!("../examples/18_arg_no_delay.rs")]
//! ```
//!
//! ## Configurating Sub Commands
//!
//! Using `sub` attribute define sub command.
//!
//! ```no_run
#![doc = include_str!("../examples/19_sub_command.rs")]
//! ```
//!
//! ### Configurating Policy
//!
//! The default [`Policy`](crate::Policy) of sub command is [`FwdPolicy`](crate::FwdPolicy).
//! For the sub commands to have sub commands, you should use [`PrePolicy`](crate::PrePolicy) instead.
//! For example, `sport` sub command does have two sub commands, it is configured with `#[sub(policy = pre)]`.
//! Without `policy = pre`, you will got output when running `cli -g=42 sport walk -d 4`:
//!
//! ```!
//! Usage: cli sport [-h,-?,--help] <COMMAND>
//! Generate help message for command line program
//!
//! Commands:
//!   walk@1      Go for a walk.
//!   play@1      Play some games.
//!
//! Options:
//!   -h,-?,--help      Display help message
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! Error:
//!    0: Parsing command `sport` failed: None
//!    1: Can not find option `-d`
//!
//! Location:
//!    src\main.rs:90
//!
//! Backtrace omitted.
//! Run with RUST_BACKTRACE=1 environment variable to display it.
//! Run with RUST_BACKTRACE=full to include source snippets.
//! ```
//! And the right output should be:
//! ```!
//! You age is set to 42
//! You are going to walk 4 kilometers
//! ```
//!
//! ### Configurating name and alias
//!
//! Using `name` and `alias` you can configure the name and alias of sub commands in `sub` attribute.
//! The name and alias will affect how to set the sub command and help message of sub command.
//! With follow change:
//!
//! ```no_run
#![doc = include_str!("../examples/20_sub_name_alias.rs")]
//! ```
//!
//! The output of commands `cli -g22 e --help` is:
//!
//! ```!
//! Usage: cli e [-h,-?,--help] <-m,--meal> [ARGS]
//!
//! Generate help message for command line program
//!
//! Options:
//!   -h,-?,--help      Display help message
//!   -m,--meal         Which meal did you have?
//!
//! Args:
//!   what@1      What did you wat? ["rice"]
//!
//! Create by araraloren <blackcatoverwall@gmail.com> v0.1.8
//! ```
//!
//! ### Configurating help message
//!
//! Using `hint`, `help`, `head`, `foot` you can configure the help message of sub commands.
//! Just like those configures how work in `cote` attribute, they can tweak the help message of sub commands.
//!
//! ```no_run
#![doc = include_str!("../examples/21_sub_help.rs")]
//! ```
//!
//! The output of commands `cli -g8 sport --help` is:
//!
//! ```!
//! Usage: cli sport [-h,-?,--help] <COMMAND>
//!
//! This is head message of sport sub command.
//!
//! Commands:
//!   [walk]      Go for a walk.
//!   [play]      Play some games.
//!
//! Options:
//!   -h,-?,--help      Display help message
//!
//! This is foot message of sport sub command.
//!
//! ```
//!
//! ### Optional Sub commands
//!
//! The sub commands are force required in default.
//! Cote will raised an error if no sub command set.
//! Using `force` make all sub commands optional avoid this error.
//!
//! ```no_run
#![doc = include_str!("../examples/22_sub_optional.rs")]
//! ```
//!
//! Instead display the help and error message, the output of commands `cli -g8 sport` is:
//!
//! ```!
//! You age is set to 8
//!
//! ```
//!
//! ## How it works
//!
//! ### Traits
//!
//! Implement follow traits, you can using the type in the struct filed.
//!
//! - [`Infer`](crate::Infer)
//!
//! `Cote` using [`infer_fill_info`](crate::Infer::infer_fill_info) inference the default settings of
//! given type.
//!
//! - [`Fetch`](crate::Fetch)
//!
//! `Cote` using [`fetch`](crate::Fetch::fetch) fetch the value from [`Set`](aopt::set::Set).
//!
//! - [`RawValParser`](crate::RawValParser)
//!
//! `Cote` using [`parse`](crate::RawValParser::parse) parsing the value from command line arguments.
//!
//! - [`Alter`](crate::Alter)
//!
//! `Cote` using the trait override the action or optional behavior of [`Infer`](crate::Infer).
//!
//!| type | action | force required | force required if has default value |
//!|------|--------|----------|----------|
//!| `T` | [`Action::Set`](crate::Action::Set) | `true` | `false` |
//!| `Option<T>` | [`Action::Set`](crate::Action::Set) | `false` | `false` |
//!| `Vec<T>` | [`Action::App`](crate::Action::App) | `true` | `false` |
//!| `Option<Vec<T>>` | [`Action::App`](crate::Action::App) | `false` | `false` |
//!| [`Pos<T>`](crate::Pos) | [`Action::Set`](crate::Action::Set) | `true` | `false` |
//!| `bool` | [`Action::Set`](crate::Action::Set) | `false` | `false` |
//!| [`Cmd`](crate::Cmd) | [`Action::Set`](crate::Action::Set) | `true` | `true` |
//!
//! ### Example
//!
//! The type `Speed` base on the type `i32` which already implemented [`RawValParser`](crate::RawValParser).
//!
//! ```rust
#![doc = include_str!("../examples/23_wrapper.rs")]
//! ```
//!
//! ### Example - Derive default behavior from `CoteOpt` or `CoteVal` macro
//!
//! ```rust
#![doc = include_str!("../examples/24_rawvalparser.rs")]
//! ```
//!
//! ### `Cote` Configurations list
//!
//! #### `cote`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `policy`  |  true      | `"pre"`, `"fwd"`, `"delay"`, or type |
//!| `name`    |  true      | string literal |
//!| `help`    |  false     | |
//!| `head`    |  true      | string literal |
//!| `foot`    |  true      | string literal |
//!| `width`   |  true      | integer |
//!| `usagew`  |  true      | integer |
//!|`aborthelp`|  false     | |
//!| `on`      |  true      | function or closure |
//!| `fallback`|  true      | function or closure |
//!| `then`    |  true      | function or closure |
//!| `strict`  |  true      | boolean |
//!| `combine` |  false     | |
//!| `embedded`|  false     | |
//!| `flag`    |  false     | |
//! * `policy`
//!
//! Configure the policy of current struct, its value should be `fwd`, `pre` or `delay.
//! The default value is `fwd` if no sub command in the struct, otherwise it will be `pre`.
//! ```rust
#![doc = include_str!("../test/01_policy.rs")]
//! ```
//!
//! * `name`
//!
//! The name is display in usage information.
//!
//! * `help`
//!
//! Add default help option `-h`|`--help`, generate help message when option set.
//!
//! * `aborthelp`
//!
//! Display help message if any error raised or command line parsing failed.
//!
//! * `head`, `foot`
//!
//! Custom the help message display.
//!
//! ```rust
#![doc = include_str!("../test/02_head_foot.rs")]
//! ```
//!
//! * `width`, `usagew`
//!
//! `width` set the maximum length of option help message. `usagew` set the maximum count of options in usage.
//! See [`Configurating Help`](#configurating-help).
//!
//! * `on`, `fallback`, `then`
//!
//! Using `then` you can configure a handler which is responsible for storing the option value
//! (which is generated from the struct and inserted by cote-derive).
//! In default the handler is [`process`](crate::Action#method.process),
//! and the action is [`App`](crate::Action::App) or [`Set`](crate::Action::Set).
//!
//! And with `on`, you can set a handler will be invoked by [`policy`](crate::Policy),
//! the return value of handler will store as the value of option.
//!
//! ```rust
#![doc = include_str!("../test/04_on.rs")]
//! ```
//!
//! The `fallback` do same things as `on` except the [`fallback`](crate::Invoker::fallback) will be called
//! if the handler returns [`None`].
//!
//! ```rust
#![doc = include_str!("../test/05_fallback.rs")]
//! ```
//!
//! * `strict`
//!
//! Enable the strict mode of parser by calling the [`set_strict`](crate::PolicySettings::set_strict).
//! If the option
//!
//! ```rust
#![doc = include_str!("../test/03_strict.rs")]
//! ```
//!
//! * `combine`, `embedded`, `flag`
//!
//! Enable some extra [`user style`](crate::UserStyle) of policy. See also [`Configurating User Style`](#configurating-user-style).
//!
//! #### `arg`, `pos`, `cmd`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `name`    |  true      | string literal |
//!| `ty`      |  true      | type |
//!| `hint`    |  true      | string literal |
//!| `help`    |  true      | string literal |
//!| `value`   |  true      | value expression |
//!| `values`  |  true      | values expression |
//!| `alias`   |  true      | string literal |
//!| `index`   |  true      | range or integer |
//!| `force`   |  true      | boolean |
//!| `action`  |  true      | [`Action`](crate::Action) |
//!| `valid`   |  true      | [`valid!`](crate::valid!) |
//!| `on`      |  true      | function or closure |
//!| `fallback`|  true      | function or closure |
//!| `then`    |  true      | function or closure |
//!| `nodelay` |  false     | |
//!| `fetch`   |  true      | function |
//!| `append`  |  false     | |
//!| `count`   |  false     | |
//!
//! * `name`, `alias`
//!
//! Configure the name and alias of current option. See also [`Configurating the name and alias`](#configurating-the-name-and-alias).
//!
//! * `hint`, `help`
//!
//! Configure the name and help message of option.
//! See also [`Configurating the hint, help and default value`](#configurating-the-hint-help-and-default-value).
//!
//! * `value`, `values`
//!
//! Configure the default value of option, `cote-derive` using [`From`] convert given value to option value.
//!
//! ```rust
#![doc = include_str!("../test/06_value.rs")]
//! ```
//!
//! * `index`
//!
//! Configure the index of option, it is using for `pos`([`Pos`](crate::Pos)) attribute generally.
//!
//! ```rust
#![doc = include_str!("../test/07_index.rs")]
//! ```
//!
//! * `force`
//!
//! Make the option force required.
//!
//! ```rust
#![doc = include_str!("../test/08_force.rs")]
//! ```
//!
//! * `action`, `ty`, `append`, `count`
//!
//! `action` can configure the [`Action`](crate::Action) which responsible for saving value of option.
//! Using `ty` specify the option type when using [`Action::Cnt`](crate::Action::Cnt).
//!  
//! ```rust
#![doc = include_str!("../test/09_action.rs")]
//! ```
//!
//! `append` is an alias of "action = [`Action::App`](crate::Action::App)",
//! `count` is an alias of "action = [`Action::Cnt`](crate::Action::Cnt)"
//!
//! * `fetch`
//!
//! Configure the handler which is used to extract value from [`set`](crate::Set).
//!
//! ```rust
#![doc = include_str!("../test/10_fetch.rs")]
//! ```
//!
//! * `valid`
//!
//! Using [`valid!`](crate::valid!) validate the value set by user. See also [`Validate values`](#validate-values).
//!
//! ```rust
#![doc = include_str!("../test/11_valid.rs")]
//! ```
//!
//! * `on`, `fallback`, `then`
//!
//! Using `then` you can configure a handler which is responsible for storing the option value.
//! In default the handler is [`process`](crate::Action#method.process), and the action is [`Null`](crate::Action::Null).
//!
//! And with `on`, you can set a handler will be invoked by [`policy`](crate::Policy),
//! the return value of handler will store as the value of option.
//!
//! ```rust
#![doc = include_str!("../test/12_on.rs")]
//! ```
//!
//! The `fallback` do same things as `on` except the [`fallback`](crate::Invoker::fallback) will be called
//! if the handler returns [`None`].
//!
//! * `nodelay`
//!
//! Invoke the option's handler before any [`Cmd`](crate::UserStyle::Cmd) or [`Pos`](crate::UserStyle::Pos).
//! Only work for [`DelayPolicy`](crate::DelayPolicy) currently.
//! See also [`Add "no delay" option`](#add-no-delay-option).
//!
//! #### `sub`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `policy`  |  true      | `"pre"`, `"fwd"`, `"delay"`, or type |
//!| `name`    |  true      | string literal |
//!| `hint`    |  true      | string literal |
//!| `help`    |  true      | string literal |
//!| `head`    |  true      | string literal |
//!| `foot`    |  true      | string literal |
//!| `alias`   |  true      | string literal |
//!| `force`   |  true      | boolean |
//!
//! * `policy`
//!
//! Override the `policy` of sub command.
//!
//! ```rust
#![doc = include_str!("../test/13_policy.rs")]
//! ```
//!
//! * `name`, `alias`
//!
//! Configure the name and alias of sub command.
//!
//! * `hint`, `help`
//!
//! Configure the hint and help of help message.
//!
//! * `head`, `foot`
//!
//! Configure the head and foot of help message of sub command.
//!
//! ```rust
#![doc = include_str!("../test/14_help.rs")]
//! ```
//!
//! * `force`
//!
//! Configure the sub command optional, in default one of sub commands must be set.
//!
//! ```rust
#![doc = include_str!("../test/15_force.rs")]
//! ```
//!
//! ### `CoteOpt` Configurations list
//!
//! `CoteOpt` derive the default behavior of [`Infer`](crate::Infer), [`Fetch`](crate::Fetch`) and [`Alter`](crate::Alter).
//!
//! #### `infer`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `val`     |  true      | value type |
//!| `action`  |  true      | [`Action`](crate::Action) |
//!| `force`   |  true      | boolean |
//!| `ctor`    |  true      | [`AStr`](crate::aopt::AStr) |
//!| `index`   |  true      | Option<[`Index`](crate::Index)> |
//!| `style`   |  true      | Vec<[`Style`](crate::Style)> |
//!| `igname`  |  true      | boolean |
//!| `igalias` |  true      | boolean |
//!| `igindex` |  true      | boolean |
//!| `valid`   |  true      | Option<[`ValValidator`](crate::ValValidator)\<[`Val`](crate::Infer::Val)\>> |
//!| `init`    |  true      | Option<[`ValInitializer`](crate::ValInitializer)> |
//!| `ty`      |  true      | [`TypeId`](std::any::TypeId) |
//!| `tweak`   |  true      | function |
//!| `fill`    |  true      | function |
//!
//! `infer` can configure the behavior of [`Infer`](crate::Infer), the configures are mostly using to providing default value.
//!
//! ##### Example
//!
//! ```rust
#![doc = include_str!("../test/16_infer.rs")]
//! ```
//!
//! #### `alter`
//!
//! `alter` is reserve for future using.
//!
//! #### `fetch`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `inner`   |  true      |  type     |
//!| `map`     |  true      |  function |
//!| `scalar`  |  true      |  function |
//!| `vector`  |  true      |  function |
//!
//! `fetch` can configure the behavior of [`Fetch`](crate::Fetch).
//!
//! You can use `inner` and `map` configure the type and map function.
//! Or use `scalar` or `vector` configure the [`fetch`](crate::Fetch#method.fetch) and [`fetch_vec`](crate::Fetch#method.fetch_vec) separately.
//!
//! ##### Example
//!
//! ```rust
#![doc = include_str!("../test/17_fetch.rs")]
//! ```
//!
//! ### `CoteVal` Configurations list
//!
//! `CoteVal` derive the default behavior of [`RawValParser`](crate::RawValParser).
//!
//! #### `coteval`
//!
//!| name      | need value | available value |
//!|-----------|------------|-----------|
//!| `forward` |  true      |  type     |
//!| `map`     |  true      |  function |
//!| `mapraw`  |  true      |  function |
//!| `mapstr`  |  true      |  function |
//!| `igcase`  |  false     | |
//!| `name`    |  true      | string literal |
//!| `alias`   |  true      | string literal |
//!
//! `coteval` can configure the behavior of [`RawValParser`](crate::RawValParser).
//!
//! Using `forward` and `map` you can forward the call to another type, and then map the value to current type.
//! Or you can use `mapraw`, `mapstr` pass a parser called by [`parse`](crate::RawValParser#method.parse).
//!
//! `CoteVal` also support generate default parsing code for simple enum type.
//! For enum type, you can use `igcase` ignore case when matching, `name` configure the name of matching
//! or use `alias` add other names of matching.
//!
//! ##### Example 1
//!
//! ```rust
#![doc = include_str!("../test/18_value.rs")]
//! ```
//!
//! ##### Example of `mapraw` and `mapstr`
//!
//! ```rust
#![doc = include_str!("../test/19_map.rs")]
//! ```
