use crate::i18n::{Locale, td_string};
use crate::utils::constant;
use leptos_i18n::Locale as LocaleTrait;
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

pub async fn send_activation_email(
    lang: &str,
    username: &str,
    user_id: &str,
    email_to: &str,
) -> Result<(), String> {
    let domain = constant::config().domain.clone();
    let smtp = constant::config().email_smtp.clone();
    let from = constant::config().email_from.clone();
    let user = constant::config().email_username.clone();
    let pass = constant::config().email_password.clone();
    let url = format!("https://{domain}/users/{user_id}/activate");

    let locale = lang.parse::<Locale>().unwrap_or_default();
    let subject = td_string!(locale, email_activation_subject, username = username);
    let body = td_string!(
        locale,
        email_activation_body,
        username = username,
        url = url,
        domain = domain
    );

    let email = Message::builder()
        .from(
            from.parse()
                .map_err(|e: lettre::address::AddressError| e.to_string())?,
        )
        .to(email_to
            .parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(user, pass);
    let mailer = SmtpTransport::relay(&smtp)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();
    mailer.send(&email).map_err(|e| e.to_string())?;
    Ok(())
}
