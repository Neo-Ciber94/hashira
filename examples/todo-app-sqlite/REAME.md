# Todo example

A hashira `todo` example using:

- Database using `sqlx` with **SQLite**
- TailwindCSS
- Server Actions
- Nested routes

## Prerequisites

- Hashira CLI:
  - <https://crates.io/crates/hashira-cli>
    - `cargo install hashira-cli --force`
- sqlx-cli
  - <https://crates.io/crates/sqlx-cli>
- node
  - <https://nodejs.org/>

## How to run

- Install the development node packages
  - `npm install`
- Create the database:
  - `sqlx database create`
- Apply the migration
  - `sqlx migrate run`
- Set the database URL
  - **(Windows Powershell)** `$env:DATABASE_URL={your_path}/todo.db`
  - **(Windows CMD)** `set DATABASE_URL={your_path}/todo.db`
  - **(Linux)** `export DATABASE_URL={your_path}/todo.db`
    - Replace `{your_path}` with the path of the directory
- Starts in watch mode
  - `hashira dev`
