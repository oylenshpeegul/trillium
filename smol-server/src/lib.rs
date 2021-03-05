use async_net::{TcpListener, TcpStream};
use myco::{async_trait, Handler};
use myco_server_common::{Acceptor, Stopper};
use smol::prelude::*;
use std::sync::Arc;

pub use myco_server_common::Server;
pub type Config<A> = myco_server_common::Config<SmolServer, A, TcpStream>;

use signal_hook::consts::signal::*;
use signal_hook_async_std::Signals;

async fn handle_signals(stop: Stopper) {
    let signals = Signals::new(&[SIGINT, SIGTERM, SIGQUIT]).unwrap();
    let mut signals = signals.fuse();
    while signals.next().await.is_some() {
        if stop.is_stopped() {
            println!("second interrupt, shutting down harshly");
            std::process::exit(1);
        } else {
            println!("shutting down gracefully");
            stop.stop();
        }
    }
}

pub struct SmolServer;

#[async_trait]
impl Server for SmolServer {
    type Transport = TcpStream;

    fn run<A: Acceptor<Self::Transport>, H: Handler>(config: Config<A>, handler: H) {
        smol::block_on(async move { Self::run_async(config, handler).await })
    }

    async fn run_async<A: Acceptor<Self::Transport>, H: Handler>(
        config: Config<A>,
        mut handler: H,
    ) {
        smol::spawn(handle_signals(config.stopper())).detach();
        let socket_addrs = config.socket_addrs();
        let listener = TcpListener::bind(&socket_addrs[..]).await.unwrap();
        log::info!("listening on {:?}", listener.local_addr().unwrap());
        let mut incoming = config.stopper().stop_stream(listener.incoming());
        handler.init().await;
        let handler = Arc::new(handler);

        while let Some(Ok(stream)) = incoming.next().await {
            myco::log_error!(stream.set_nodelay(config.nodelay()));
            smol::spawn(config.clone().handle_stream(stream, handler.clone())).detach();
        }

        config.graceful_shutdown().await;
    }
}

pub fn run(handler: impl Handler) {
    config().run(handler)
}

pub async fn run_async(handler: impl Handler) {
    config().run_async(handler).await
}

pub fn config() -> Config<()> {
    Config::new()
}
