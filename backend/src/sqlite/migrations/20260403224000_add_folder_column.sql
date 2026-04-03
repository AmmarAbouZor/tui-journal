-- Add folder column to entries table
ALTER TABLE entries ADD COLUMN folder TEXT NOT NULL DEFAULT '';
