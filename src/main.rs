use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cli {
    #[structopt(about = "Set the Nordigen API token")]
    SetToken {},
    #[structopt(about = "Add a new Nordigen account")]
    AddAccount {
        #[structopt(short, long)]
        country: Option<String>,
    },
    #[structopt(about = "Fetch transactions from Nordigen and import then into Firefly-iii")]
    Import {
        #[structopt(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MyConfig {
    api_token: String,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            api_token: "".into(),
        }
    }
}

fn main() -> Result<(), ::std::io::Error> {
    let opt = Cli::from_args();
    let mut cfg: MyConfig = confy::load("nordigen2fireflyiii").expect("Failed to load config");

    match opt {
        Cli::SetToken {} => {save_key(&mut cfg);}
        Cli::AddAccount {country} => {}
        Cli::Import {dry_run} => {}
    }

    Ok(())
}

fn save_key(cfg: &mut MyConfig) {
    println!("Paste your Nordigen API token below:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read from stdin");

    cfg.api_token = input.trim().to_string();
    confy::store("nordigen2fireflyiii", cfg).expect("Failed to save config");
}
