use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;


pub struct LB {
    lb:   Arc<LoadBalancer<RoundRobin>>,
    host: String
}


#[async_trait]
impl ProxyHttp for LB {

    // We don't need context storage for now
    type CTX = ();
    fn new_ctx(&self) -> () {()}

    async fn upstream_peer(
        &self, _session: &mut Session, _ctx: &mut ()
    ) -> Result<Box<HttpPeer>> {
        let upstream = self.lb
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        println!("upstream peer is: {upstream:?}");

        // Set SNI to one.one.one.one
        let peer = Box::new(HttpPeer::new(upstream, true, self.host.clone()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self, _session: &mut Session, upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("Host", self.host.clone()).unwrap();
        Ok(())
    }
}


fn main() {
    let remote_host: String = "pswww.slac.stanford.edu".to_string();
    let upstream_addr = format!("{}:{}", remote_host, 443);
    println!("Redirecting to: {}", upstream_addr);

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let upstreams =
        LoadBalancer::try_from_iter([upstream_addr,]).unwrap();

    let mut lb = http_proxy_service(
        &my_server.configuration, LB{
            lb: Arc::new(upstreams), host: remote_host
        }
    );
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(lb);

    my_server.run_forever();
}
