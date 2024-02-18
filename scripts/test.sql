create table public.tenant(
    tenant_id UUID DEFAULT gen_random_uuid()  PRIMARY KEY,
    name VARCHAR(255) UNIQUE,
    status VARCHAR(64) CHECK (status IN ('active', 'suspended', 'disabled')),
    tier VARCHAR(64) CHECK (tier IN ('gold', 'silver', 'bronze'))
);

create table public.tenant_user(
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES public.tenant (tenant_id) ON DELETE RESTRICT,
    email VARCHAR(255) NOT NULL UNIQUE,
    given_name VARCHAR(255) NOT NULL CHECK (given_name <> '')
)


select * from public.tenant;

select * from tenant_user;

insert into public.tenant (name, status, tier) values ('acme', 'active', 'gold');
insert into public.tenant (name, status, tier) values ('tenant2', 'active', 'silver');


insert into public.tenant_user (tenant_id, email, given_name) 
select tenant_id, 'user' || tenant_id || '@xx.com', 'acme user' || tenant_id from public.tenant where name = 'acme';

insert into public.tenant_user (tenant_id, email, given_name)
select tenant_id, 'user' || tenant_id || '@xx.com', 'tenant2 user' || tenant_id from public.tenant where name = 'tenant2';

insert into public.tenant_user (tenant_id, email, given_name) 
select tenant_id, 'user2' || tenant_id || '@xx.com', 'acme user' || tenant_id from public.tenant where name = 'acme';

set pgmt.tenant_strategy to 'user';

show pgmt.tenant_strategy;

select pgmt.mark_tenant_column('public', 'tenant_user', 'tenant_id');
select pgmt.mark_tenant_column('public', 'tenant', 'tenant_id');


create role "<tenant_id>";

grant all on table public.tenant to "<tenant_id>";
grant all on table public.tenant_user to "<tenant_id>";

set role "<tenant_id>";
select * from public.tenant;

select * from public.tenant_user;

reset role;

drop TABLE public.tenant_user;
drop TABLE public.tenant;

drop role "<tenant_id>" CASCADE;
