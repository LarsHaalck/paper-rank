use structopt::StructOpt;

use prank::user::User;
use prank::item::Item;
use prank::DbConn;
use rocket::fairing::Fairing;
use std::io::Error;
use chrono::NaiveDate;

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
    Delete(IdsOnly)
}

#[derive(StructOpt, Debug)]
enum ItemsSubcommand {
    Show(Options),
    Delete(IdsOnly),
    DiscussOn {
        id: i32,
        date: NaiveDate
    },
    CancelDiscuss {
        id: i32
    }
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(long)]
    all: bool,
    #[structopt(conflicts_with = "all", required_unless = "all")]
    ids: Vec<i32>,
}

#[derive(StructOpt, Debug)]
struct IdsOnly {
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
        Delete(o) => {
            let rows = User::delete(o.ids, conn).await?;
            println!("Deleted {} users", rows);
            Ok(())
        }
        Show(o) => {
            let users = User::get(o.ids, conn).await?;
            println!("Found {} users", users.len());
            users.iter().for_each(|u| println!("{:?}", u));
            Ok(())
        }
    };
}

async fn handle_items_command(cmd: ItemsSubcommand, conn: &DbConn) -> Result<(), Error> {
    use ItemsSubcommand::*;
    return match cmd {
        Show(o) => {
            let items = Item::get(o.ids, conn).await?;
            println!("Found {} items", items.len());
            items.iter().for_each(|u| println!("{:?}", u));
            Ok(())
        }
        Delete(o) => {
            let rows = Item::delete(o.ids, conn).await?;
            println!("Deleted {} items", rows);
            Ok(())
        }
        DiscussOn { id, date } => {
            Item::set_discussed(id, Some(date), conn).await?;
            println!("Updated item {}", id);
            Ok(())
        },
        CancelDiscuss { id } => {
            Item::set_discussed(id, None, conn).await?;
            println!("Updated item {}", id);
            Ok(())
        }
    }
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
