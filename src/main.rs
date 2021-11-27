use gems::GemContext;
use gems::rt::EvaluatorRuntime;

fn main() {
    let ctx = GemContext::new();
    let iter = EvaluatorRuntime::new(generator(&ctx), &ctx);
    for i in iter {
        println!("{}", i);
    }
}

async fn generator(ctx: &GemContext<u32>) {
    for i in 0..10 {
        ctx.r#yield(i).await;
    }
}