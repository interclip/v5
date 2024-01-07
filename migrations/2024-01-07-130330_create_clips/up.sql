-- Your SQL goes here
CREATE TABLE `clips` (
    `id` int NOT NULL AUTO_INCREMENT,
    `code` text CHARACTER SET utf8 NOT NULL,
    `url` text CHARACTER SET utf8 NOT NULL,
    `created_at` date SET NOT NULL,
    `expires_at` date DEFAULT NULL,
    PRIMARY KEY (`id`)
) AUTO_INCREMENT = 0 DEFAULT CHARSET = utf8;