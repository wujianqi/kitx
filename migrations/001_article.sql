/* sqlite: */
CREATE TABLE article (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    tenant_id INTEGER NOT NULL DEFAULT 0,
    title TEXT NOT NULL,
    content TEXT,
    views INTEGER DEFAULT 0,
    deleted INTEGER DEFAULT 0 CHECK(deleted IN (0, 1)),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE article_tag (
    article_id INTEGER NOT NULL,
    share_seq INTEGER NOT NULL,
    tenant_id INTEGER NOT NULL DEFAULT 0,
    tag TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (article_id, share_seq),
    FOREIGN KEY (article_id) REFERENCES article(id)
);

/* mysql: */
/* 
CREATE TABLE article (
    id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    tenant_id INT NOT NULL DEFAULT 0,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    views INT DEFAULT 0,
    deleted BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE article_tag (
    article_id INT NOT NULL,
    share_seq INT NOT NULL,
    tag VARCHAR(255) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (article_id, share_seq),
    FOREIGN KEY (article_id) REFERENCES article(id)
); */


/* postgresql: */
/* CREATE TABLE article (
    id SERIAL NOT NULL PRIMARY KEY,
    tenant_id INTEGER NOT NULL DEFAULT 0,
    title TEXT NOT NULL,
    content TEXT,
    views INTEGER DEFAULT 0,
    deleted BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE article_tag (
    article_id INTEGER NOT NULL,
    share_seq INTEGER NOT NULL,
    tag TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (article_id, share_seq),
    FOREIGN KEY (article_id) REFERENCES article(id)
);
 */