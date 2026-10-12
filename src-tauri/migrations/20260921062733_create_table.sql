-- Add migration script here
create table if not exists texts (
    id integer primary key
    , name text not null
    , "order" integer not null default 0
    , created_at text not null default (datetime('now'))
)
;

create virtual table if not exists texts_fts using fts5(
    name
    , content='texts'
    , tokenize='trigram'
)
;

create trigger if not exists texts_ai after insert on texts begin
    insert into texts_fts(rowid, name) values (new.id, new.name);
end
;

create trigger if not exists texts_ad after delete on texts begin
    insert into texts_fts(texts_fts, rowid, name) values('delete', old.id, old.name);
end
;

create trigger if not exists texts_au after update on texts begin
    insert into texts_fts(texts_fts, rowid, name) values('delete', old.id, old.name);
    insert into texts_fts(rowid, name) values (new.id, new.name);
end
;