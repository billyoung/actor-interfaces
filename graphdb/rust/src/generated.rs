extern crate rmp_serde as rmps;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

extern crate log;
#[cfg(feature = "guest")]
extern crate wapc_guest as guest;
#[cfg(feature = "guest")]
use guest::prelude::*;

#[cfg(feature = "guest")]
use lazy_static::lazy_static;
#[cfg(feature = "guest")]
use std::sync::RwLock;

#[cfg(feature = "guest")]
pub struct Host {
    binding: String,
}

#[cfg(feature = "guest")]
impl Default for Host {
    fn default() -> Self {
        Host {
            binding: "default".to_string(),
        }
    }
}

/// Creates a named host binding
#[cfg(feature = "guest")]
pub fn host(binding: &str) -> Host {
    Host {
        binding: binding.to_string(),
    }
}

/// Creates the default host binding
#[cfg(feature = "guest")]
pub fn default() -> Host {
    Host::default()
}

#[cfg(feature = "guest")]
impl Host {
    pub(crate) fn query(&self, graphName: String, query: String) -> HandlerResult<QueryResponse> {
        let input_args = QueryGraphArgs {
            graph_name: graphName,
            query: query,
        };
        host_call(
            &self.binding,
            "wasmcloud:graphdb",
            "QueryGraph",
            &serialize(input_args)?,
        )
        .map(|vec| {
            let resp = deserialize::<QueryResponse>(vec.as_ref()).unwrap();
            resp
        })
        .map_err(|e| e.into())
    }

    pub fn delete_graph(&self, graphName: String) -> HandlerResult<DeleteResponse> {
        let input_args = DeleteGraphArgs {
            graph_name: graphName,
        };
        host_call(
            &self.binding,
            "wasmcloud:graphdb",
            "DeleteGraph",
            &serialize(input_args)?,
        )
        .map(|vec| {
            let resp = deserialize::<DeleteResponse>(vec.as_ref()).unwrap();
            resp
        })
        .map_err(|e| e.into())
    }
}

#[cfg(feature = "guest")]
pub struct Handlers {}

#[cfg(feature = "guest")]
impl Handlers {
    pub fn register_query_graph(f: fn(String, String) -> HandlerResult<QueryResponse>) {
        *QUERY_GRAPH.write().unwrap() = Some(f);
        register_function(&"QueryGraph", query_graph_wrapper);
    }
    pub fn register_delete_graph(f: fn(String) -> HandlerResult<DeleteResponse>) {
        *DELETE_GRAPH.write().unwrap() = Some(f);
        register_function(&"DeleteGraph", delete_graph_wrapper);
    }
}

#[cfg(feature = "guest")]
lazy_static! {
    static ref QUERY_GRAPH: RwLock<Option<fn(String, String) -> HandlerResult<QueryResponse>>> =
        RwLock::new(None);
    static ref DELETE_GRAPH: RwLock<Option<fn(String) -> HandlerResult<DeleteResponse>>> =
        RwLock::new(None);
}

#[cfg(feature = "guest")]
fn query_graph_wrapper(input_payload: &[u8]) -> CallResult {
    let input = deserialize::<QueryGraphArgs>(input_payload)?;
    let lock = QUERY_GRAPH.read().unwrap().unwrap();
    let result = lock(input.graph_name, input.query)?;
    Ok(serialize(result)?)
}

#[cfg(feature = "guest")]
fn delete_graph_wrapper(input_payload: &[u8]) -> CallResult {
    let input = deserialize::<DeleteGraphArgs>(input_payload)?;
    let lock = DELETE_GRAPH.read().unwrap().unwrap();
    let result = lock(input.graph_name)?;
    Ok(serialize(result)?)
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct QueryGraphArgs {
    #[serde(rename = "graphName")]
    pub graph_name: String,
    #[serde(rename = "query")]
    pub query: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct DeleteGraphArgs {
    #[serde(rename = "graphName")]
    pub graph_name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct QueryResponse {
    #[serde(rename = "resultSet")]
    pub result_set: ResultSet,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct DeleteResponse {
    #[serde(rename = "success")]
    pub success: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct ResultSet {
    #[serde(rename = "columns")]
    pub columns: Vec<Column>,
    #[serde(rename = "statistics")]
    pub statistics: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Column {
    #[serde(rename = "scalars")]
    pub scalars: Option<Vec<Scalar>>,
    #[serde(rename = "nodes")]
    pub nodes: Option<Vec<Node>>,
    #[serde(rename = "relations")]
    pub relations: Option<Vec<Relation>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Scalar {
    #[serde(rename = "boolValue")]
    pub bool_value: Option<bool>,
    #[serde(rename = "intValue")]
    pub int_value: Option<i64>,
    #[serde(rename = "doubleValue")]
    pub double_value: Option<f64>,
    #[serde(rename = "stringValue")]
    pub string_value: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Node {
    #[serde(rename = "labels")]
    pub labels: Vec<String>,
    #[serde(rename = "properties")]
    pub properties: std::collections::HashMap<String, Scalar>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Relation {
    #[serde(rename = "typeName")]
    pub type_name: String,
    #[serde(rename = "properties")]
    pub properties: std::collections::HashMap<String, Scalar>,
}

/// The standard function for serializing codec structs into a format that can be
/// used for message exchange between actor and host. Use of any other function to
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(
    item: T,
) -> ::std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    item.serialize(&mut Serializer::new(&mut buf).with_struct_map())?;
    Ok(buf)
}

/// The standard function for de-serializing codec structs from a format suitable
/// for message exchange between actor and host. Use of any other function to
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<'de, T: Deserialize<'de>>(
    buf: &[u8],
) -> ::std::result::Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let mut de = Deserializer::new(Cursor::new(buf));
    match Deserialize::deserialize(&mut de) {
        Ok(t) => Ok(t),
        Err(e) => Err(format!("Failed to de-serialize: {}", e).into()),
    }
}
