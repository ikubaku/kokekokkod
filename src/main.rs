use tokio::time;

mod obniz;
use obniz::ObnizResponse;

mod context;
use context::{Context, StatusKind, EventKind};
use crate::obniz::Status;

async fn get_data_from_obniz() -> Result<ObnizResponse, Box<dyn std::error::Error>> {
    let res = reqwest::get("http://localhost:3000/")
        .await?
        .json::<ObnizResponse>()
        .await?;
    println!("{:?}", res);
    Ok(res)
}

fn update_and_check_status(ctx: &mut Context, res: ObnizResponse) {
    ctx.update_status(
        res.get_datetime(),
        if res.is_heavier_than(-7500000.0) {
            StatusKind::Sleeping
        } else {
            StatusKind::Awake
        }
    );
    if let Some(e) = ctx.read_change() {
        match e.kind {
            EventKind::WakeUp => println!("WakeUp: {}", e.datetime),
            EventKind::StartSleeping => println!("Sleep: {}", e.datetime),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(time::Duration::from_secs(90));
    let mut ctx = Context::default();

    loop {
        interval.tick().await;
        let res = get_data_from_obniz().await?;
        match res.status() {
            Status::Ok => update_and_check_status(&mut ctx, res),
            Status::Ng => println!("Error when communicating to the obniz application"),
            Status::NoData => println!("No data is available. Skipping.")
        }
    }
}
