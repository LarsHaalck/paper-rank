use structopt::StructOpt;

use prank::user::User;
use prank::DbConn;
use rocket::fairing::Fairing;
use std::io::Error;

#[derive(StructOpt, Debug)]
#[structopt(about = "prankctl: CLI tool to manage prank")]
enum PrankCtl {
    Users(UsersSubcommand),
    Items(ItemsSubcommand),
}

#[derive(StructOpt, Debug)]
enum UsersSubcommand {
    Approve(Options),
    Reject(Options),
    Show(Options),
    Delete {
        #[structopt(required = true)]
        ids: Vec<i32>,
    },
}

#[derive(StructOpt, Debug)]
enum ItemsSubcommand {
    Delete(Options),
    Show(Options),
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(long)]
    all: bool,
    #[structopt(conflicts_with = "all", required_unless = "all")]
    ids: Vec<i32>,
}

async fn handle_users_command(cmd: UsersSubcommand, conn: &DbConn) -> Result<(), Error> {
    use UsersSubcommand::*;
    return match cmd {
        Approve(o) => {
            let rows = User::set_approve(o.ids, true, conn).await?;
            println!("Approved {} users", rows);
            Ok(())
        }
        Reject(o) => {
            let rows = User::set_approve(o.ids, false, conn).await?;
            println!("Rejected {} users", rows);
            Ok(())
        }
        Delete { ids } => {
            let rows = User::delete(ids, conn).await?;
            println!("Deleted {} users", rows);
            Ok(())
        }
        Show(o) => {
            let users = User::show(o.ids, conn).await?;
            println!("Found {} users", users.len());
            users.iter().for_each(|u| println!("{:?}", u));
            Ok(())
        }
    };
}

async fn handle_items_command(_cmd: ItemsSubcommand, _conn: &DbConn) -> Result<(), Error> {
    Ok(())
}

async fn handle_command(args: PrankCtl, conn: &DbConn) -> Result<(), Error> {
    return match args {
        PrankCtl::Users(c) => handle_users_command(c, conn).await,
        PrankCtl::Items(c) => handle_items_command(c, conn).await,
    };
}

#[rocket::main]
async fn main() {
    let args = PrankCtl::from_args();
    let rocket = DbConn::fairing().on_ignite(rocket::build()).await.unwrap();
    let conn = DbConn::get_one(&rocket)
        .await
        .expect("Unable to establish db connection");
    handle_command(args, &conn).await.unwrap();
}
