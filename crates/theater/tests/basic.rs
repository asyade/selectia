use theater::{dispatcher::channel_iterator, prelude::*};

#[derive(Debug, Clone, Task)]
pub struct ServiceATask {
    callback: TaskCallback<()>,
}

#[derive(Debug, Clone, Task)]
pub struct ServiceAEvent {}

#[singleton_service(ServiceA)]
pub async fn service_a(
    _ctx: ServiceContext,
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

#[singleton_service(ServiceB)]
pub async fn service_b(
    _ctx: ServiceContext,
    mut rx: ServiceReceiver<ServiceAEvent>,
) -> TheaterResult<()> {
    tracing::info!("ServiceB spawned");
    let msg = rx.recv().await.unwrap();
    tracing::info!("ServiceB received event: {:?}", msg);
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

    tracing::info!("Starting basic system");

    let ctx = OwnedTheaterContext::new().await;

    ServiceA::spawn(&*ctx, "hello".to_string()).await.unwrap();

    ctx.ready().await;
    let svg = ctx.get_singleton_address::<ServiceA>().await.unwrap();
    let dispatcher = ctx
        .get_singleton_dispatcher::<ServiceA, ServiceAEvent>()
        .await
        .unwrap();

    dispatcher
        .register(ServiceB::spawn(&*ctx).await.unwrap())
        .await;

    dispatcher
        .register(channel_iterator(|event| {
            println!("Received event: {:?}", event);
        }))
        .await;

    let (callback, recv) = TaskCallback::new();
    svg.send(ServiceATask { callback }).await.unwrap();

    let _ = recv.wait().await;
}
