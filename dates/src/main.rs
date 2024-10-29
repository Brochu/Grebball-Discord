use chrono::{ Utc, DateTime };

fn main() {
    let strdate = "2024-10-25T00:15:00Z";
    //let strdate = "2024-10-25T00:15Z";
    //let strdate = "2024-10-25T00:15Z";
    //let mut strdate = "2024-10-25T00:15Z".to_string();
    //strdate = strdate.replace("Z", ":00Z");

    let res = DateTime::parse_from_rfc3339(strdate)
        .map(|dt| dt.with_timezone(&Utc));

    match res {
        Ok(d) => println!("Got the date -> {}", d),
        Err(e) => println!("Got an error -> {}", e),
    }
}
