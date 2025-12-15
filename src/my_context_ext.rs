use async_compat::Compat;
use gpui::*;
use tracing::*;

pub trait MyContextExt<T: 'static> {
    fn my_spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<()>
    where
        T: 'static,
        AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> anyhow::Result<R> + 'static,
        R: 'static;

    fn my_spawn_in<AsyncFn, R>(&mut self, window: &Window, f: AsyncFn) -> Task<()>
    where
        R: 'static,
        AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncWindowContext) -> anyhow::Result<R> + 'static;

    fn my_listener<E: ?Sized, R>(
        &self,
        f: impl Fn(&mut T, &E, &mut Window, &mut Context<T>) -> anyhow::Result<R> + 'static,
    ) -> impl Fn(&E, &mut Window, &mut App) + 'static
    where
        R: 'static;
}

impl<T: 'static> MyContextExt<T> for Context<'_, T> {
    // 1. Add compatibility for using the tokio async runtime
    // 2. Use anyhow to log error information
    fn my_spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<()>
    // Here we return Task<()> instead of Task<R> because the purpose is only to log errors.
    // To get the original return value, use the original spawn method.
    where
        T: 'static,
        AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> anyhow::Result<R> + 'static,
        R: 'static,
    {
        self.spawn(async move |entity, cx| {
            let result = async move |entity, cx| {
                // Since reqwest is based on tokio, we need to use Compat for compatibility.
                Compat::new(async {
                    //
                    let r = f(entity, cx).await;
                    r
                })
                .await
            };
            let result = result(entity, cx).await;

            // Handle result and log error if any
            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed: {:?}", err);
                }
            }
        })
    }

    // 1. Add compatibility for using the tokio async runtime
    // 2. Use anyhow to log error information
    fn my_spawn_in<AsyncFn, R>(&mut self, window: &Window, f: AsyncFn) -> Task<()>
    // Here we return Task<()> instead of Task<R> because the purpose is only to log errors.
    // To get the original return value, use the original spawn_in method.
    where
        T: 'static,
        AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncWindowContext) -> anyhow::Result<R> + 'static,
    {
        self.spawn_in(window, async move |entity, window| {
            let result = async move |entity, window| {
                // Since reqwest is based on tokio, we need to use Compat for compatibility.
                Compat::new(async {
                    //
                    let r = f(entity, window).await;
                    r
                })
                .await
            };
            let result = result(entity, window).await;

            // Handle result and log error if any
            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed: {:?}", err);
                }
            }
        })
    }

    // Use anyhow to log error information
    fn my_listener<E: ?Sized, R>(
        &self,
        f: impl Fn(&mut T, &E, &mut Window, &mut Context<T>) -> anyhow::Result<R> + 'static,
    ) -> impl Fn(&E, &mut Window, &mut App) + 'static
    where
        R: 'static,
    {
        self.listener(move |this, event, window, cx| {
            let result = f(this, event, window, cx);

            match result {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed: {:?}", err);
                }
            }
        })
    }
}

pub fn listener_box<Ev: ?Sized, T>(
    weak_entity: WeakEntity<T>,
    f: impl Fn(&mut T, &Ev, &mut Window, &mut Context<T>) + 'static,
) -> Box<dyn Fn(&Ev, &mut Window, &mut App) + 'static>
where
    T: 'static,
{
    return Box::new({
        move |e: &Ev, window: &mut Window, cx: &mut App| {
            weak_entity
                .update(cx, |view, cx| f(view, e, window, cx))
                .ok();
        }
    });
}

pub fn my_listener_box<Ev: ?Sized, T, R>(
    weak_entity: WeakEntity<T>,
    f: impl Fn(&mut T, &Ev, &mut Window, &mut Context<T>) -> anyhow::Result<R> + 'static,
) -> Box<dyn Fn(&Ev, &mut Window, &mut App) + 'static>
where
    T: 'static,
    R: 'static,
{
    listener_box(weak_entity, move |this, event, window, cx| {
        let result = f(this, event, window, cx);

        match result {
            Ok(_) => {}
            Err(err) => {
                error!("Failed: {:?}", err);
            }
        }
    })
}
