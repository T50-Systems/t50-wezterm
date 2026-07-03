struct LuaPipe {
    sender: Sender<mlua::Lua>,
    receiver: Receiver<mlua::Lua>,
}
impl LuaPipe {
    pub fn new() -> Self {
        let (sender, receiver) = smol::channel::unbounded();
        Self { sender, receiver }
    }
}

/// The implementation is only slightly crazy...
/// `Lua` is Send but !Sync.
/// We take care to reference this only from the main thread of
/// the application.
/// We also need to take care to keep this `lua` alive if a long running
/// future is outstanding while a config reload happens.
/// We have to use `Rc` to manage its lifetime, but due to some issues
/// with rust's async lifetime tracking we need to indirectly schedule
/// some of the futures to avoid it thinking that the generated future
/// in the async block needs to be Send.
///
/// A further complication is that config reloading tends to happen in
/// a background filesystem watching thread.
///
/// The result of all these constraints is that the LuaPipe struct above
/// is used as a channel to transport newly loaded lua configs to the
/// main thread.
///
/// The main thread pops the loaded configs to obtain the latest one
/// and updates LuaConfigState
struct LuaConfigState {
    lua: Option<Rc<mlua::Lua>>,
}

impl LuaConfigState {
    /// Consume any lua contexts sent to us via the
    /// config loader until we end up with the most
    /// recent one being referenced by LUA_CONFIG.
    fn update_to_latest(&mut self) {
        while let Ok(lua) = LUA_PIPE.receiver.try_recv() {
            self.lua.replace(Rc::new(lua));
        }
    }

    /// Take a reference on the latest generation of the lua context
    fn get_lua(&self) -> Option<Rc<mlua::Lua>> {
        self.lua.as_ref().map(Rc::clone)
    }
}

pub fn designate_this_as_the_main_thread() {
    LUA_CONFIG.with(|lc| {
        let mut lc = lc.borrow_mut();
        if lc.is_none() {
            lc.replace(LuaConfigState { lua: None });
        }
    });
}

#[must_use = "Cancels the subscription when dropped"]
pub struct ConfigSubscription(usize);

impl Drop for ConfigSubscription {
    fn drop(&mut self) {
        CONFIG.unsub(self.0);
    }
}

pub fn subscribe_to_config_reload<F>(subscriber: F) -> ConfigSubscription
where
    F: Fn() -> bool + 'static + Send,
{
    ConfigSubscription(CONFIG.subscribe(subscriber))
}

/// Spawn a future that will run with an optional Lua state from the most
/// recently loaded lua configuration.
/// The `func` argument is passed the lua state and must return a Future.
///
/// This function MUST only be called from the main thread.
/// In exchange for the caller checking for this, the parameters to
/// this method are not required to be Send.
///
/// Calling this function from a secondary thread will panic.
/// You should use `with_lua_config` if you are triggering a
/// call from a secondary thread.
pub async fn with_lua_config_on_main_thread<F, RETF, RET>(func: F) -> anyhow::Result<RET>
where
    F: FnOnce(Option<Rc<mlua::Lua>>) -> RETF,
    RETF: Future<Output = anyhow::Result<RET>>,
{
    let lua = LUA_CONFIG.with(|lc| {
        let mut lc = lc.borrow_mut();
        let lc = lc.as_mut().expect(
            "with_lua_config_on_main_thread not called
             from main thread, use with_lua_config instead!",
        );
        lc.update_to_latest();
        lc.get_lua()
    });

    func(lua).await
}

pub fn run_immediate_with_lua_config<F, RET>(func: F) -> anyhow::Result<RET>
where
    F: FnOnce(Option<Rc<mlua::Lua>>) -> anyhow::Result<RET>,
{
    let lua = LUA_CONFIG.with(|lc| {
        let mut lc = lc.borrow_mut();
        let lc = lc.as_mut().expect(
            "with_lua_config_on_main_thread not called
             from main thread, use with_lua_config instead!",
        );
        lc.update_to_latest();
        lc.get_lua()
    });

    func(lua)
}

fn schedule_with_lua<F, RETF, RET>(func: F) -> promise::spawn::Task<anyhow::Result<RET>>
where
    F: 'static,
    RET: 'static,
    F: Fn(Option<Rc<mlua::Lua>>) -> RETF,
    RETF: Future<Output = anyhow::Result<RET>>,
{
    promise::spawn::spawn(async move { with_lua_config_on_main_thread(func).await })
}

/// Spawn a future that will run with an optional Lua state from the most
/// recently loaded lua configuration.
/// The `func` argument is passed the lua state and must return a Future.
pub async fn with_lua_config<F, RETF, RET>(func: F) -> anyhow::Result<RET>
where
    F: Fn(Option<Rc<mlua::Lua>>) -> RETF,
    RETF: Future<Output = anyhow::Result<RET>> + Send + 'static,
    F: Send + 'static,
    RET: Send + 'static,
{
    promise::spawn::spawn_into_main_thread(async move { schedule_with_lua(func).await }).await
}
