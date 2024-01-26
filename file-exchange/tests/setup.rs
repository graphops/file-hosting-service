// use serde_json::json;
// use wiremock::{
//     matchers::{body_string_contains, method, path},
//     Mock, MockServer, ResponseTemplate,
// };

// use indexer_common::{prelude::SubgraphClient, subgraph_client::DeploymentDetails};

// mod test_vectors;

// pub async fn setup_mock_network_subgraph() -> MockServer {
//     // Set up a mock network subgraph
//     let mock_server = MockServer::start().await;
//     // Mock result for current epoch requests
//     mock_server
//         .register(
//             Mock::given(method("POST"))
//                 .and(path(format!("/network",)))
//                 .and(body_string_contains("currentEpoch"))
//                 .respond_with(ResponseTemplate::new(200).set_body_json(
//                     json!({ "data": { "graphNetwork": { "currentEpoch": 896419 }}}),
//                 )),
//         )
//         .await;

//     // Mock result for allocations query
//     mock_server
//         .register(
//             Mock::given(method("POST"))
//                 .and(path(format!("/network",)))
//                 .and(body_string_contains("activeAllocations"))
//                 .respond_with(
//                     ResponseTemplate::new(200)
//                         .set_body_raw(test_vectors::ALLOCATIONS_QUERY_RESPONSE, "application/json"),
//                 ),
//         )
//         .await;

//     mock_server
// }

// pub async fn setup_mock_escrow_subgraph() -> MockServer {
//     let mock_server = MockServer::start().await;
//     println!("mock server: {:#?}", mock_server.uri());
//     let mock = Mock::given(method("POST"))
//         .and(path(format!("/escrow",)))
//         .respond_with(
//             ResponseTemplate::new(200)
//                 .set_body_raw(test_vectors::ESCROW_QUERY_RESPONSE, "application/json"),
//         );
//     mock_server.register(mock).await;
//     mock_server
// }
