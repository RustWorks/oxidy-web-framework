use crate::libs::handler::handler;
use crate::libs::threadpool::ThreadPool;
use crate::structs::Context;
use crate::structs::Middleware;
use std::io::ErrorKind::WouldBlock;
use std::net::TcpListener;

pub(crate) type MiddlewareCallback = fn(&mut Context) -> Middleware;

#[derive(Clone)]
pub struct Server {
    pub(crate) middlewares: Vec<MiddlewareCallback>,
    pub(crate) gets: Vec<(String, fn(&mut Context) -> ())>,
    pub(crate) posts: Vec<(String, fn(&mut Context) -> ())>,
    pub(crate) puts: Vec<(String, fn(&mut Context) -> ())>,
    pub(crate) deletes: Vec<(String, fn(&mut Context) -> ())>,
    pub(crate) patchs: Vec<(String, fn(&mut Context) -> ())>,
    pub(crate) catchs: Option<fn(&mut Context) -> ()>,
    pub(crate) allow_threads: usize,
}

impl Server {
    /// Middlewares
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::structs::Middleware;
    /// use oxidy::server::Server;
    ///
    /// fn mid (_: &mut Context) -> Middleware {
    ///     println!("Middleware Function");
    ///     let start = Instant::now();
    ///     (
    ///         true,
    ///         Some(Box::new(move |_: &mut Context| {
    ///             let end = Instant::now();
    ///             println!("Response takes: {:?}", end - start);
    ///         })),
    ///     )
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.middleware(mid);
    /// assert_eq!((), a);
    /// ```
    pub fn middleware(&mut self, callback: MiddlewareCallback) -> () {
        self.middlewares.push(callback);
    }
    /// GET Route
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn index (_: &mut Context) -> () {
    ///     println!("Index GET Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.get("/", index);
    /// assert_eq!((), a);
    /// ```
    pub fn get(&mut self, path: &str, callback: fn(&mut Context) -> ()) -> () {
        self.gets.push((path.to_string(), callback));
    }
    /// POST Route
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn user (_: &mut Context) -> () {
    ///     println!("User POST Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.post("/", user);
    /// assert_eq!((), a);
    /// ```
    pub fn post(&mut self, path: &str, callback: fn(&mut Context) -> ()) -> () {
        self.posts.push((path.to_string(), callback));
    }
    /// PUT Route
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn user (_: &mut Context) -> () {
    ///     println!("User PUT Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.put("/", user);
    /// assert_eq!((), a);
    /// ```
    pub fn put(&mut self, path: &str, callback: fn(&mut Context) -> ()) -> () {
        self.puts.push((path.to_string(), callback));
    }
    /// DELETE Route
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn user (_: &mut Context) -> () {
    ///     println!("User DELETE Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.delete("/", user);
    /// assert_eq!((), a);
    /// ```
    pub fn delete(&mut self, path: &str, callback: fn(&mut Context) -> ()) -> () {
        self.deletes.push((path.to_string(), callback));
    }
    /// PATCH Route
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn user (_: &mut Context) -> () {
    ///     println!("User PATCH Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.patch("/", user);
    /// assert_eq!((), a);
    /// ```
    pub fn patch(&mut self, path: &str, callback: fn(&mut Context) -> ()) -> () {
        self.patchs.push((path.to_string(), callback));
    }
    /// CATCH Method
    ///
    /// Catch Function will call on any HTTP Error
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::structs::Context;
    /// use oxidy::server::Server;
    ///
    /// fn catch (_: &mut Context) -> () {
    ///     println!("CATCH Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// let a = app.catch(catch);
    /// assert_eq!((), a);
    /// ```
    pub fn catch(&mut self, callback: fn(&mut Context) -> ()) -> () {
        self.catchs = Some(callback);
    }
    /// Multi Threading
    ///
    /// Number of Threads
    /// Default is 0 (Zero) Number of total CPU CORE
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::server::Server;
    ///
    /// let mut app = Server::new();
    /// let a = app.threads(1);
    /// assert_eq!((), a);
    /// ```
    pub fn threads(&mut self, allow: usize) -> () {
        self.allow_threads = allow;
    }
    /* /// Listen
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::server::Server;
    ///
    /// let mut app = Server::new();
    /// let a = app.listen("127.0.0.1:3000");
    /// assert_eq!((), a);
    /// ```*/
    pub fn listen(&self, address: &'static str) -> () {
        /*
         * Bind Server
         */
        let server: TcpListener = TcpListener::bind(address).unwrap();
        /*
         * Set Non Blocking
         */
        server.set_nonblocking(true).unwrap();
        /*
         * Thread Pool
         */
        let pool: ThreadPool = ThreadPool::new(self.allow_threads);
        /*
         * Log
         */
        println!("Listening [{}]", address);
        /*
         * Handle Client
         */
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
                    let server_cp = self.clone();
                    pool.execute(move || handler(stream, server_cp));
                }
                Err(ref e) if e.kind() == WouldBlock => continue,
                Err(e) => println!("Error: Stream Failed: {}", e),
            }
        }
    }
}
/// New Server Instence
///
/// # Example
///
/// ```
/// use oxidy::server::Server;
/// use oxidy::structs::Context;
///
/// fn index (_: &mut Context) -> () {
///     println!("Index GET Function");
/// }
///
/// fn user (_: &mut Context) -> () {
///     println!("User POST Function");
/// }
///
/// let mut app = Server::new();
/// app.get("/", index);
/// app.post("/", user);
/// ```
impl Server {
    /// New Server Instence
    ///
    /// # Example
    ///
    /// ```
    /// use oxidy::server::Server;
    /// use oxidy::structs::Context;
    ///
    /// fn index (_: &mut Context) -> () {
    ///     println!("Index GET Function");
    /// }
    ///
    /// fn user (_: &mut Context) -> () {
    ///     println!("User POST Function");
    /// }
    ///
    /// let mut app = Server::new();
    /// app.get("/", index);
    /// app.post("/", user);
    /// ```
    pub fn new() -> Server {
        Server {
            middlewares: Vec::new(),
            gets: Vec::new(),
            posts: Vec::new(),
            puts: Vec::new(),
            deletes: Vec::new(),
            patchs: Vec::new(),
            catchs: None,
            allow_threads: 0,
        }
    }
}