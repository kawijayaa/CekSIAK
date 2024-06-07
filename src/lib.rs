use reqwest::cookie::Jar;
use reqwest::Certificate;
use reqwest::{multipart, Client};
use scraper::Selector;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

pub struct SIAKSession {
    pub client: reqwest::Client,
    pub jar: Arc<Jar>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SIAKCourse {
    pub course_code: String,
    pub curriculum: String,
    pub name_indonesian: String,
    pub name_english: String,
    pub status: String,
}

impl SIAKSession {
    pub fn new() -> Self {
        let mut buf = Vec::new();
        let _ = File::open("./ui.ac.id.pem").unwrap().read_to_end(&mut buf);
        let cert = Certificate::from_pem(&buf).unwrap();

        let jar = Arc::new(Jar::default());

        let client = Client::builder()
            .use_rustls_tls()
            .add_root_certificate(cert)
            .cookie_store(true)
            .cookie_provider(jar.clone())
            .build()
            .unwrap();

        Self { client, jar }
    }

    pub async fn login(&self, username: String, password: String) -> Result<(), String> {
        let form = multipart::Form::new()
            .text("u", username)
            .text("p", password);

        self.client
            .request(
                reqwest::Method::POST,
                "https://academic.ui.ac.id/main/Authentication/Index",
            )
            .multipart(form)
            .send()
            .await
            .unwrap();

        self.client
            .request(
                reqwest::Method::GET,
                "https://academic.ui.ac.id/main/Authentication/ChangeRole",
            )
            .send()
            .await
            .unwrap();

        Ok(())
    }

    pub async fn get_scores(&self) -> Option<Vec<SIAKCourse>> {
        let resp = self
            .client
            .get("https://academic.ui.ac.id/main/Academic/HistoryByTerm")
            .send()
            .await
            .unwrap();

        let data = resp.text().await.unwrap();
        let parser = scraper::Html::parse_document(&data);
        let a = match parser
            .select(&Selector::parse("table.box > tbody > tr:not(.x, .alt)").unwrap())
            .last()
        {
            Some(a) => a,
            None => {
                log::error!("{:?}", data);
                return None;
            }
        };
        let siblings = a.next_siblings();
        let mut courses: Vec<SIAKCourse> = Vec::new();

        for sib in siblings {
            if sib.has_children() {
                let course_code = sib
                    .children()
                    .skip(3)
                    .next()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_text()
                    .unwrap()
                    .to_string();
                let curriculum = sib
                    .children()
                    .skip(5)
                    .next()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_text()
                    .unwrap()
                    .to_string();
                let name_indonesian = sib
                    .children()
                    .skip(7)
                    .next()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_text()
                    .unwrap()
                    .to_string();
                let name_english = sib
                    .children()
                    .skip(9)
                    .next()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_text()
                    .unwrap()
                    .to_string();
                let status = match sib
                    .children()
                    .skip(15)
                    .next()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .value()
                    .as_text()
                    .unwrap()
                    .to_string()
                    .as_str()
                {
                    "Empty" => "Empty",
                    "Not published" => "Not published",
                    _ => "Published",
                };
                courses.push(SIAKCourse {
                    course_code,
                    curriculum,
                    name_indonesian,
                    name_english,
                    status: status.to_string(),
                });
            }
        }

        Some(courses)
    }

    pub fn is_courses_updated(courses: &Vec<SIAKCourse>) -> bool {
        let file = match File::open("./courses.json") {
            Ok(file) => file,
            Err(_) => return true,
        };
        let contents: Vec<SIAKCourse> = match serde_json::from_reader(file) {
            Ok(contents) => contents,
            Err(_) => Vec::new(),
        };

        for course in courses {
            if !contents.contains(&course) {
                return true;
            }
            if course.status
                != contents[contents
                    .iter()
                    .position(|c| c.course_code == course.course_code)
                    .unwrap()]
                .status
            {
                return true;
            }
        }

        false
    }

    pub fn save_courses(courses: &Vec<SIAKCourse>) {
        let file = File::create("./courses.json").unwrap();
        serde_json::to_writer(file, courses).unwrap();
    }
}
