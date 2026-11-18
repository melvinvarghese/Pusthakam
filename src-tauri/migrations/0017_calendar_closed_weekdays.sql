-- Recurring closures use SQLite weekday numbering: Sunday = 0 through Saturday = 6.
CREATE TABLE closed_weekdays (
    weekday INTEGER PRIMARY KEY CHECK (weekday BETWEEN 0 AND 6)
);
