use clap::{Parser, Subcommand};
use reqwest::Response;
use serde::Serialize;

const QUERIES_QUERY: &'static str = "\
{
  __schema {
    queryType {
      fields {
        name
      }
    }
  }
}
";

const MUTATIONS_QUERY: &'static str = "\
{
  __schema {
    mutationType {
      fields {
        name
      }
    }
  }
}
";

const LOGIN_QUERY: &'static str = "\
mutation Login($username: String!, $password: String!, $remember: Boolean!) {
      login(username: $username, password: $password, remember: $remember) {
        accessToken
      }
    }
";

#[derive(Serialize)]
struct Auth<'a> {
    username: &'a str,
    password: &'a str,
    remember: bool
}

#[derive(Serialize)]
struct GraphQLQuery<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Auth<'a>>,
}

struct GraphQLClient<'a> {
    client: reqwest::Client,
    endpoint: &'a str,
}

impl<'a> GraphQLClient<'a> {
    pub fn new(endpoint: &'a str) -> Self {
	let client = reqwest::Client::new();

	Self { client, endpoint }
    }
    
    async fn fetch_graphql(&self, query: &GraphQLQuery<'_>)
			   -> Result<Response, Box<dyn std::error::Error>>{
	let payload = serde_json::to_string(query)?;

	Ok(self.client.post(self.endpoint)
	    .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await?)
    }
}

#[derive(Subcommand)]
enum Query {
    /// Login to the bookmarks server
    Login {
	/// Username for login query
	#[clap(short, long)]
	username: String,

	/// Password for login query
	#[clap(short, long)]
	password: String,
    },
    /// Retrieve a list of queries to the API from the server
    QueryType {},
    /// Retrieve a list of API mutations from the server
    MutationType {},
}

///Bookmarks client
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Query type
    #[clap(subcommand)]
    query: Query,

    /// Endpoint to connect to
    endpoint: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let endpoint = args.endpoint;
    let graphql_client = GraphQLClient::new(&endpoint);
    let query = match &args.query {
	Query::Login {
	    username,
	    password,
	} => GraphQLQuery {
	    query: LOGIN_QUERY,
	    variables: Some(Auth {
		username: &username,
		password: &password,
		remember: false,
	    }),
	},
	Query::QueryType {} => GraphQLQuery {
	    query: QUERIES_QUERY,
	    variables: None,
	},
	Query::MutationType {} => GraphQLQuery {
	    query: MUTATIONS_QUERY,
	    variables: None,
	},
    };
    let res = graphql_client.fetch_graphql(&query).await?.text().await?;
    
    println!("{}", res);

    Ok(())
}
