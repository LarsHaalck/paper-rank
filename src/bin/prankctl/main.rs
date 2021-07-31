use structopt::StructOpt;

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
    ids: Vec<u32>
}

fn main() {
    let opt = PrankCtl::from_args();
    println!("{:#?}", opt);
}
