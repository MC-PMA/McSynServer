CREATE TABLE IF NOT EXISTS `single_economy` (
    `uuid` TEXT UNIQUE NOT NULL,
    `balance` INTEGER DEFAULT 0
);