use graphql_client::{GraphQLQuery, Response};
use std::error::Error;
use reqwest;

pub const URL: &str = "https://devex-apollo.zilliqa.com/";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/schema.json",
    query_path = "src/graphql/queries/fights.graphql",
    response_derives = "Debug",
)]
pub struct UnionQuery;

pub async fn get_fights_stats(variables: union_query::Variables) -> Result<(), Box<dyn Error>> {
    // this is the important line
    let request_body = UnionQuery::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client.post(URL).json(&request_body).send().await?;
    let response_body: Response<union_query::ResponseData> = res.json().await?;
    println!("{:#?}", response_body);
    Ok(())
}
