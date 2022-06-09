CREATE TABLE IF NOT EXISTS messages(
  id          SERIAL PRIMARY KEY,
  payload     JSON NOT NULL
);