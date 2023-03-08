-- Add migration script here
CREATE TABLE url (
    id  BIGSERIAL PRIMARY KEY,
    url varchar(255) NOT NULL,
    visit INT DEFAULT 0
);
