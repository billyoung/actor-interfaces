[![crates.io](https://img.shields.io/crates/v/wasmcloud-actor-graphdb.svg)](https://crates.io/crates/wasmcloud-actor-graphdb)&nbsp;
![Rust](https://img.shields.io/github/workflow/status/wasmcloud/actor-interfaces/GraphDB)
![license](https://img.shields.io/crates/l/wasmcloud-actor-graphdb.svg)&nbsp;
[![documentation](https://docs.rs/wasmcloud-actor-graphdb/badge.svg)](https://docs.rs/wasmcloud-actor-graphdb)
# wasmCloud GraphDB Actor Interface

This crate provides an abstraction over the `wasmcloud:graphdb` contract. This
allows actors to interact with a graph database, such as RedisGraph or Neo4j.

Example:

```rust
use serde_json::json;
extern crate wapc_guest as guest;
use wasmcloud_actor_graphdb as graph;
use graph::*;
use wasmcloud_actor_http_server as http;

use guest::prelude::*;

#[no_mangle]
pub fn wapc_init() {
    http::Handlers::register_handle_request(handle_http_request);
    wasmcloud_actor_core::Handlers::register_health_request(health);
}

fn handle_http_request(_: http::Request) -> HandlerResult<http::Response> {
    // Replace `query_db` with `graph::default().query_graph()`
    let (name, birth_year): (String, u32) = query_db(
        "MotoGP".to_string(),
        "MATCH (r:Rider)-[:rides]->(t:Team) WHERE t.name = 'Yamaha' RETURN r.name, r.birth_year"
            .to_string(),
    )?;

    assert_eq!(name, "Alice Rider".to_string());
    assert_eq!(birth_year, 1985);

    let result = json!({
        "name": name,
        "birth_year": birth_year
    });

    Ok(http::Response::json(result, 200, "OK"))
}

fn health(_: wasmcloud_actor_core::HealthCheckRequest) -> HandlerResult<actor_core::HealthCheckResponse> {
  Ok(actor_core::HealthCheckResponse::healthy())   
}
```

