use std::{env, error::Error, process::Command};

use migration::MigratorTrait;
use opencab::config::Config;
use sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenvy::from_filename(".env.local").ok();

  let environment = env::var("ENVIRONMENT").unwrap_or("development".to_string());
  let config = Config::load(&environment)?;

  // Parse command line arguments
  let args: Vec<String> = env::args().collect();
  let command = args.get(1).map(|s| s.as_str()).unwrap_or("up");

  println!("Connecting to database...");
  let db = Database::connect(&config.database.url).await?;

  match command {
    "up" => {
      println!("Running migrations...");
      migration::Migrator::up(&db, None).await?;
      println!("Migrations run successfully!");
    }
    "down" => {
      // Check if there's a number argument for partial rollback
      let steps = args.get(2).and_then(|s| s.parse::<u32>().ok());

      if let Some(n) = steps {
        println!("Rolling back last {} migration(s)...", n);
        migration::Migrator::down(&db, Some(n)).await?;
        println!("Rolled back {} migration(s) successfully!", n);
      } else {
        println!("Rolling back all migrations...");
        migration::Migrator::down(&db, None).await?;
        println!("All migrations rolled back successfully!");
      }
    }
    _ => {
      eprintln!("Unknown command: {}", command);
      eprintln!("Usage:");
      eprintln!("  cargo run --bin migrate          # Run all pending migrations");
      eprintln!("  cargo run --bin migrate up       # Run all pending migrations");
      eprintln!("  cargo run --bin migrate down     # Rollback all migrations");
      eprintln!("  cargo run --bin migrate down 1   # Rollback last migration");
      eprintln!("  cargo run --bin migrate down N   # Rollback last N migrations");
      std::process::exit(1);
    }
  }

  if environment == "development" {
    generate_entities(&config.database.url)?;
  }

  Ok(())
}

fn generate_entities(database_url: &str) -> Result<(), Box<dyn Error>> {
  println!("\nGenerating entities...");

  let output = Command::new("sea-orm-cli")
    .args([
      "generate",
      "entity",
      "--database-url",
      database_url,
      "--with-serde",
      "both",
      // SeaORM 2.0 "dense" format: `#[sea_orm::model]`, inline relations, strongly-typed `COLUMN`.
      "--entity-format",
      "dense",
      // ActiveModelBehavior impls are provided by our model wrappers (and 2 entities), so don't
      // generate empty ones that would conflict.
      "--impl-active-model-behavior=false",
      "-o",
      "./src/models/_entities/",
    ])
    .output()?;

  if output.status.success() {
    println!("Entities generated successfully!");
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("Failed to generate entities: {}", stderr);
    return Err("Entity generation failed".into());
  }

  Ok(())
}
