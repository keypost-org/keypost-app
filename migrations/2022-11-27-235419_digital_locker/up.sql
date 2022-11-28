CREATE TABLE lockers (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  locker_id VARCHAR NOT NULL,
  psswd_file TEXT NOT NULL,
  ciphertext TEXT NOT NULL,
  inserted_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
)