use theater::prelude::*;

#[derive(Debug, Clone, Task)]
pub struct ServiceATask {
    callback: TaskCallback<()>,
}

#[derive(Debug, Clone, Task)]
pub struct ServiceAEvent {}


#[singleton_service(ServiceA)]
pub async fn service_a(
    ctx: ServiceContext,
    mut rx: ServiceReceiver<ServiceATask>,
    dispatcher: EventDispatcher<ServiceAEvent>,
    params: String,
) -> TheaterResult<()> {
    tracing::info!("ServiceA spawned with params: {}", params);
    dispatcher.dispatch(ServiceAEvent {}).await.unwrap();
    let msg = rx.recv().await;
    msg.unwrap().callback.resolve(()).await.unwrap();
    Ok(())
}


#[tokio::test]
async fn basic_system() {
    tracing_subscriber::fmt()
        .with_env_filter("trace")
        .pretty()
        .with_file(true)
        .fmt_fields(tracing_subscriber::fmt::format::PrettyFields::new())
        .init();

    let ctx = OwnedTheaterContext::new().await;

    ServiceA::spawn(&*ctx, "hello".to_string()).await.unwrap();

    ctx.ready().await;
    let svg = ctx
        .get_singleton_address::<ServiceA, _>()
        .await
        .unwrap();

    let (callback, recv) = TaskCallback::new();
    svg.send(ServiceATask { callback }).await.unwrap();
    let _ = recv.wait().await;
}


//TODO: test launching a service when the context is ready (spoiler alert: it's not working)