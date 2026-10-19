// ============================================================
// Todo API with Axum + SQLx (SQLite database)
// Step 7: our todos are now saved in a real database file,
// so they are still there after we stop and restart the server.
// ============================================================

// IntoResponse and Response let us turn our own error type into an HTTP response
use axum::response::{IntoResponse, Response};
// Path reads values from the URL (like the 1 in /todos/1)
// State gives each handler access to our shared database pool
use axum::extract::{Path, State};
// StatusCode has the HTTP codes: 200, 201, 204, 400, 404, 500 ...
use axum::http::StatusCode;
// get is the routing helper for GET routes
use axum::routing::get;
// Json reads and writes JSON; Router holds all our routes
use axum::{Json, Router};
// Serialize turns a Rust struct into JSON, Deserialize turns JSON into a Rust struct
use serde::{Deserialize, Serialize};
// SqliteConnectOptions = settings for how to open the database file
use sqlx::sqlite::SqliteConnectOptions;
// SqlitePoolOptions = a builder to create the pool of database connections
use sqlx::sqlite::SqlitePoolOptions;
// SqlitePool = a group of ready-to-use database connections that we share
use sqlx::SqlitePool;

// ------------------------------------------------------------
// DATA TYPES
// ------------------------------------------------------------

// A todo item, exactly as it exists in the database and in our JSON replies
// Serialize: Todo -> JSON (for sending to the client)
// sqlx::FromRow: a database row -> Todo (for reading from the database)
#[derive(Serialize, sqlx::FromRow)]
struct Todo {
    id: i64,         // unique number; SQLite integers are i64 in Rust
    title: String,   // the text, like "Learn Axum"
    completed: bool, // true = done, false = not done
}

// The JSON the client sends to create a todo, like {"title":"Buy milk"}
// Deserialize: JSON -> CreateTodo
#[derive(Deserialize)]
struct CreateTodo {
    title: String, // the client only sends the title; the database gives the id
}

// The JSON the client sends to update a todo
#[derive(Deserialize)]
struct UpdateTodo {
    title: String,   // the new title
    completed: bool, // the new done / not done value
}

// ------------------------------------------------------------
// ERRORS
// ------------------------------------------------------------

// All the things that can go wrong in our API
enum AppError {
    // The todo with that id does not exist -> 404
    NotFound,
    // The client sent bad data; the String is the message for them -> 400
    BadRequest(String),
    // Something broke inside the database -> 500
    // sqlx::Error is the error type that SQLx gives us
    Database(sqlx::Error),
}

// The JSON shape of every error reply, like {"error":"Todo not found"}
#[derive(Serialize)]
struct ErrorBody {
    error: String, // the message for the client
}

// This tells Axum how to turn our AppError into a real HTTP response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Choose the status code and the message for each kind of error
        let (status, message) = match self {
            // 404 with a clear message
            AppError::NotFound => (StatusCode::NOT_FOUND, String::from("Todo not found")),
            // 400 with the message we were given
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            // 500: print the real error in OUR terminal, but show the client a safe message
            // (never send database details to the client, it can leak secrets)
            AppError::Database(err) => {
                eprintln!("Database error: {err}"); // eprintln! prints to the terminal
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Something went wrong"),
                )
            }
        };
        // Build the response: (status code, JSON body), then convert it into a Response
        (status, Json(ErrorBody { error: message })).into_response()
    }
}

// This lets us use the `?` operator on database calls inside our handlers
// If a database call fails, `?` automatically turns sqlx::Error into AppError::Database
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err) // wrap the database error in our own type
    }
}

// A helper that checks a title and cleans it
// Input: the title the client sent. Output: the cleaned title, or a 400 error
fn clean_title(title: String) -> Result<String, AppError> {
    // trim() removes spaces at the start and end; to_string() makes an owned String
    let trimmed = title.trim().to_string();

    // If nothing is left, the title was empty (or only spaces)
    if trimmed.is_empty() {
        // Stop and return a 400 error with a clear message
        return Err(AppError::BadRequest(String::from("Title cannot be empty")));
    }

    // The title is fine, so return it inside Ok
    Ok(trimmed)
}

// ------------------------------------------------------------
// MAIN: start the database and the server
// ------------------------------------------------------------

// #[tokio::main] turns main into an async function run by Tokio
#[tokio::main]
async fn main() {
    // Describe how to open the database: a file called todos.db in our project folder
    let options = SqliteConnectOptions::new()
        .filename("todos.db")       // the database lives in this file
        .create_if_missing(true);   // create the file if it does not exist yet

    // Create a pool of up to 5 database connections that handlers can share
    let pool = SqlitePoolOptions::new()
        .max_connections(5)             // at most 5 connections at the same time
        .connect_with(options)          // open the database using our options
        .await                          // wait until it is ready
        .expect("Could not open the database"); // crash with this message if it fails

    // Create the todos table if it is not there yet
    // This runs every time the server starts, but IF NOT EXISTS makes it safe
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool) // run this SQL using our pool
    .await          // wait for the database to finish
    .expect("Could not create the todos table"); // crash with this message if it fails

    // Build the router: it connects URLs + HTTP methods to our handler functions
    let app = Router::new()
        // GET / -> the hello handler
        .route("/", get(hello))
        // GET /todos -> list all todos, POST /todos -> create a todo
        .route("/todos", get(list_todos).post(create_todo))
        // GET /todos/1 -> read one, PUT /todos/1 -> update one, DELETE /todos/1 -> delete one
        // {id} is a placeholder for the number in the URL
        .route(
            "/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        // Give the database pool to every handler that asks for State
        .with_state(pool);

    // Open port 3000 on our own computer
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap(); // unwrap = crash if the port is already used (fine for learning)

    // Print a message so we know the server started
    println!("Server running on http://127.0.0.1:3000");

    // Start the server: listen on the port and send every request to our router
    axum::serve(listener, app).await.unwrap();
}

// ------------------------------------------------------------
// HANDLERS: one function for each thing our API can do
// ------------------------------------------------------------

// GET / -> a simple text reply to check that the server works
async fn hello() -> &'static str {
    "Hello from Axum!" // this text becomes the response body
}

// GET /todos -> return all todos as a JSON array
async fn list_todos(State(pool): State<SqlitePool>) -> Result<Json<Vec<Todo>>, AppError> {
    // query_as runs a SELECT and turns each row into a Todo
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos ORDER BY id")
        .fetch_all(&pool) // fetch_all = get every row as a list
        .await?;          // wait for the database; `?` returns an error if it failed

    // Send the list back as JSON (status 200 by default)
    Ok(Json(todos))
}

// POST /todos -> create a new todo and return it
// Json must be the LAST extractor, because it reads the request body
async fn create_todo(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    // Check the title; `?` stops here with a 400 error if the title is empty
    let title = clean_title(payload.title)?;

    // Insert a new row; the database picks the id for us
    // The ? is a placeholder; bind() fills it in safely
    // RETURNING gives us the new row back, so we do not need a second query
    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title) VALUES (?) RETURNING id, title, completed",
    )
    .bind(title)        // put the title into the first ?
    .fetch_one(&pool)   // fetch_one = we expect exactly one row back
    .await?;            // wait for the database; `?` returns an error if it failed

    // 201 CREATED is the correct status when we make something new
    Ok((StatusCode::CREATED, Json(todo)))
}

// GET /todos/{id} -> return one todo, or a 404 error
async fn get_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>, // take the number from the URL
) -> Result<Json<Todo>, AppError> {
    // fetch_optional = give us Some(row) if found, or None if there is no such row
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
        .bind(id)             // put the id into the ?
        .fetch_optional(&pool)
        .await?;

    // ok_or turns None into our NotFound error, and Some(todo) into Ok(todo)
    let todo = todo.ok_or(AppError::NotFound)?;

    // Send the todo back as JSON
    Ok(Json(todo))
}

// PUT /todos/{id} -> change the title and completed of one todo
async fn update_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    // Check and clean the new title first
    let title = clean_title(payload.title)?;

    // UPDATE changes the row where the id matches, RETURNING gives us the new version
    // If no row has this id, nothing changes and we get None
    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = ?, completed = ? WHERE id = ? RETURNING id, title, completed",
    )
    .bind(title)               // first ?  -> the new title
    .bind(payload.completed)   // second ? -> the new completed value
    .bind(id)                  // third ?  -> which todo to change
    .fetch_optional(&pool)
    .await?;

    // None becomes a 404 error, Some(todo) becomes a success reply
    let todo = todo.ok_or(AppError::NotFound)?;

    // Send the updated todo back as JSON
    Ok(Json(todo))
}

// DELETE /todos/{id} -> remove one todo
async fn delete_todo(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // execute runs the SQL and tells us how many rows were changed
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    // If 0 rows were deleted, that id did not exist
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound); // 404
    }

    // 204 NO CONTENT = deleted successfully, nothing to send back
    Ok(StatusCode::NO_CONTENT)
}