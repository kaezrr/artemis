CREATE TABLE collection (id INTEGER PRIMARY KEY, title TEXT NOT NULL);

CREATE TABLE collection_media (
  collection_id INTEGER NOT NULL,
  media_id INTEGER NOT NULL,
  PRIMARY KEY (collection_id, media_id),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE,
  FOREIGN KEY (collection_id) REFERENCES collection (id) ON DELETE CASCADE
);
