# Todo API (Rust + Axum + SQLite)

A small REST API to create, read, update and delete todos.
Built with **Axum** (web framework), **Tokio** (async runtime), **Serde** (JSON) and **SQLx** (database).
Todos are saved in a SQLite file, so they survive a server restart.

## What you need

- [Rust](https://www.rust-lang.org/tools/install) (comes with `cargo`)
- Something to send requests: Postman, Thunder Client, or `curl`

## Run it

1. Open a terminal in the project folder.
2. Start the server:

```bash
   cargo run
```

3. You will see `Server running on http://127.0.0.1:3000`.
4. Open `http://127.0.0.1:3000` in the browser. You should see `Hello from Axum!`

The first run creates a `todos.db` file. That is your database.

## Endpoints

| Method | URL           | What it does   | Body (JSON)                                |
| ------ | ------------- | -------------- | ------------------------------------------ |
| GET    | `/`           | Check server   | none                                       |
| GET    | `/todos`      | List all todos | none                                       |
| POST   | `/todos`      | Create a todo  | `{"title": "Buy milk"}`                    |
| GET    | `/todos/{id}` | Get one todo   | none                                       |
| PUT    | `/todos/{id}` | Update a todo  | `{"title": "Buy milk", "completed": true}` |
| DELETE | `/todos/{id}` | Delete a todo  | none                                       |

## Try it

Create a todo:

```bash
curl -X POST http://127.0.0.1:3000/todos -H "Content-Type: application/json" -d '{"title":"Buy milk"}'
```

List all todos:

```bash
curl http://127.0.0.1:3000/todos
```

On Windows PowerShell, use Postman or `curl.exe` with escaped quotes (`"{\"title\":\"Buy milk\"}"`).

## Status codes

- `200` OK
- `201` Created (new todo)
- `204` Deleted (no content)
- `400` Bad request (for example, an empty title)
- `404` Todo not found
- `500` Server or database error

Errors come back as JSON, like `{"error": "Todo not found"}`.

## Project files

- `src/main.rs`: all the code (types, errors, routes, handlers)
- `Cargo.toml`: the list of libraries
- `todos.db`: the SQLite database (created automatically)

## Tips

- Delete `todos.db` (with the server stopped) to start with an empty list.
- Port 3000 already in use? Close the other program, or change `3000` in `main.rs`.
- Add `todos.db` and `target/` to `.gitignore` if you use Git.

## What I learned building this

Routes, JSON with Serde, extractors (`State`, `Path`, `Json`), shared state,
CRUD, custom errors with `IntoResponse`, and a real database with SQLx.
