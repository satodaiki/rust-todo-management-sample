CREATE EXTENSION "uuid-ossp";

CREATE TABLE "todo" (
    id UUID primary key,
    description varchar(100) not null,
    deadline_at timestamp not null
);