use structopt::StructOpt;

use anyhow::{Error, Result};
use chrono::NaiveDate;
use prank::item::Item;
use prank::user::User;
use prank::DbConn;
use rocket::fairing::Fairing;

mod mail;

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
    Delete(IdsOnly),
    GeneratePassword { id: i32 },
}

#[derive(StructOpt, Debug)]
enum ItemsSubcommand {
    Show(Options),
    Delete(IdsOnly),
    DiscussOn { id: i32, date: NaiveDate },
    CancelDiscuss { id: i32 },
    Dump(ItemDumpCommand),
    Mail(MailCommand),
}

#[derive(StructOpt, Debug)]
struct ItemDumpCommand {
    #[structopt(long, conflicts_with = "markdown", required_unless = "markdown")]
    html: bool,
    #[structopt(long, conflicts_with = "html", required_unless = "html")]
    markdown: bool,
    id: i32,
}

#[derive(StructOpt, Debug)]
struct MailCommand {
    id: i32,
    #[structopt(short = "f", long)]
    from: String,
    #[structopt(short = "t", long)]
    to: String,
    #[structopt(short = "s", long)]
    subject: Option<String>,
    #[structopt(short = "c", long)]
    comment: Option<String>,
    #[structopt(short = "u", long)]
    username: String,
    #[structopt(short = "h", long)]
    server: String,
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

async fn handle_users_command(cmd: UsersSubcommand, conn: &DbConn) -> Result<()> {
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
            let users = User::from_ids(o.ids, conn).await?;
            println!("Found {} users", users.len());
            users.iter().for_each(|u| println!("{:?}", u));
            Ok(())
        }
        GeneratePassword { id } => {
            let pass = User::set_random_password(id, conn).await?;
            println!("Set random password {} for id {}", pass, id);
            Ok(())
        }
    };
}

fn format_item(item: &Item, html: bool) -> String {
    if html {
        format!("<h3>{}</h3>\n{}", item.title, item.html)
    } else {
        format!("{}\n-------\n{}", item.title, item.markdown)
    }
}

async fn handle_items_command(cmd: ItemsSubcommand, conn: &DbConn) -> Result<()> {
    use ItemsSubcommand::*;
    return match cmd {
        Show(o) => {
            let items = Item::from_ids(o.ids, conn).await?;
            println!("Found {} items", items.len());
            items.iter().for_each(|u| {
                println!(
                    "Item {{ id: {}, title: {}, markdown: <omitted>, discussed_on: {:?} }}",
                    u.id, u.title, u.discussed_on
                );
            });
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
        }
        CancelDiscuss { id } => {
            Item::set_discussed(id, None, conn).await?;
            println!("Updated item {}", id);
            Ok(())
        }
        Dump(o) => {
            let item = Item::from_id(o.id, conn)
                .await
                .ok_or(Error::msg("Item not found"))?;
            println!("Dump of item with id {}:", o.id);
            println!("##############################");
            println!("{}", format_item(&item, o.html));
            Ok(())
        }
        Mail(o) => {
            let item = Item::from_id(o.id, conn)
                .await
                .ok_or(Error::msg("Item not found"))?;
            mail::send(
                &item, o.from, o.to, o.subject, o.comment, o.username, o.server,
            )?;
            println!("Send mail was successful");
            Ok(())
        }
    };
}

async fn handle_command(args: PrankCtl, conn: &DbConn) -> Result<()> {
    return match args {
        PrankCtl::Users(c) => handle_users_command(c, conn).await,
        PrankCtl::Items(c) => handle_items_command(c, conn).await,
    };
}

#[rocket::main]
async fn main() {
    let args = PrankCtl::from_args();
    let rocket = DbConn::fairing()
        .on_ignite(rocket::build())
        .await
        .expect("Unable to establish db connection.");
    let conn = DbConn::get_one(&rocket)
        .await
        .expect("Unable to establish db connection.");
    if let Err(e) = handle_command(args, &conn).await {
        println!("Error: {}", e);
    }
}
