use structopt::StructOpt;

use prank::user::User;
use prank::DbConn;
use futures::executor::block_on;
use std::io::Error;

#[derive(StructOpt, Debug)]
#[structopt(about = "prankctl: CLI tool to manage prank")]
enum PrankCtl {
    Users(UsersSubcommand),
    Items(ItemsSubcommand)
}

#[derive(StructOpt, Debug)]
enum UsersSubcommand {
    Approve(Options),
    Reject(Options),
    Show(Options)
}

#[derive(StructOpt, Debug)]
enum ItemsSubcommand {
    Delete(Options),
    Show(Options)
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(long)]
    all: bool,
    #[structopt(conflicts_with = "all", required_unless = "all")]
    ids: Vec<i32>
}

async fn handle_users_command(cmd: UsersSubcommand, conn: &DbConn) -> Result<(), Error> {
    use UsersSubcommand::*;
    return match cmd {
        Approve(o) => Ok(()),
        Reject(o) => Ok(()),
        Show(o) => {
            let users = User::show(o.ids, conn).await;
            println!("{:?}", users);
            Ok(())
        }
    }
}

async fn handle_items_command(cmd: ItemsSubcommand, conn: &DbConn) -> Result<(), Error> {
    Ok(())
}

async fn handle_command(args: PrankCtl) -> Result<(), Error> {
    // let fairing = DbConn::fairing();
    // let rocket = rocket::build()
    //     .attach(fairing);
    // fairing.on_ignite(&rocket).await;
    // let conn = DbConn::get_one(&rocket).await.expect("Unable to establish db connection");
    // return match args {
    //     PrankCtl::Users(c) => handle_users_command(c, &conn).await,
    //     PrankCtl::Items(c) => handle_items_command(c, &conn).await
    // }
    
}

fn main() {
    let args = PrankCtl::from_args();
    block_on(handle_command(args));
}
