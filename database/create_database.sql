create table writes
(
    id         integer
        constraint writes_pk primary key autoincrement,
    created_at timestamp
        default CURRENT_TIMESTAMP not null,
    created_by text
        not null,
    content    text,
    hidden     integer
        default 0 not null,
    source     text
);

create unique index writes_id_uindex
    on writes (id);

create index writes_id_hidden_index
    on writes (id, hidden);

create table reads
(
    id         integer
        constraint reads_pk primary key autoincrement,
    write_id   integer
        not null,
    nickname   text
        not null,
    created_at datetime
        not null default CURRENT_TIMESTAMP
);