use crate::{models::my_errors::MyErrors, workers::mailer::args::EmailArgs};
use lettre::{
  message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart},
  transport::smtp::authentication::Credentials,
  AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn process_email(args: EmailArgs) -> Result<(), MyErrors> {
  tracing::info!("Start sending email to: {}", args.to);

  let email = build_email(&args)?;
  send_email(email).await?;

  tracing::info!("Email sent successfully");
  Ok(())
}

fn build_email(args: &EmailArgs) -> Result<Message, MyErrors> {
  let to: Mailbox = if let Some(name) = &args.to_name {
    format!("{} <{}>", name, args.to).parse()?
  } else {
    args.to.parse()?
  };

  let smtp_user = std::env::var("SMTP_AUTH_USER")?;
  let from_name = args.from_name.clone().unwrap_or("OpenCab".to_string());

  let mut message_builder = Message::builder()
    .from(format!("{} <{}>", from_name, smtp_user).parse()?)
    .to(to)
    .subject(&args.subject);

  if let Some(reply_to) = &args.reply_to {
    message_builder = message_builder.reply_to(reply_to.parse()?);
  }

  let message = if args.attachments.is_empty() {
    build_simple_body(message_builder, args)?
  } else {
    build_multipart_body(message_builder, args)?
  };

  Ok(message)
}

fn build_simple_body(
  builder: lettre::message::MessageBuilder,
  args: &EmailArgs,
) -> Result<Message, MyErrors> {
  Ok(builder.body(args.text_body.clone())?)
}

fn build_multipart_body(
  builder: lettre::message::MessageBuilder,
  args: &EmailArgs,
) -> Result<Message, MyErrors> {
  let mut multipart = MultiPart::mixed().singlepart(SinglePart::plain(args.text_body.clone()));

  for attachment in &args.attachments {
    let data = attachment.decode_data()?;
    let content_type: ContentType = attachment.content_type.parse()?;

    multipart =
      multipart.singlepart(Attachment::new(attachment.filename.clone()).body(data, content_type));
  }

  Ok(builder.multipart(multipart)?)
}

async fn send_email(email: Message) -> Result<(), MyErrors> {
  let smtp_host = std::env::var("SMTP_SERVER_HOST")?;
  let smtp_port: u16 = std::env::var("SMTP_SERVER_PORT")?.parse()?;
  let smtp_user = std::env::var("SMTP_AUTH_USER")?;
  let smtp_password = std::env::var("SMTP_AUTH_PASSWORD")?;

  let creds = Credentials::new(smtp_user, smtp_password);

  let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
    .credentials(creds)
    .port(smtp_port)
    .build();

  transport.send(email).await?;

  Ok(())
}
