/* sqlite: */
CREATE TABLE "article" (
  "a_id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  "a_class" TEXT,
  "a_content" TEXT
);

/* 
mysql:

CREATE TABLE `article` (
  `a_id` INT NOT NULL AUTO_INCREMENT,
  `a_class` TEXT,
  `a_content` TEXT,
  PRIMARY KEY (`a_id`)
); 
*/

