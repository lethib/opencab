use std::{env, sync::Arc};

use opencab::{
  config::Config,
  models::{
    _entities::users::Entity as Users,
    my_errors::{unexpected_error::UnexpectedError, MyErrors},
  },
  workers::mailer::{self, args::EmailArgs},
};
use sea_orm::{Database, EntityTrait};

#[tokio::main]
async fn main() -> Result<(), MyErrors> {
  dotenvy::from_filename(".env.local").ok();

  let environment = env::var("ENVIRONMENT").unwrap_or("development".to_string());
  let config = Arc::new(Config::load(&environment)?);

  let args: Vec<String> = env::args().collect();
  let user_id: i32 = args
    .get(1)
    .ok_or(UnexpectedError::new("user_id must be provided".to_string()))?
    .parse()?;

  let db = Database::connect(&config.database.url).await?;

  let user_to_invite = Users::find_by_id(user_id)
    .one(&db)
    .await?
    .ok_or(UnexpectedError::new("user_not_found".to_string()))?;

  match user_to_invite.access_key {
    Some(access_key) => {
      let email_args = EmailArgs::new_text(
          user_to_invite.email,
          "Votre code d'accès à OpenCab".to_string(),
          format!("Bonjour,\n\nVoici votre code d'accès à la plateforme OpenCab: {}\nVous pouvez l'utiliser juste après vous être connecté: {}/login", access_key, config.app.base_url)
      );

      println!("Sending email...");

      mailer::worker::process_email(email_args).await?;

      println!("Email sent successfully !")
    }
    None => println!("No access key registered for this user"),
  }

  Ok(())
}
