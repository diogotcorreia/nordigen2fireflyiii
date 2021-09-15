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
    let cfg: MyConfig = confy::load("nordigen2fireflyiii").expect("Failed to load config");
    println!("{:?}", opt);
    println!("{:?}", cfg);

    save_key(cfg);

    Ok(())
}

fn save_key(cfg: MyConfig) {
    println!("Paste your Nordigen API token below:");
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(n) => {
            println!("{} bytes read", n);
            println!("{}", input);
        }
        Err(error) => println!("error: {}", error),
    }
}
