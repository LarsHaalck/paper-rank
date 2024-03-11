use structopt::StructOpt;

use anyhow::{Context, Error, Result};
use chrono::NaiveDate;
use prank::item::{Item, ItemFormat};
use prank::user::User;
use prank::DbConn;
use rocket::fairing::Fairing;

use prank::mail;

#[derive(StructOpt, Debug)]
#[structopt(about = "prankctl: CLI tool to manage prank")]
enum PrankCtl {
    Users(UsersSubcommand),
    Items(ItemsSubcommand),
}

#[derive(StructOpt, Debug)]
enum UsersSubcommand {
    Admin(IdOptions),
    RemoveAdmin(IdOptions),
    Approve(IdOptions),
    Reject(IdOptions),
    List(IdOptions),
    Delete(IdsOnly),
    GeneratePassword { id: i32 },
}

#[derive(StructOpt, Debug)]
enum ItemsSubcommand {
    List {
        #[structopt(flatten)]
        id_opt: IdOptions,
        #[structopt(flatten)]
        date_opt: ItemDateOption,
    },
    Delete(IdsOnly),
    DiscussOn {
        id: i32,
        date: NaiveDate,
    },
    CancelDiscuss {
        id: i32,
    },
    Dump(ItemDumpCommand),
    Mail(MailCommand),
}

#[derive(StructOpt, Debug)]
struct ItemDateOption {
    #[structopt(long, conflicts_with_all = &["discussed"])]
    undiscussed: bool,
    #[structopt(long, conflicts_with_all = &["undiscussed"])]
    discussed: bool,
}

#[derive(StructOpt, Debug)]
struct ItemDumpCommand {
    #[structopt(long)]
    html: bool,
    id: i32,
}

#[derive(StructOpt, Debug)]
struct MailCommand {
    id: i32,
    #[structopt(short = "f", long)]
    from: String,
    #[structopt(short = "t", long)]
    to: String,
    #[structopt(short = "c", long)]
    comment: Option<String>,
    #[structopt(short = "u", long)]
    username: String,
    #[structopt(short = "h", long)]
    server: String,
}

#[allow(dead_code)]
#[derive(StructOpt, Debug)]
struct IdOptions {
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
        List(o) => {
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

async fn handle_items_command(cmd: ItemsSubcommand, conn: &DbConn) -> Result<()> {
    use ItemsSubcommand::*;
    return match cmd {
        List { id_opt, date_opt } => {
            let items =
                Item::from_ids(id_opt.ids, date_opt.discussed, date_opt.undiscussed, conn).await?;
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
            println!(
                "{}",
                item.format(if o.html {
                    ItemFormat::HTML
                } else {
                    ItemFormat::Markdown
                })
            );
            Ok(())
        }
        Mail(o) => {
            let item = Item::from_id(o.id, conn)
                .await
                .ok_or(Error::msg("Item not found"))?;
            let password = rpassword::prompt_password_stdout("Password: ")
                .context("Error getting password")?;
            mail::send(
                &item, o.from, o.to, o.comment, o.username, o.server, password,
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
