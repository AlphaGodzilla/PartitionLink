use std::sync::Arc;
use std::time::Duration;

use jni::objects::{JClass, JString};
use jni::JNIEnv;

use config::Config;
use log::debug;
use runtime::Runtime;
use tokio::time::sleep;
use tokio_context::context::RefContext;

mod cmd_server;
mod command;
mod config;
mod db;
mod discover;
mod node;
mod protocol;
mod runtime;
mod until;

#[no_mangle]
pub extern "C" fn Java_HelloWorld_hello<'local>(
    // Notice that this `env` argument is mutable. Any `JNIEnv` API that may
    // allocate new object references will take a mutable reference to the
    // environment.
    mut env: JNIEnv<'local>,
    // this is the class that owns our static method. Not going to be used, but
    // still needs to have an argument slot
    _class: JClass<'local>,
    input: JString<'local>,
) -> JString<'local> {
    // First, we have to get the string out of java. Check out the `strings`
    // module for more info on how this works.
    let input: String = env
        .get_string(&input)
        .expect("Couldn't get java string!")
        .into();

    // Then we have to create a new java string to return. Again, more info
    // in the `strings` module.
    let msg = format!("Hello, {}!", input);
    println!("==== {}", &msg);
    let output = env.new_string(msg).expect("Couldn't create java string!");
    output
}

#[no_mangle]
pub extern "C" fn Java_TokioRuntime_start(mut env: JNIEnv, _class: JClass) {
    main1();
}

pub fn main1() {
    println!("==== 准备启动");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { main0().await.unwrap() });
    println!("===== 结束");
}

pub async fn main0() -> anyhow::Result<()> {
    env_logger::init();

    let cfg = Arc::new(Config::default());

    let rt = Runtime::new();

    let (ctx, ctx_handler) = RefContext::new();

    let (discover_handler, command_handler) = rt.start(&ctx, cfg.clone())?;

    sleep(Duration::from_secs(60)).await;

    ctx_handler.cancel();

    discover_handler.await?;
    command_handler.await?;
    Ok(())
}
