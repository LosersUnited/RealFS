use may_minihttp::{HttpServer, HttpService, Request, Response};
use std::io;
mod handlers;
mod http_lib;
mod utils_lib;
#[derive(Clone)]
struct RealFS;

impl HttpService for RealFS {
    fn call(&mut self, req: Request, res: &mut Response) -> io::Result<()> {
        let path = req.path();
        let method = req.method();
        if path.starts_with(handlers::read::BASE) && method == handlers::read::METHOD {
            return handlers::read::handle_read(
                req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
                res,
            );
        }
        if path.starts_with(handlers::list::BASE) && method == handlers::list::METHOD {
            return handlers::list::handle_list(
                req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
                res,
            );
        }
        if path.starts_with(handlers::stat::BASE) && method == handlers::stat::METHOD {
            return handlers::stat::handle_stat(
                req,
                (std::env::args().collect::<Vec<String>>()[1]).as_str(),
                res,
            );
        }
        Ok(())
    }
}

fn main() {
    let server = HttpServer(RealFS).start("0.0.0.0:2137").unwrap();
    server.join().unwrap();
}
