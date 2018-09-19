extern crate futures;
extern crate reqwest;
extern crate tokio;
extern crate clap;

use clap::{Arg, App};

use std::mem;
use std::io::{self, Cursor};
use futures::{Future, Stream};
use reqwest::async::{Client, Decoder};
use tokio::runtime::Runtime;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

pub struct TokioRuntimeWrapper {
    rt: Rc<RefCell<Runtime>>,
}

impl TokioRuntimeWrapper {
    pub fn new() -> Self {
        TokioRuntimeWrapper {
            rt: Rc::new(RefCell::new(Runtime::new().unwrap())),
        }
    }

    pub fn fetch(url: &str) -> impl Future<Item=(), Error=()> {
        let client = Client::builder()
            .build()
            .unwrap();

        client
            .get(url)
            .send()
            .and_then(|mut res| {
                println!("{}", res.status());
                println!("{:?}", res);
                let body = mem::replace(res.body_mut(), Decoder::empty());
                println!("[Body and_then] {:?}", body);
                body.concat2()
            })
            .map_err(|err| println!("request error: {}", err))
            .map(|body| {
                println!("[Body] {:?}", body);
                let mut body = Cursor::new(body);
                let _ = io::copy(&mut body, &mut io::stdout())
                    .map_err(|err| {
                        println!("stdout error: {}", err);
                    });
            })
    }

    pub fn start(&self, url: &str) {
        println!("scraping {} is scheduled", url);
        self.schedule(TokioRuntimeWrapper::fetch(url));
    }

    fn schedule<F>(&self, future: F)
        where F: Future<Item=(), Error=()> + Send + 'static {
        self.rt.borrow_mut().spawn(future);
    }
}

fn main() {
    let matches = App::new("Tokio Runtime Wrapper")
        .version("1.0")
        .author("woods <woodsmur@gmail.com>")
        .about("Tokio Runtime Wrapper for scraping")
        .arg(Arg::with_name("url")
            .takes_value(true)
            .short("u")
            .long("url")
            .required(true)
            .help("url"))
        .get_matches();

    let url = matches.value_of("url").unwrap_or("https://youtube.com/");
    let tokio_wrapper = TokioRuntimeWrapper::new();
    tokio_wrapper.start(url);

    thread::sleep(Duration::from_secs(5));
}
