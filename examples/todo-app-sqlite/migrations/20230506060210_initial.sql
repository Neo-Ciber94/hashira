CREATE TABLE todos (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    description TEXT
);

INSERT INTO todos VALUES (1, 'Buy groceries', 0, 'Eggs, milk, bread');
INSERT INTO todos VALUES (2, 'Do laundry', 0, 'Wash clothes and fold');
INSERT INTO todos VALUES (3, 'Finish project', 0, 'Code new features and write documentation');
INSERT INTO todos VALUES (4, 'Call Mom', 0, 'Catch up and check on health');
INSERT INTO todos VALUES (5, 'Go for a run', 0, 'Run 5 miles and stretch');