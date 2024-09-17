use may_minihttp::{HttpServer, HttpService, Request, Response};
use std::io;
mod handlers;
mod http_lib;
mod utils_lib;
#[derive(Clone)]
struct RealFS;

static mut MNT: &str = "";

impl HttpService for RealFS {
    fn call(&mut self, req: Request, res: &mut Response) -> io::Result<()> {
        // res.header("Access-Control-Allow-Origin: *");
        // dbg!(unsafe { MNT });
        let path = req.path();
        let method = req.method();
        if path.starts_with(handlers::read::BASE) && method == handlers::read::METHOD {
            return handlers::read::handle_read(req, unsafe { MNT }, res);
        }
        if path.starts_with(handlers::list::BASE) && method == handlers::list::METHOD {
            return handlers::list::handle_list(req, unsafe { MNT }, res);
        }
        if path.starts_with(handlers::stat::BASE) && method == handlers::stat::METHOD {
            return handlers::stat::handle_stat(req, unsafe { MNT }, res);
        }
        if path.starts_with(handlers::write::BASE) && method == handlers::write::METHOD {
            return handlers::write::handle_write(req, unsafe { MNT }, res);
        }
        Ok(())
    }
}

fn main() {
    may::config().set_stack_size(4096 * 2);
    unsafe {
        let t = Box::into_raw(Box::new(std::env::args().collect::<Vec<String>>()));
        let target = t.as_ref().unwrap().get(1).unwrap();
        println!("{target}");
        let target_raw = String::from(target);
        let target_raw2 = Box::into_raw(Box::new(target_raw));
        MNT = target_raw2.as_ref().unwrap().as_str();
        t.drop_in_place();
    };
    let server = HttpServer(RealFS).start("0.0.0.0:2137").unwrap();
    server.join().unwrap();
}
