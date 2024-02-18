
create schema pgmt;
create table pgmt.table_tenant_column (
    id serial primary key,
    table_oid int not null,
    column_name text not null,
    UNIQUE (table_oid, column_name)
);
