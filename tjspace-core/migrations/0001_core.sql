CREATE TABLE IF NOT EXISTS packages (
  package_id TEXT PRIMARY KEY,
  version TEXT NOT NULL,
  source_path TEXT NOT NULL,
  installed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  data_purged BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE TABLE IF NOT EXISTS services (
  package_id TEXT PRIMARY KEY REFERENCES packages(package_id) ON DELETE CASCADE,
  state TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS backups (
  monolith_id TEXT PRIMARY KEY,
  package_id TEXT NOT NULL,
  target_id TEXT NOT NULL,
  state TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id UUID PRIMARY KEY,
  sha256 TEXT NOT NULL,
  signature BYTEA NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_services_state ON services(state);
CREATE INDEX IF NOT EXISTS idx_backups_package ON backups(package_id);
