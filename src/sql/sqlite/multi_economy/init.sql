CREATE TABLE IF NOT EXISTS `multi_economy` (
    `uuid` TEXT NOT NULL,
    `money` TEXT NOT NULL,
    `balance` INTEGER DEFAULT 0,
    PRIMARY KEY (`uuid`, `money`)
);

CREATE TABLE IF NOT EXISTS `multi_economy_key` (
    'money' TEXT UNIQUE NOT NULL,
    `key` TEXT NOT NULL
);
