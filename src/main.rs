use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cli {
    #[structopt(about = "Set the Nordigen API token")]
    SetToken {},
    #[structopt(about = "Add a new Nordigen account")]
    AddAccount {
        #[structopt(short, long)]
        country: String,
    },
    #[structopt(about = "Fetch transactions from Nordigen and import then into Firefly-iii")]
    Import {
        #[structopt(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MyConfig {
    api_token: Option<String>,
    client_uuid: Option<String>,
    accounts: Option<Vec<BankAccount>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BankAccount {
    name: String,
    uuid: String,
}

#[derive(Debug, Deserialize)]
struct Bank {
    id: String,
    name: String,
    bic: String,
}

#[derive(Debug, Deserialize)]
struct NordigenRequisition {
    id: String,
    accounts: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize)]
struct NordigenRequisitionPayload {
    enduser_id: String,
    reference: String,
    redirect: String,
    agreements: Vec<String>,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            api_token: None,
            client_uuid: None,
            accounts: None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), ::std::io::Error> {
    let opt = Cli::from_args();
    let mut cfg: MyConfig = confy::load("nordigen2fireflyiii").expect("Failed to load config");

    match opt {
        Cli::SetToken {} => {
            save_key(&mut cfg);
        }
        Cli::AddAccount { country } => {
            add_account(country, &mut cfg).await;
        }
        Cli::Import { dry_run } => {}
    }

    Ok(())
}

fn save_key(cfg: &mut MyConfig) {
    println!("Paste your Nordigen API token below:");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read from stdin");

    cfg.api_token = Some(input.trim().to_string());
    confy::store("nordigen2fireflyiii", cfg).expect("Failed to save config");
}

async fn add_account(country: String, cfg: &mut MyConfig) {
    let api_token = cfg
        .api_token
        .clone()
        .expect("Please set the API token using the set-token command");

    println!("Fetching available banks...");

    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://ob.nordigen.com/api/aspsps/?country={}",
            country
        ))
        .header("Accept", "application/json")
        .header("Authorization", format!("Token {}", api_token))
        .send()
        .await
        .expect("Failed to fetch banks")
        .json::<Vec<Bank>>()
        .await
        .expect("Failed to fetch banks");

    for (i, x) in res.iter().enumerate() {
        println!("{:>2} - {}", i + 1, x.name);
    }

    println!();
    println!("Type the ID of the bank:");

    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read from stdin");
    let bank_id: usize = buffer.trim().parse().expect("Invalid bank ID");

    if bank_id <= 0 || bank_id > res.len() {
        panic!("Invalid bank ID");
    }

    println!(
        "Selected bank '{}' ({})",
        res[bank_id - 1].name,
        res[bank_id - 1].id
    );

    let user_uuid = match &cfg.client_uuid {
        None => {
            println!("This is the first account being added, creating requisition...");
            let uuid = uuid::Uuid::new_v4().to_hyphenated().to_string();

            let payload = NordigenRequisitionPayload {
                enduser_id: uuid,
                reference: String::from("nordigen2fireflyiii"),
                redirect: String::from(
                    "https://github.com/diogotcorreia/nordigen2fireflyiii/wiki/Account-Added",
                ),
                agreements: Vec::new(),
            };

            let requisition = client
                .post("https://ob.nordigen.com/api/requisitions/")
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Token {}", api_token))
                .json(&payload)
                .send()
                .await
                .expect("Failed to create requisition")
                .json::<NordigenRequisition>()
                .await
                .expect("Failed to create requisitions");

            cfg.client_uuid = Some(String::clone(&requisition.id));
            confy::store("nordigen2fireflyiii", cfg).expect("Failed to save config");

            println!("Created requisition!");

            String::from(requisition.id)
        }
        Some(uuid) => String::from(uuid),
    };

    println!("{}", user_uuid);

    let mut link_request_payload = std::collections::HashMap::new();
    link_request_payload.insert("aspsp_id", res[bank_id - 1].id.clone());

    let link_response = client
        .post(format!(
            "https://ob.nordigen.com/api/requisitions/{}/links/",
            user_uuid
        ))
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Token {}", api_token))
        .json(&link_request_payload)
        .send()
        .await
        .expect("Failed to get PSD2 authorization link")
        .json::<std::collections::HashMap<String, String>>()
        .await
        .expect("Failed to get PSD2 authorization link");

    let link = link_response
        .get("initiate")
        .expect("Failed to get PSD2 authorization link");

    println!("Almost there, you just need to complete the authorization step in your browser:");
    println!("{}", link);
}
