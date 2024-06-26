//! Optional addon for running the main application loop.

use std::ffi::c_void;

use crate::{
    core::{c_types::WorldT, world::World, FTime},
    sys::{
        ecs_app_desc_t, ecs_app_init_action_t, ecs_app_run, ecs_fini, ecs_get_world_info,
        ecs_should_quit,
    },
};

/// Application interface.
pub struct App {
    world: *mut WorldT,
    desc: ecs_app_desc_t,
}

impl App {
    /// Create a new application.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to run the application on.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::app_builder`
    #[doc(alias = "app_builder::app_builder")]
    pub fn new(world: &World) -> Self {
        let mut obj = Self {
            world: world.raw_world,
            desc: ecs_app_desc_t::default(),
        };

        let stats = unsafe { ecs_get_world_info(world.raw_world) };
        obj.desc.target_fps = unsafe { (*stats).target_fps };
        let zero: FTime = 0.0;
        if obj.desc.target_fps.to_bits() == zero.to_bits() {
            obj.desc.target_fps = 60.0;
        }
        obj
    }

    /// Set the target frames per second.
    ///
    /// # Arguments
    ///
    /// * `fps` - The target frames per second.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::target_fps`
    #[doc(alias = "app_builder::target_fps")]
    pub fn set_target_fps(&mut self, fps: FTime) -> &mut Self {
        self.desc.target_fps = fps;
        self
    }

    /// Set the time delta.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time delta.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::delta_time`
    #[doc(alias = "app_builder::delta_time")]
    pub fn set_delta_time(&mut self, delta_time: FTime) -> &mut Self {
        self.desc.delta_time = delta_time;
        self
    }

    /// Set the number of threads.
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::threads`
    #[doc(alias = "app_builder::threads")]
    pub fn set_threads(&mut self, threads: i32) -> &mut Self {
        self.desc.threads = threads;
        self
    }

    /// Set the number of frames.
    ///
    /// # Arguments
    ///
    /// * `frames` - The number of frames.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::frames`
    #[doc(alias = "app_builder::frames")]
    pub fn set_frames(&mut self, frames: i32) -> &mut Self {
        self.desc.frames = frames;
        self
    }

    /// Enable the REST API.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::enable_rest`
    #[doc(alias = "app_builder::enable_rest")]
    #[cfg(feature = "flecs_rest")]
    pub fn enable_rest(&mut self, port: u16) -> &mut Self {
        self.desc.enable_rest = true;
        self.desc.port = port;
        self
    }

    /// Enable the monitor.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable the monitor.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::enable_monitor`
    #[doc(alias = "app_builder::enable_monitor")]
    #[cfg(feature = "flecs_monitor")]
    pub fn enable_monitor(&mut self, enable: bool) -> &mut Self {
        self.desc.enable_monitor = enable;
        self
    }

    // TODO change this to FnMut(&mut World) -> cint
    /// Set the application init action.
    ///
    /// # Arguments
    ///
    /// * `value` - The init action.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::init`
    #[doc(alias = "app_builder::init")]
    pub fn init(&mut self, value: ecs_app_init_action_t) -> &mut Self {
        self.desc.init = value;
        self
    }

    /// Set the application context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::ctx`
    #[doc(alias = "app_builder::ctx")]
    pub fn context(&mut self, ctx: *mut c_void) -> &mut Self {
        self.desc.ctx = ctx;
        self
    }

    /// Run application. This will run the application with the parameters specified in desc.
    /// After the application quits (`ecs_quit`() is called) the world will be cleaned up.
    /// If a custom run action is set, it will be invoked by this operation.
    /// The default run action calls the frame action in a loop until it returns a non-zero value.
    ///
    /// # Returns
    ///
    /// The exit code of the application.
    ///
    /// # See also
    ///
    /// * C++ API: `app_builder::run`
    #[doc(alias = "app_builder::run")]
    pub fn run(&mut self) -> i32 {
        unsafe {
            let result = ecs_app_run(self.world, &mut self.desc);
            if ecs_should_quit(self.world) {
                // Only free world if quit flag is set. This ensures that we won't
                // try to cleanup the world if the app is used in an environment
                // that takes over the main loop, like with emscripten.
                ecs_fini(self.world);
            }
            result
        }
    }
}
