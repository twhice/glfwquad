use super::*;
/// `ElapsedQuery` is used to measure duration of GPU operations.
///
/// Usual timing/profiling methods are difficult apply to GPU workloads as draw calls are submitted
/// asynchronously effectively hiding execution time of individual operations from the user.
/// `ElapsedQuery` allows to measure duration of individual rendering operations, as though the time
/// was measured on GPU rather than CPU side.
///
/// The query is created using [`ElapsedQuery::new()`] function.
/// ```
/// use miniquad::graphics::ElapsedQuery;
/// // initialization
/// let mut query = ElapsedQuery::new();
/// ```
/// Measurement is performed by calling [`ElapsedQuery::begin_query()`] and
/// [`ElapsedQuery::end_query()`]
///
/// ```
/// # use miniquad::graphics::ElapsedQuery;
/// # let mut query = ElapsedQuery::new();
///
/// query.begin_query();
/// // one or multiple calls to miniquad::Context::draw()
/// query.end_query();
/// ```
///
/// Retreival of measured duration is only possible at a later point in time. Often a frame or
/// couple frames later. Measurement latency can especially be high on WASM/WebGL target.
///
/// ```
/// // couple frames later:
/// # use miniquad::graphics::ElapsedQuery;
/// # let mut query = ElapsedQuery::new();
/// # query.begin_query();
/// # query.end_query();
/// if query.is_available() {
///   let duration_nanoseconds = query.get_result();
///   // use/display duration_nanoseconds
/// }
/// ```
///
/// And during finalization:
/// ```
/// // clean-up
/// # use miniquad::graphics::ElapsedQuery;
/// # let mut query = ElapsedQuery::new();
/// # query.begin_query();
/// # query.end_query();
/// # if query.is_available() {
/// #   let duration_nanoseconds = query.get_result();
/// #   // use/display duration_nanoseconds
/// # }
/// query.delete();
/// ```
///
/// It is only possible to measure single query at once.
///
/// On OpenGL/WebGL platforms implementation relies on [`EXT_disjoint_timer_query`] extension.
///
/// [`EXT_disjoint_timer_query`]: https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_disjoint_timer_query.txt
///
#[derive(Clone)]
pub struct ElapsedQuery {
    gl_query: GLuint,
}

impl ElapsedQuery {
    pub fn new() -> ElapsedQuery {
        ElapsedQuery { gl_query: 0 }
    }

    /// Submit a beginning of elapsed-time query.
    ///
    /// Only a single query can be measured at any moment in time.
    ///
    /// Use [`ElapsedQuery::end_query()`] to finish the query and
    /// [`ElapsedQuery::get_result()`] to read the result when rendering is complete.
    ///
    /// The query can be used again after retriving the result.
    ///
    /// Implemented as `glBeginQuery(GL_TIME_ELAPSED, ...)` on OpenGL/WebGL platforms.
    ///
    /// Use [`ElapsedQuery::is_supported()`] to check if functionality is available and the method can be called.
    pub fn begin_query(&mut self) {
        if self.gl_query == 0 {
            unsafe { glGenQueries(1, &mut self.gl_query) };
        }
        unsafe { glBeginQuery(GL_TIME_ELAPSED, self.gl_query) };
    }

    /// Submit an end of elapsed-time query that can be read later when rendering is complete.
    ///
    /// This function is usd in conjunction with [`ElapsedQuery::begin_query()`] and
    /// [`ElapsedQuery::get_result()`].
    ///
    /// Implemented as `glEndQuery(GL_TIME_ELAPSED)` on OpenGL/WebGL platforms.
    pub fn end_query(&mut self) {
        unsafe { glEndQuery(GL_TIME_ELAPSED) };
    }

    /// Retreieve measured duration in nanonseconds.
    ///
    /// Note that the result may be ready only couple frames later due to asynchronous nature of GPU
    /// command submission. Use [`ElapsedQuery::is_available()`] to check if the result is
    /// available for retrieval.
    ///
    /// Use [`ElapsedQuery::is_supported()`] to check if functionality is available and the method can be called.
    pub fn get_result(&self) -> u64 {
        // let mut time: GLuint64 = 0;
        // assert!(self.gl_query != 0);
        // unsafe { glGetQueryObjectui64v(self.gl_query, GL_QUERY_RESULT, &mut time) };
        // time
        0
    }

    /// Reports whenever elapsed timer is supported and other methods can be invoked.
    pub fn is_supported() -> bool {
        unimplemented!();
        //unsafe { sapp_is_elapsed_timer_supported() }
    }

    /// Reports whenever result of submitted query is available for retrieval with
    /// [`ElapsedQuery::get_result()`].
    ///
    /// Note that the result may be ready only couple frames later due to asynchrnous nature of GPU
    /// command submission.
    ///
    /// Use [`ElapsedQuery::is_supported()`] to check if functionality is available and the method can be called.
    pub fn is_available(&self) -> bool {
        // let mut available: GLint = 0;

        // // begin_query was not called yet
        // if self.gl_query == 0 {
        //     return false;
        // }

        //unsafe { glGetQueryObjectiv(self.gl_query, GL_QUERY_RESULT_AVAILABLE, &mut available) };
        //available != 0

        false
    }
}

impl Drop for ElapsedQuery {
    fn drop(&mut self) {
        unsafe { glDeleteQueries(1, &mut self.gl_query) }
        self.gl_query = 0;
    }
}
