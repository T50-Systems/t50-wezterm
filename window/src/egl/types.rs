struct EglWrapper {
    _lib: libloading::Library,
    egl: ffi::Egl,
}

impl std::fmt::Debug for EglWrapper {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("EglWrapper").finish()
    }
}

#[derive(Debug)]
pub struct GlConnection {
    egl: EglWrapper,
    display: ffi::types::EGLDisplay,
    is_opengl: bool,
    extensions: String,
}

impl GlConnection {
    #[allow(dead_code)]
    pub fn has_extension(&self, wanted: &str) -> bool {
        self.extensions.split(' ').any(|ext| ext == wanted)
    }
}

impl std::ops::Deref for GlConnection {
    type Target = ffi::Egl;

    fn deref(&self) -> &ffi::Egl {
        &self.egl.egl
    }
}

impl Drop for GlConnection {
    fn drop(&mut self) {
        unsafe {
            self.egl.egl.Terminate(self.display);
        }
    }
}

#[derive(Debug)]
pub struct GlState {
    connection: Rc<GlConnection>,
    surface: ffi::types::EGLSurface,
    context: ffi::types::EGLContext,
}

impl Drop for GlState {
    fn drop(&mut self) {
        unsafe {
            self.connection.MakeCurrent(
                self.connection.display,
                ffi::NO_SURFACE,
                ffi::NO_SURFACE,
                ffi::NO_CONTEXT,
            );
            self.connection
                .DestroySurface(self.connection.display, self.surface);
            self.connection
                .DestroyContext(self.connection.display, self.context);
        }
    }
}

type GetProcAddressFunc =
    unsafe extern "C" fn(*const std::os::raw::c_char) -> *const std::os::raw::c_void;
