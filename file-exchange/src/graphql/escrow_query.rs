use graphql_http::http::request::IntoRequestParameters;

use serde::Deserialize;

use crate::{errors::Error, graphql::graphql_query};

use super::Query;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EscrowAccount {
    // pub sender: ID,
    // pub receiver: ID,
    pub balance: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EscrowStatus {
    escrow_accounts: Vec<EscrowAccount>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ID {
    pub id: String,
}

// Query escrow accounts related to the sender from the escrow subgraph
pub async fn escrow_accounts(
    graphql_client: &reqwest::Client,
    escrow_subgraph: &str,
    sender: &str,
) -> Result<Vec<EscrowAccount>, anyhow::Error> {
    // Types for deserializing the network subgraph response
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct EscrowAccountsData {
        escrow_accounts: Vec<EscrowAccount>,
    }

    let query = r#"query accounts($sender: ID!) { escrowAccounts(where: {sender: $sender}) { sender { id } receiver { id } balance } }"#;
    let result = graphql_query::<EscrowAccountsData>(
        graphql_client,
        escrow_subgraph,
        Query::new_with_variables(query, [("sender", sender.into())]),
    )
    .await?;

    Ok(result?.escrow_accounts)
}

// Query escrow accounts related to the sender and receiver from the escrow subgraph
pub async fn escrow_balance(
    graphql_client: &reqwest::Client,
    escrow_subgraph: &str,
    sender: &str,
    receiver: &str,
) -> Result<Option<f64>, Error> {
    // Query  escrow accounts
    //     {
    //   escrowAccounts(where:{sender:"0xe9a1cabd57700b17945fd81feefba82340d9568f",
    //                          receiver:"0xe9a1cabd57700b17945fd81feefba82340d9568f"}){
    //     balance
    //   }
    // }

    let query = r#"query account($sender: ID!, $receiver: ID!) { escrowAccounts(where: {sender: $sender, receiver: $receiver}) { balance } }"#;
    // let query = r#"query account($sender: ID!, $receiver: ID!) { escrowAccounts(where: {sender: $sender, receiver: $receiver}) { sender { id } receiver { id } balance } }"#;
    let q = Query::new_with_variables(
        query,
        [("sender", sender.into()), ("receiver", receiver.into())],
    );
    tracing::warn!(q = tracing::field::debug(&q.into_request_parameters()), "q");
    let result = graphql_query::<EscrowStatus>(
        // let result: Result<Vec<EscrowAccount>, _> = typed_query(
        graphql_client,
        escrow_subgraph,
        Query::new_with_variables(
            query,
            [("sender", sender.into()), ("receiver", receiver.into())],
        ),
    )
    .await?;
    result
        .map(|status| {
            if status.escrow_accounts.is_empty() {
                None
            } else {
                status
                    .escrow_accounts
                    .first()
                    .unwrap()
                    .balance
                    .parse::<f64>()
                    .ok()
            }
        })
        .map_err(Error::GraphQLResponseError)
}
