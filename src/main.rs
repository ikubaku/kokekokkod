use tokio::time;

use elefren::prelude::*;
use elefren::scopes::Write;
use elefren::helpers::toml;
use elefren::helpers::cli;

use isolang::Language;

mod obniz;
use obniz::ObnizResponse;

mod context;
use context::{Context, StatusKind, EventKind, Event};
use crate::obniz::Status;

async fn get_data_from_obniz() -> Result<ObnizResponse, Box<dyn std::error::Error>> {
    let res = reqwest::get("http://localhost:3000/")
        .await?
        .json::<ObnizResponse>()
        .await?;
    println!("{:?}", res);
    Ok(res)
}

fn update_and_check_status(ctx: &mut Context, res: ObnizResponse) -> Option<Event> {
    ctx.update_status(
        res.get_datetime(),
        if res.is_heavier_than(-7500000.0) {
            StatusKind::Sleeping
        } else {
            StatusKind::Awake
        }
    );
    ctx.read_change()
}

fn register() -> Result<Mastodon, Box<dyn std::error::Error>> {
    println!("Enter the URL of the your Mastodon server (please begin with `http://`)");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let url = buffer.strip_suffix("\r\n")
        .or(buffer.strip_suffix("\n"))
        .unwrap_or(&buffer);
    let registration = Registration::new(url)
        .client_name("kokekokko")
        .scopes(Scopes::write(Write::Statuses))
        .build()?;
    let mastodon = cli::authenticate(registration)?;

    toml::to_file(&*mastodon, "mastodon-data.toml")?;

    Ok(mastodon)
}

fn post_status(mastodon: &Mastodon, event: Event) -> Result<(), Box<dyn std::error::Error>> {
    match event.kind {
        EventKind::WakeUp => mastodon.new_status(
                StatusBuilder::new()
                    .status("おはよー")
                    .language(Language::Jpn)
                    .build()?
        )?,
        EventKind::StartSleeping => mastodon.new_status(
            StatusBuilder::new()
                .status("おやすみー")
                .language(Language::Jpn)
                .build()?
        )?
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(time::Duration::from_secs(90));
    let mastodon = if let Ok(data) = toml::from_file("mastodon-data.toml") {
        Mastodon::from(data)
    } else {
        register()?
    };
    let mut ctx = Context::default();

    loop {
        interval.tick().await;
        let res = get_data_from_obniz().await?;
        match res.status() {
            Status::Ok => {
                if let Some(event) = update_and_check_status(&mut ctx, res) {
                    post_status(&mastodon, event)?;
                }
            },
            Status::Ng => println!("Error when communicating to the obniz application"),
            Status::NoData => println!("No data is available. Skipping.")
        }
    }
}
