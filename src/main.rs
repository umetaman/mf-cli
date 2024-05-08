use std::collections::HashMap;
use std::io::Read;
use dialoguer::{self, Confirm};
use serde::Serialize;
use serde_derive::{Serialize, Deserialize};
use reqwest;
use clap::{arg, Command};

#[derive(Serialize, Deserialize)]
struct MoneyFoward {
    client_id: String,
    client_secret: String,
    scope: String,
    redirect_uri: String,
    approve_code: String,
    token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}

impl Default for MoneyFoward {
    fn default() -> Self {
        MoneyFoward {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            scope: "".to_string(),
            redirect_uri: "".to_string(),
            approve_code: "".to_string(),
            token: "".to_string(),
            refresh_token: "".to_string(),
        }
    }
}

fn cli() -> Command {
    return Command::new("mf-cli")
        .about("Commandline application for Moneyfoward API.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            // 初期設定
            Command::new("setup")
                .about("Setup tokens to use Moneyfoward API.")
        )
        .subcommand(
            Command::new("upload")
                .about("Upload a receipt to Moneyfoward.")
                .arg_required_else_help(true)
                .arg(arg!(-f --file <FILE> "The file to upload.")
                    .required(true))
                .arg(arg!(-p --process <PROCESS> "The process type to upload.")
                    .value_parser(["ocr", "operator"])
                    .required(true)
                    .default_value("ocr")
                    .default_missing_value("ocr"))
        )
        .subcommand(
            Command::new("watch")
                .about("Watch a directory and upload receipts to Moneyfoward.")   
                .arg_required_else_help(true)
                .arg(arg!(-d --directory <DIRECTORY> "The directory to watch.")
                    .required(true))
        )
        .subcommand(
            Command::new("token")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("refresh")
                        .about("Refresh the token.")
                )
                .subcommand(
                    Command::new("expiry")
                        .about("Check the token expiry.")
                )
        )
}

fn setup(config: &mut MoneyFoward) {
    if Confirm::new()
        .with_prompt(
            "You need to create an app in Moneyfoward. Do you want to open a browser and log in?",
        )
        .default(true)
        .interact()
        .unwrap()
    {
        open::that("https://moneyforward.com/apps").unwrap();
    }

    let client_id = dialoguer::Input::<String>::new()
        .with_prompt("Enter the client ID")
        .interact()
        .unwrap();
    config.client_id = client_id.clone();

    let client_secret = dialoguer::Input::<String>::new()
        .with_prompt("Enter the client secret")
        .interact()
        .unwrap();
    config.client_secret = client_secret.clone();

    let scope = dialoguer::Input::<String>::new()
        .with_prompt("Enter the scope")
        .interact()
        .unwrap();
    config.scope = scope.clone();

    let redirect_uri = dialoguer::Input::<String>::new()
        .with_prompt("Enter the redirect URI")
        .interact()
        .unwrap();
    config.redirect_uri = redirect_uri.clone();

    let url_approve = format!(
        "https://moneyforward.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
        client_id, redirect_uri, urlencoding::encode(&scope)
    );

    if Confirm::new()
        .with_prompt(&format!(
            "Please approve your app on MoneyFoward. Do you want to open browser? {}",
            url_approve
        ))
        .default(true)
        .interact()
        .unwrap()
    {
        open::that(&url_approve).unwrap();
    }

    let approve_code = dialoguer::Input::<String>::new()
        .with_prompt("Enter the approve code")
        .interact()
        .unwrap();
    config.approve_code = approve_code.clone();

    let url_token = "https://expense.moneyforward.com/oauth/token";

    let mut token_parameters = HashMap::new();
    token_parameters.insert("client_id", config.client_id.clone());
    token_parameters.insert("client_secret", config.client_secret.clone());
    token_parameters.insert("redirect_uri", config.redirect_uri.clone());
    token_parameters.insert("grant_type", "authorization_code".to_string());
    token_parameters.insert("code", config.approve_code.clone());

    // let client = reqwest::blocking::Client::new();
    // let _res = client
    //     .post(url_token)
    //     .form(&token_parameters)
    //     .send();

    // _res.
    let response: TokenResponse = reqwest::blocking::Client::new()
        .post(url_token)
        .form(&token_parameters)
        .send().unwrap()
        .json().unwrap();
    config.token = response.access_token.clone();
    config.refresh_token = response.refresh_token.clone();
}

const APP_NAME: &str = "mf-cli";
const CONFIG_NAME: &str = "config";

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("setup", _)) => {
            println!("Setup Command");
        }
        Some(("upload", submatches)) => {
            let file = submatches.get_one::<String>("file").unwrap();
            let process = submatches.get_one::<String>("process").unwrap();
            println!("Upload Command: file={}, process={}", file, process);
        }
        Some(("watch", submatches)) => {
            let directory = submatches.get_one::<String>("directory").unwrap();
            println!("Watch Command: directory={}", directory);
        }
        Some(("token", submatches)) => {
            match submatches.subcommand() {
                Some(("refresh", _)) => {
                    println!("Token Refresh Command");
                }
                Some(("expiry", _)) => {
                    println!("Token Expiry Command");
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    }

    // let mut config = confy::load(APP_NAME, CONFIG_NAME).unwrap();
    // let file_path = confy::get_configuration_file_path(APP_NAME, CONFIG_NAME).unwrap();

    // println!("The configuration file path is: {:#?}", file_path);

    // setup(&mut config);

    // confy::store(APP_NAME, None, &config).unwrap();
}
