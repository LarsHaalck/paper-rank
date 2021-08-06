use lettre::message::{header, MultiPart, SinglePart};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use anyhow::{Context, Result};
use prank::item::Item;

use crate::format_item;

fn format_comment(comment: &Option<String>, html: bool) -> String {
    match comment {
        Some(c) => {
            // TODO: why is this needed?
            let c = c.replace("\\n", "\n");
            if html {
                format!("\n<hr><p>{}</p>", c)
            } else {
                format!("\n\n-------\n{}", c)
            }
        }
        None => String::new(),
    }
}

pub fn send(
    item: &Item,
    from: String,
    to: String,
    subject: Option<String>,
    comment: Option<String>,
    user: String,
    server: String,
) -> Result<()> {
    let subject = match subject {
        Some(s) => s,
        None => match item.discussed_on {
            Some(d) => format!("Next Paper for {:?}: {}", d, item.title),
            None => format!("Next Paper: {}", item.title),
        },
    };

    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::parse("text/markdown").unwrap())
                        .body(format!(
                            "{}{}",
                            format_item(item, false),
                            format_comment(&comment, false)
                        )),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!(
                            "{}{}",
                            format_item(item, true),
                            format_comment(&comment, true)
                        )),
                ),
        )?;

    let creds = Credentials::new(
        user,
        rpassword::prompt_password_stdout("Password: ").context("Error getting password")?,
    );

    let mailer = SmtpTransport::starttls_relay(&server)?
        .credentials(creds)
        .build();

    mailer.send(&email).context("Error sending mail")?;
    Ok(())
}
