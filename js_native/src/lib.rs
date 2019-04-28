//!
//! duktape引擎的safe包装
//! 

#[macro_use]
extern crate error_chain;
extern crate typemap;
#[macro_use]
extern crate bitflags;
extern crate dukbind;

mod callable;
pub mod class;
mod ctx;
pub mod error;
mod macros;
mod privates;
pub mod types;

pub use self::callable::Callable;
pub use self::ctx::*;
pub use self::macros::*;
pub use self::typemap::Key;

pub mod prelude {
    pub use super::callable::Callable;
    pub use super::class;
    pub use super::ctx::*;
    pub use super::error::Error as DukError;
    pub use super::error::ErrorKind as DukErrorKind;
    pub use super::error::Result as DukResult;
    pub use super::macros::*;
    pub use super::types::*;
}

#[cfg(test)]
mod test {
    use super::prelude::*;
    #[test]
    fn test() -> DukResult<()> {
        let ctx = DukContext::new()?;

        let mut builder = class::build();

        let global: Object = ctx.push_global_object().getp()?;

        builder.method(
            "greet",
            (1, |ctx: &DukContext, _this: &mut class::Instance| {
                let name = ctx.get::<String>(0)?;
                ctx.push(format!("Hello {}", name))?;
                Ok(1)
            }),
        );

        global.set("Greeter", builder);

        let greeting: String = ctx
            .eval(
                r#"
    
                    var greeter = new Greeter();

                    var greeting = greeter.greet('me');
                    greeting + '!';
                    "#,
            )?
            .get(-1)?;

        assert_eq!(greeting, "Hello me!");
        println!("{}", greeting);

        let greeter: Object = ctx.get_global_string("Greeter").construct(0)?.getp()?;
        println!("{}", greeter.call::<_, _, String>("greet", "eevee")?);

        Ok(())
    }
}
