use serde::Deserialize;

use crate::{errors::Error, graphql::graphql_query};

use super::Query;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EscrowAccount {
    pub sender: String,
    pub receiver: String,
    pub balance: u64,
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

    let query = r#"query accounts($sender: ID!) { escrowAccounts(where: {sender: $sender}) { sender receiver balance } }"#;
    let result = graphql_query::<EscrowAccountsData>(
        graphql_client,
        escrow_subgraph,
        Query::new_with_variables(query, [("sender", sender.into())]),
    )
    .await?;

    Ok(result?.escrow_accounts)
}

// Query escrow accounts related to the sender and receiver from the escrow subgraph
pub async fn escrow_account(
    graphql_client: &reqwest::Client,
    escrow_subgraph: &str,
    sender: &str,
    receiver: &str,
) -> Result<Option<EscrowAccount>, Error> {
    // Query  escrow accounts
    //     {
    //   escrowAccounts(where:{sender:"0xe9a1cabd57700b17945fd81feefba82340d9568f", receiver:"0xe9a1cabd57700b17945fd81feefba82340d9568f"}){
    //     balance
    //   }
    // }

    let query = r#"query accounts($sender: ID!, $receiver: ID!) { escrowAccounts(where: {sender: $sender, receiver: $receiver}) { sender receiver balance } }"#;
    let result = graphql_query::<Option<EscrowAccount>>(
        graphql_client,
        escrow_subgraph,
        Query::new_with_variables(
            query,
            [("sender", sender.into()), ("receiver", receiver.into())],
        ),
    )
    .await?;
    result.map_err(Error::GraphQLResponseError)
}
