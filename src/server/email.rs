use crate::utils::constant;
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

pub async fn send_activation_email(
    lang: &str,
    username: &str,
    nickname: &str,
    user_id: &str,
    email_to: &str,
) -> Result<(), String> {
    let domain = constant::config().domain.clone();
    let smtp = constant::config().email_smtp.clone();
    let from = constant::config().email_from.clone();
    let user = constant::config().email_username.clone();
    let pass = constant::config().email_password.clone();
    let url = format!("https://{domain}/users/{user_id}/activate");

    let (subject, body) = if lang == "zh" {
        (
            format!("{nickname}（{username}），来自毕剖的账户激活邮件"),
            format!(
                "你好，{nickname}（{username}）！\n\n请点击以下链接激活您的毕剖账户：\n\n{url}\n\n若非本人操作，请忽略此邮件。\n\n毕剖 https://{domain}"
            ),
        )
    } else {
        (
            format!("{nickname} ({username}), account activation email from bipou.com"),
            format!(
                "Hi {nickname} ({username})!\n\nPlease click the link below to activate your BiPou account:\n\n{url}\n\nIf you did not register, please ignore this email.\n\nBiPou https://{domain}"
            ),
        )
    };

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
