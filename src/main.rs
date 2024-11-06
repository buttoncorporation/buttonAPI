use redb::{Database, Error, ReadableTable, TableDefinition};
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{UNIX_EPOCH, Duration};
#[macro_use] extern crate rocket;

// FARING for CORS
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "https://buttoncorp.org"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}










// Main home page (will also be processed as 404 page)
// nothing complex
#[get("/")]
fn index() -> &'static str {
    "Hello. Please ignore this page and return to our site https://buttoncorp.org. Thank you."
}


// API hook for site opening
// takes in uuid and epoch timestamp to generate a key, and then insert key into the database with
// the value (open)
#[get("/open/<uuid>/<date>")]
fn open(uuid: &str, date: u64) -> String {
    let timestamp_str = get_time_string(date);
    let current_table: TableDefinition<&str, bool> = TableDefinition::new(&timestamp_str);
    let write_transaction = get_db_handle().begin_write().unwrap();
    {
        let mut table = write_transaction.open_table(current_table).unwrap();
        table.insert(uuid, false).expect("Error writing to database");
    }
    write_transaction.commit();

    format!("{} submitted for button {}", uuid, timestamp_str)
}


// API hook for site opening
// takes in uuid and epoch timestamp to generate a key, and then insert key into the database with
// the value (open)
#[get("/press/<uuid>/<date>")]
fn press(uuid: &str, date: u64) -> String {
    let timestamp_str = get_time_string(date);
    let current_table: TableDefinition<&str, bool> = TableDefinition::new(&timestamp_str);
    let write_transaction = get_db_handle().begin_write().unwrap();
    {
        let mut table = write_transaction.open_table(current_table).unwrap();
        table.insert(uuid, true).expect("Error writing to database");
    }
    write_transaction.commit();

    format!("{} submitted for button {}", uuid, timestamp_str)
}


#[get("/read/<date>")]
fn read(date: u64) -> String {
    let date_string = get_time_string(date);
    let current_table: TableDefinition<&str, bool> = TableDefinition::new(&date_string);
    let read_transaction = get_db_handle().begin_read().unwrap();
    let mut open_count: u8 = 0;
    let mut press_count: u8 = 0;
    {
        let table = read_transaction.open_table(current_table).unwrap();
        let mut range = table.iter().unwrap();
        for item in range {
            let (uuid, choice) = item.unwrap();
            if choice.value() { press_count+=1;}
            else {open_count+=1;}
        }
    }
        format!("{} didn't press the button, {} pressed the button on BUT-{}",open_count,press_count,date_string)
}


// Rocket launch
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![open])
        .mount("/", routes![read, press])
        .attach(CORS)
}

// Transforms timestamp into a timestamp string
fn get_time_string(timestamp: u64) -> String {
    let d = UNIX_EPOCH + Duration::from_secs(timestamp);
    let datetime = DateTime::<Utc>::from(d);
    let timestamp_str = datetime.format("%Y-%m-%d").to_string();
    timestamp_str
}


// Grabs a DB handle.
// Keeps things less messy about handing DB back and forth and startup and all that
fn get_db_handle() -> redb::Database {
    let database = Database::open("buttoncorp.redb");
    match database {
        Ok(handle) => return handle,
        Err(_e) => {
        let handle = Database::create("buttoncorp.redb");
        return handle.expect("Database creation failed.");
        }
    }
}


// TO DO:
// +comment code
// +record into database on open
// +duplicate open into press
// +update frontend with new cookie maths
// +set up webhooks with basic analytics
