use lettre::message::{header, MultiPart, SinglePart};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use crate::item::{Item, ItemFormat};
use anyhow::{Context, Result};

fn format_comment(comment: &Option<String>, item_format: ItemFormat) -> String {
    match comment {
        Some(c) => {
            // TODO: why is this needed?
            let c = c.replace("\\n", "\n");
            match item_format {
                ItemFormat::HTML => format!("\n<hr><p>{}</p>", c),
                ItemFormat::Markdown => format!("\n\n-------\n{}", c),
            }
        }
        None => String::new(),
    }
}

pub fn send(
    item: &Item,
    from: String,
    to: String,
    comment: Option<String>,
    user: String,
    server: String,
    password: String,
) -> Result<()> {
    let subject = match item.discussed_on {
        Some(d) => format!("Next Paper for {:?}: {}", d, item.title),
        None => format!("Next Paper: {}", item.title),
    };

    let email = Message::builder()
        .from(from.parse().with_context(|| format!("Failed to parse email field from: {}", from))?)
        .to(to.parse().with_context(|| format!("Failed to parse email field to: {}", to))?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::parse("text/markdown").unwrap())
                        .body(format!(
                            "{}{}",
                            item.format(ItemFormat::Markdown),
                            format_comment(&comment, ItemFormat::Markdown)
                        )),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!(
                            "{}{}",
                            item.format(ItemFormat::HTML),
                            format_comment(&comment, ItemFormat::HTML)
                        )),
                ),
        )?;

    let creds = Credentials::new(user, password);
    let mailer = SmtpTransport::starttls_relay(&server)?
        .credentials(creds)
        .build();

    mailer.send(&email).context("Error sending mail")?;
    Ok(())
}
