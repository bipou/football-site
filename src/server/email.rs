use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

pub async fn send_activation_email(
    lang: &str,
    username: &str,
    nickname: &str,
    user_id: &str,
    email_to: &str,
) -> Result<(), String> {
    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost:3000".to_string());
    let smtp   = std::env::var("EMAIL_SMTP").map_err(|e| e.to_string())?;
    let from   = std::env::var("EMAIL_FROM").map_err(|e| e.to_string())?;
    let user   = std::env::var("EMAIL_USERNAME").map_err(|e| e.to_string())?;
    let pass   = std::env::var("EMAIL_PASSWORD").map_err(|e| e.to_string())?;
    let url    = format!("https://{domain}/users/{user_id}/activate");

    let (subject, body) = if lang == "zh" {
        (
            format!("{nickname}（{username}），来自毕剖的账户激活邮件"),
            format!(
                "你好，{nickname}（{username}）！\n\n\
                 请点击以下链接激活您的毕剖账户：\n\n{url}\n\n\
                 若非本人操作，请忽略此邮件。\n\n毕剖 https://{domain}"
            ),
        )
    } else {
        (
            format!("{nickname} ({username}), account activation email from bipou.com"),
            format!(
                "Hi {nickname} ({username})!\n\n\
                 Please click the link below to activate your BiPou account:\n\n{url}\n\n\
                 If you did not register, please ignore this email.\n\nBiPou https://{domain}"
            ),
        )
    };

    let email = Message::builder()
        .from(from.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .to(email_to.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| e.to_string())?;

    let creds  = Credentials::new(user, pass);
    let mailer = SmtpTransport::relay(&smtp)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;
    Ok(())
}
