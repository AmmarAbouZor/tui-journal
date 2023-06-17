CREATE TABLE IF NOT EXISTS tags (
  entry_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  PRIMARY KEY (entry_id, tag)
  FOREIGN KEY (entry_id) REFERENCES entries (id) ON DELETE CASCADE
) 

