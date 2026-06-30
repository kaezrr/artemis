CREATE TABLE media (
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('anime', 'movie', 'game', 'tvshow')),
  provider TEXT NOT NULL,
  provider_id INTEGER NOT NULL,
  title TEXT NOT NULL,
  cover_url TEXT NOT NULL,
  wide_url TEXT,
  logo_url TEXT,
  description TEXT NOT NULL,
  release_year INTEGER CHECK (release_year >= 0),
  rating INTEGER CHECK (rating BETWEEN 1 AND 7),
  notes TEXT,
  status TEXT NOT NULL CHECK (
    status IN (
      'planned',
      'in_progress',
      'finished',
      'on_hold',
      'dropped'
    )
  ) DEFAULT 'planned',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE (provider, provider_id)
);

CREATE TABLE media_tag (
  media_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  PRIMARY KEY (tag, media_id),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE
);

CREATE TABLE anime_meta (
  media_id INTEGER PRIMARY KEY,
  studio TEXT NOT NULL,
  episodes INTEGER NOT NULL CHECK (episodes >= 0),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE
);

CREATE TABLE movie_meta (
  media_id INTEGER PRIMARY KEY,
  director TEXT NOT NULL,
  duration INTEGER NOT NULL CHECK (duration >= 0),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE
);

CREATE TABLE game_meta (
  media_id INTEGER PRIMARY KEY,
  developer TEXT NOT NULL,
  playtime INTEGER CHECK (playtime >= 0),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE
);

CREATE TABLE tvshow_meta (
  media_id INTEGER PRIMARY KEY,
  director TEXT NOT NULL,
  episodes INTEGER NOT NULL CHECK (episodes >= 0),
  FOREIGN KEY (media_id) REFERENCES media (id) ON DELETE CASCADE
);
