use lettre::message::{header, MultiPart, SinglePart};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use prank::item::Item;
use anyhow::{Result, Context};

use crate::format_item;

pub fn send(
    item: &Item,
    from: String,
    to: String,
    subject: Option<String>,
    user: String,
    server: String,
) -> Result<()> {
    let subject = match subject {
        Some(s) => s,
        None => {
            match item.discussed_on {
                Some(d) => format!("Next Paper for {:?}: {}", d, item.title),
                None => format!("Next Paper: {}", item.title)
            }
        }
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
                        .body(format_item(item, false)),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format_item(item, true)),
                ),
        )?;

    let creds = Credentials::new(
        user,
        rpassword::prompt_password_stdout("Password: ")
            .context("Error getting password")?,
    );

    let mailer = SmtpTransport::starttls_relay(&server)?
        .credentials(creds)
        .build();

    mailer.send(&email).context("Error sending mail")?;
    Ok(())
}
