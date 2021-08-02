## Paper-Rank (prank)

Paper-Rank is a simple tool written in Rust to organize topics of recurrent meetings with
ongoing elections. It is designed to decide on papers for a recurring meeting
discussing current research topics but is not specific to this particular use
case and should be easy to adopt to other applications.

This tool started as a fork of
[jonhoo/vote.rs](https://github.com/jonhoo/vote.rs) but is now standalone due to many changes including:
* Update to [Rocket](https://rocket.rs/) 0.5 with full async functionality
* User login with passwords to avoid spam using [bcrypt-pbkdf](https://crates.io/crates/bcrypt-pbkdf), including password change feature and register feature for new users
* Markdown editor using the [Comrak](https://github.com/kivikakk/comrak) crate for [CommonMark](https://commonmark.org)-Support to let users add items (images and raw html are filtered)
* History site, where old entries are archived
* Pin upcoming topics to make them unvotable but still show them to all users
* Dockerfile and docker-compose for easy deployment
* small adming page to edit items and change the dates

As the original version [jonhoo/vote.rs](https://github.com/jonhoo/vote.rs),
this tool uses 
[ranked choice voting](https://ballotpedia.org/Ranked-choice_voting_(RCV)) using [LivingInSyn/rcir](https://github.com/LivingInSyn/rcira).

The basic idea is that users rank the available items according to their preference
and the final election is run before each meeting to determine the topic.
Since ranked choice voting lets users specify multiple preferences, this process can
then be repeated for the next meeting, where it will go to each user's second
preferred candidate, etc.

## Deploying
To deploy, the sqlite db has to be initialized using:

```console
$ sqlite3 db/db.sqlite < schema.sql
$ cargo run --release
```

Change `Rocket.toml` according to your needs.
It contains a default value for `secret_key` which should be changed using `openssl rand -base64 32`
as described [here](https://rocket.rs/v0.5-rc/guide/configuration/#secret-key).
The key can either be set in the `Rocket.toml` or in a file with the name `.env` and the key `ROCKET_SECRET_KEY` if you want to use the `docker-compose.yml` file.


Afterwards run the application:
```console
$ cargo run --release
```
or using the docker image
```console
$ docker-compose up -d
```

The web interface will now be available on port `8000`.

## Usage
### User Management
Users must register using the icon in the top right corner and must be approved by using the cli tool:


```console
prankctl users approve <id>
```

where a user id can be found using

```console
prankctl users show --all
```

For other commands see
```console
prankctl --help
```


### Item Management
Items can be added by every user using the button `New Item` in the navigation bar.

Pinning a topic before the event should be done by using the site `/show`.
An item can be edited by clicking the edit button and then changing the date.

The cli tool can also be used using
```console
prankctl items discuss-on <id> <date>
```

or 

```console
prankctl items cancel-discuss <id>
```

All items where `discussed_on` is `NULL` (or unset) remain voteble.
The item will display at the top of the start page and at the top of the voting page until the date is reached (using UTC timezone) and only one item will be shown if multiple items have a date in the future (the item with a date that is "further away").
