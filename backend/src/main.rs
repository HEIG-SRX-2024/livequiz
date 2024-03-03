use std::collections::HashMap;
use std::env;

use rocket::http::Header;
use rocket::serde::Deserialize;
use rocket::serde::{json::Json, Serialize};
use rocket::tokio::fs;
use rocket::{tokio::sync::Mutex, State};
use rocket::{Build, Request, Response, Rocket};

#[macro_use]
extern crate rocket;

struct Users {
    list: Mutex<HashMap<String, User>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[serde(crate = "rocket::serde")]
struct User {
    secret: String,
    name: Option<String>,
    answers: Vec<String>,
}

impl User {
    fn update_selected(&mut self, pos: usize, sel: String) {
        if self.answers.len() <= pos {
            self.answers.resize(pos + 1, "empty".to_string())
        }
        self.answers[pos] = sel;
    }

    fn new_with_selected(secret: String, pos: usize, sel: String) -> Self {
        let mut u = Self {
            secret,
            name: None,
            answers: vec!["empty".to_string(); pos],
        };
        u.answers[pos] = sel;
        u
    }
}

#[get("/api/v1/updateName?<secret>&<name>")]
async fn update_name(users: &State<Users>, secret: String, name: String) -> &'static str {
    let mut list = users.list.lock().await;
    list.entry(secret.clone())
        .and_modify(|u| u.name = Some(name.clone()))
        .or_insert_with(|| User {
            secret: secret.clone(),
            name: Some(name.clone()),
            answers: vec![],
        });
    println!("List is {list:?}");
    "{}"
}

#[get("/api/v1/updateQuestion?<secret>&<question>&<selected>")]
async fn update_question(
    users: &State<Users>,
    secret: String,
    question: usize,
    selected: String,
) -> &'static str {
    let mut list = users.list.lock().await;
    list.entry(secret.clone())
        .and_modify(|u| u.update_selected(question, selected.clone()))
        .or_insert_with(|| User::new_with_selected(secret, question, selected));
    println!("List is {list:?}");
    "{}"
}

#[get("/api/v1/getResults")]
async fn get_results(users: &State<Users>) -> Json<Vec<User>> {
    let list = users.list.lock().await;
    println!("List is {list:?}");
    Json(list.values().cloned().collect())
}

#[derive(Debug)]
struct Config {
    questionnaire: String,
    admin_secret: String,
    show_answers: Mutex<bool>,
}

const CONFIG_QUESTIONNAIRE_STRING: &str = "QUESTIONNAIRE_STRING";
const CONFIG_QUESTIONNAIRE: &str = "QUESTIONNAIRE";
const CONFIG_ADMIN_SECRET: &str = "ADMIN_SECRET";

impl Config {
    pub async fn new() -> Self {
        let questionnaire = Self::get_questionnaire().await;
        Self {
            questionnaire,
            admin_secret: env::var(CONFIG_ADMIN_SECRET)
                .expect(&format!("Need '{CONFIG_ADMIN_SECRET}'")),
            show_answers: Mutex::new(false),
        }
    }

    async fn get_questionnaire() -> String {
        if let Ok(q) = env::var(CONFIG_QUESTIONNAIRE_STRING) {
            return q;
        }
        fs::read_to_string(env::var(CONFIG_QUESTIONNAIRE).expect(&format!(
            "Need '{CONFIG_QUESTIONNAIRE}' environment variable"
        )))
        .await
        .expect("Couldn't read file")
    }
}

#[get("/api/v1/getQuestionnaire")]
async fn get_questionnaire(config: &State<Config>) -> String {
    config.questionnaire.clone()
}

#[get("/api/v1/getShowAnswers")]
async fn get_show_answers(config: &State<Config>) -> String {
    format!("{}", config.show_answers.lock().await)
}

#[get("/api/v1/setShowAnswers?<secret>&<show>")]
async fn set_show_answers(config: &State<Config>, secret: String, show: String) {
    if config.admin_secret == secret {
        *config.show_answers.lock().await = show == "true";
    }
}

#[launch]
async fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(CORS)
        .mount(
            "/",
            routes![
                update_name,
                update_question,
                get_results,
                get_questionnaire,
                get_show_answers,
                set_show_answers
            ],
        )
        .manage(Users {
            list: Mutex::new(HashMap::new()),
        })
        .manage(Config::new().await)
}

use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[cfg(test)]
mod test {
    use rocket::local::asynchronous::{Client, LocalResponse};

    use super::*;

    struct TestClient {
        c: Client,
    }

    const QUESTIONNAIRE_TEST: &str =
        "# Test Questions\n\n## Q1\nQuestion\n=1\n- choice1\n- choice2\n## End";

    impl TestClient {
        async fn new() -> Self {
            env::set_var(CONFIG_QUESTIONNAIRE_STRING, QUESTIONNAIRE_TEST);
            env::set_var(CONFIG_ADMIN_SECRET, "1234");
            Self {
                c: Client::tracked(rocket().await)
                    .await
                    .expect("valid rocket instance"),
            }
        }

        async fn update_name(&self, secret: &str, name: &str) -> LocalResponse {
            self.c
                .get(format!(
                    "/api/v1/updateName?secret={}&name={}",
                    secret, name
                ))
                .dispatch()
                .await
        }

        async fn update_question(
            &self,
            secret: &str,
            question: usize,
            selected: &str,
        ) -> LocalResponse {
            self.c
                .get(format!(
                    "/api/v1/updateQuestion?secret={}&question={}&selected={}",
                    secret, question, selected
                ))
                .dispatch()
                .await
        }

        async fn get_results(&self) -> Vec<User> {
            self.c
                .get("/api/v1/getResults")
                .dispatch()
                .await
                .into_json()
                .await
                .expect("Expected JSON")
        }

        async fn get_questionnaire(&self) -> String {
            self.c
                .get("/api/v1/getQuestionnaire")
                .dispatch()
                .await
                .into_string()
                .await
                .expect("No questionnaire")
        }

        async fn get_show_answers(&self) -> String {
            self.c
                .get("/api/v1/getShowAnswers")
                .dispatch()
                .await
                .into_string()
                .await
                .expect("No questionnaire")
        }

        async fn set_show_answers(&self, secret: String, show: String) {
            self.c
                .get(&format!(
                    "/api/v1/setShowAnswers?secret={}&show={}",
                    secret, show
                ))
                .dispatch()
                .await;
        }
    }

    #[async_test]
    async fn test_add_name() {
        let client = TestClient::new().await;
        let mut user1 = User {
            secret: "1234".to_string(),
            name: Some("foo".to_string()),
            answers: vec![],
        };
        assert_eq!(
            200,
            client
                .update_name(&user1.secret, user1.name.as_ref().unwrap())
                .await
                .status()
                .code
        );
        assert_eq!(vec![user1.clone()], client.get_results().await);
        user1.name = Some("bar".to_string());
        assert_eq!(
            200,
            client
                .update_name(&user1.secret, user1.name.as_ref().unwrap())
                .await
                .status()
                .code
        );
        assert_eq!(vec![user1.clone()], client.get_results().await);

        let user2 = User {
            secret: "1235".to_string(),
            name: Some("foobar".to_string()),
            answers: vec![],
        };
        assert_eq!(
            200,
            client
                .update_name(&user2.secret, user2.name.as_ref().unwrap())
                .await
                .status()
                .code
        );
        let mut users = client.get_results().await;
        users.sort();
        assert_eq!(vec![user1, user2], users);
    }

    #[async_test]
    async fn test_update_question() {
        let client = TestClient::new().await;
        let mut user = User {
            secret: "1234".to_string(),
            name: Some("foo".to_string()),
            answers: vec!["empty".to_string(), "correct".to_string()],
        };
        client
            .update_name(&user.secret, &user.name.as_ref().unwrap())
            .await;
        client
            .update_question(&user.secret, 1, &user.answers[1])
            .await;
        assert_eq!(vec![user.clone()], client.get_results().await);
        user.answers.insert(2, "correct".to_string());
        client
            .update_question(&user.secret, 2, &user.answers[2])
            .await;
        assert_eq!(vec![user], client.get_results().await);
    }

    #[async_test]
    async fn test_questionnaire() {
        let client = TestClient::new().await;
        assert_eq!(client.get_questionnaire().await, QUESTIONNAIRE_TEST);
    }

    #[async_test]
    async fn test_show_answers() {
        let client = TestClient::new().await;
        assert_eq!("false", client.get_show_answers().await);
        client.set_show_answers("12".to_string(), "true".to_string()).await;
        assert_eq!("false", client.get_show_answers().await);
        client.set_show_answers("1234".to_string(), "true".to_string()).await;
        assert_eq!("true", client.get_show_answers().await);
    }
}