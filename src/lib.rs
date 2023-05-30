pub mod graphics;
pub use glfw;
pub use graphics::gl;

pub trait CreateContext {
    fn create_context(&mut self) -> graphics::GraphicsContext;
}

impl CreateContext for glfw::Window {
    fn create_context(&mut self) -> graphics::GraphicsContext {
        let loader = |proc: &str| unsafe { std::mem::transmute(self.get_proc_address(proc)) };
        gl::load_gl_funcs(loader);
        let mut context = graphics::GraphicsContext::new(unsafe { gl::is_gl2() });
        context.window = Some(self as *mut glfw::Window);
        context
    }
}

#[cfg(test)]
mod tests {
    use glfw::Context;
    use graphics::*;

    use super::*;

    #[test]
    fn test_name() -> Result<(), Box<dyn std::error::Error>> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;
        let (mut window, receiver) = glfw
            .create_window(800, 450, "Test ", glfw::WindowMode::Windowed)
            .ok_or("未能创建窗口")?;

        let mut context = window.create_context();
        let ctx = &mut context;
        while !window.should_close() {
            glfw.poll_events();
            for (_time, event) in glfw::flush_messages(&receiver) {
                match event {
                    glfw::WindowEvent::Close => window.set_should_close(true),
                    _ => {}
                }
            }

            let pass_action = pass::PassAction::Clear(Clear::default().color(1.0, 1.0, 1.0, 1.0));

            ctx.begin_pass(None, pass_action)
                .end_render_pass()
                .commit_frame();
            window.swap_buffers();
        }

        Ok(())
    }
}
