use axum::{Router, extract::State, response::{Redirect, Html}, Form, routing::get};
use chrono::NaiveDateTime;
use tera::{Context, Tera};
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;


pub fn deserialize_date<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<NaiveDateTime, D::Error> {
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M")
        .map_err(serde::de::Error::custom)
}

#[derive(Serialize)]
struct Index {
    name: String
}

#[derive(Clone)]
struct ServiceState {
    tera: Tera,
    pool: PgPool,
}

#[derive(Debug, Serialize, FromRow)]   
struct Todo {
    id: Uuid,
    description: String,
    deadline_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    description: String,
    #[serde(deserialize_with = "deserialize_date")]
    deadline_at: NaiveDateTime,
}

async fn index(State(state): State<ServiceState>) -> Html<String> {
    let index = Index { name: String::from("test") };
    let page = state.tera.render("index.html", &Context::from_serialize(&index).unwrap()).unwrap();
    Html(page)
}

async fn get_create_todo(State(state): State<ServiceState>) -> Html<String> {
    let page = state.tera.render("create_todo.html", &Context::new()).unwrap();
    Html(page)
}

async fn post_create_todo(
    State(state): State<ServiceState>,
    Form(todo): Form<CreateTodo>,
) -> Redirect {
    let todo = Todo {
        id: Uuid::new_v4(),
        description: todo.description,
        deadline_at: todo.deadline_at,
    };

    sqlx::query("INSERT INTO todo VALUES ($1, $2, $3);")
        .bind(todo.id)
        .bind(todo.description)
        .bind(todo.deadline_at)
        .execute(&state.pool)
        .await
        .expect("todoの取得に失敗しました");

    Redirect::to("/todos")
}

async fn get_todos(
    State(state): State<ServiceState>,
) -> Html<String> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todo")
        .fetch_all(&state.pool)
        .await
        .expect("todoの取得に失敗しました");
    let mut context = Context::new();
    context.insert("todos", &todos);

    let page = state.tera.render("todos.html", &context).expect("todosの描画に失敗しました");
    Html(page)
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgresql://postgres:example@localhost:5432/postgres").await.unwrap();

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/todos", get(get_todos))
        .route("/create_todo", get(get_create_todo).post(post_create_todo))
        .with_state(ServiceState { tera, pool });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
