mod graphice;

pub use graphice::gl;
pub use graphice::*;

pub trait CreateContext {
    fn create_context(&mut self) -> GraphicsContext;
}

impl CreateContext for glfw::Window {
    fn create_context(&mut self) -> GraphicsContext {
        let loader = |proc: &str| unsafe { std::mem::transmute(self.get_proc_address(proc)) };
        gl::load_gl_funcs(loader);
        let mut context = GraphicsContext::new(unsafe { gl::is_gl2() });
        context.window = Some(self as *mut glfw::Window);
        context
    }
}
