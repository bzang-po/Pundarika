-- Add migration script here
create table if not exists texts (
    id integer primary key
    , name text not null
    , "order" integer not null
    , created_at text not null default (datetime('now'))
)
;

create view if not exists texts_view as
select
    id
    , name
    , "order"
    , created_at
from
    texts
;

create trigger insert_texts_with_auto_increment_order
instead of insert on texts_view
begin
    insert into texts (name, "order")
    values (
        new.name
        , coalesce((select max("order") from texts), 0) + 1
    )
    ;
end
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