mod builder;
mod method;

pub use self::builder::*;
pub use self::method::{Instance, Method};

pub fn build<'a>() -> Builder<'a> {
    Builder::default()
}

#[cfg(test)]
pub mod tests {

    use super::super::ctx::DukContext;
    use super::super::types::Object;
    use super::method::Instance;
    #[test]
    fn class_builder() {
        let ctx = DukContext::new().unwrap();

        let mut b = super::build();

        b.method(
            "testMethodNoArg",
            |ctx: &DukContext, _this: &mut Instance| {
                ctx.push_string("Hello, World!");
                Ok(1)
            },
        )
        .method(
            "testMethodArg",
            (1, |_ctx: &DukContext, _this: &mut Instance| Ok(0)),
        );

        ctx.push_class(b).unwrap();
        ctx.construct(0).unwrap();

        let out = ctx.getp::<Object>().unwrap();

        let greeting = out.call::<_, _, String>("testMethodNoArg", ()).unwrap();
        assert_eq!(greeting, "Hello, World!");
    }
}
