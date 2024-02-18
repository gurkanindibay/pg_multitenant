#[cfg(any(test, feature = "pg_test"))]
use pgrx::pg_schema;
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::pg_test;
    use pgrx::Spi;
    use pgrx::name;
    use pgrx::spi;
    use pgrx::iter::TableIterator;
    use pgrx::Uuid;

    fn create_prereq() {
        let _ = Spi::run(
            "CREATE TABLE public.tenant(
                tenant_id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
                name VARCHAR(255) UNIQUE,
                status VARCHAR(64) CHECK (status IN ('active', 'suspended', 'disabled')),
                tier VARCHAR(64) CHECK (tier IN ('gold', 'silver', 'bronze'))
            )",
        );

        let _ = Spi::run(
            "CREATE TABLE public.tenant_user(
                id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
                tenant_id UUID NOT NULL REFERENCES public.tenant (tenant_id) ON DELETE RESTRICT,
                email VARCHAR(255) NOT NULL UNIQUE,
                given_name VARCHAR(255) NOT NULL CHECK (given_name <> '')
            )",
        );
    }

    fn drop_prereq() {
        let _ = Spi::run("DROP TABLE public.tenant_user");
        let _ = Spi::run("DROP TABLE public.tenant");
    }

    #[pg_test]
    fn test_mark_tenant_column() {
        create_prereq();
        Spi::run("select pgmt.mark_tenant_column('public','tenant','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant");
        Spi::run("select pgmt.mark_tenant_column('public','tenant_user','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant_user");

        let result: Option<i64> = Spi::get_one(
            "SELECT count(*) FROM pg_policy WHERE polname = 'tenant_isolation_policy'",
        )
        .expect("Failed to get policy count");
        assert_eq!(result, Some(2));
        drop_prereq();
    }

    #[pg_test]
    fn test_unmark_tenant_column() {
        create_prereq();
        Spi::run("select pgmt.mark_tenant_column('public','tenant','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant");
        Spi::run("select pgmt.mark_tenant_column('public','tenant_user','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant_user");

        Spi::run("select pgmt.unmark_tenant_column('public','tenant')")
            .expect("Failed to execute unmark_tenant_column on tenant");
        Spi::run("select pgmt.unmark_tenant_column('public','tenant_user')")
            .expect("Failed to execute unmark_tenant_column on tenant_user");

        let result: Option<i64> = Spi::get_one(
            "SELECT count(*) FROM pg_policy WHERE polname = 'tenant_isolation_policy'",
        )
        .expect("Failed to get policy count");
        assert_eq!(result, Some(0));
        drop_prereq();
    }

    #[pg_test]
    fn test_insert_tenant() {
        create_prereq();
        Spi::run("select pgmt.mark_tenant_column('public','tenant','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant");
        Spi::run("select pgmt.mark_tenant_column('public','tenant_user','tenant_id')")
            .expect("Failed to execute mark_tenant_column on tenant_user");

        Spi::run("insert into public.tenant (name, status, tier) values ('acme', 'active', 'gold')")
            .expect("Failed to insert acme");
        Spi::run("insert into public.tenant (name, status, tier) values ('tenant2', 'active', 'silver')")
            .expect("Failed to insert tenant2");

        let acme_tenant_option: Option<String> = Spi::get_one("select tenant_id::text from public.tenant where name = 'acme'")
            .expect("Failed to get acme tenant_id");
        let acme_tenant_id = acme_tenant_option.unwrap();
        Spi::run(format!("set pgmt.tenant_strategy to 'user'").as_str()).expect("'user' strategy failed");

        Spi::run(format!("create user \"{}\"", acme_tenant_id).as_str())
            .expect("Failed to create user");
        Spi::run(format!("grant all on public.tenant to \"{}\"", acme_tenant_id).as_str())
            .expect("Failed to grant select on public.tenant to user");
        Spi::run(format!("grant all on public.tenant_user to \"{}\"", acme_tenant_id).as_str())
            .expect("Failed to grant select on public.tenant_user to user");
        

        Spi::run("insert into public.tenant_user (tenant_id, email, given_name) 
                        select tenant_id, 'user' || tenant_id || '@xx.com', 'acme user' || tenant_id 
                        from public.tenant where name = 'acme'")
            .expect("Failed to insert user acme");

        Spi::run("insert into public.tenant_user (tenant_id, email, given_name) 
            select tenant_id, 'user' || tenant_id || '@xx.com', 'tenant2 user' || tenant_id 
            from public.tenant where name = 'tenant2'")
            .expect("Failed to insert user acme user");

        Spi::run("insert into public.tenant_user (tenant_id, email, given_name) 
            select tenant_id, 'user2' || tenant_id || '@xx.com', 'tenant2 user' || tenant_id 
            from public.tenant where name = 'tenant2'")
            .expect("Failed to insert user tenant2");

        Spi::run("insert into public.tenant_user (tenant_id, email, given_name) 
            select tenant_id, 'user2' || tenant_id || '@xx.com', 'acme user' || tenant_id 
            from public.tenant where name = 'acme'")
            .expect("Failed to insert user acme user2");

        Spi::run("insert into public.tenant_user (tenant_id, email, given_name) 
            select tenant_id, 'user3' || tenant_id || '@xx.com', 'acme user' || tenant_id 
            from public.tenant where name = 'acme'")
            .expect("Failed to insert user acme user2");

        Spi::run(format!("GRANT \"{}\" to CURRENT_USER", acme_tenant_id).as_str()).expect("role grant operation failed");
        let set_role_command = format!("set role \"{}\"", acme_tenant_id);
        println!("set_role_command: {}", set_role_command);
        Spi::run(set_role_command.as_str()).expect("Failed to create user");
        println!("set role command executed");
        let result: Option<i64> = Spi::get_one("select count(*) from public.tenant_user")
            .expect("Failed to get tenant_user count");
        println!("select executed");
        assert_eq!(result, Some(3));
        let return_value = get_tenants().unwrap();
        for row in return_value {
            let tenant_id = row.0;
            match tenant_id {
                Ok(id) => println!("tenant_id: {:?}", format!("{}", id.unwrap())),
                Err(e) => println!("Failed to get tenant_id: {:?}", e),
            }
        }

        Spi::run("reset role").expect("Failed to reset role");
        drop_prereq();

    }

    fn get_tenants() -> Result<
    TableIterator<
        'static,
        (
            name!(tenant_id, Result<Option<Uuid>, pgrx::spi::Error>),
            name!(email, Result<Option<String>, pgrx::spi::Error>),
        ),
    >,
    spi::Error,
> {

        let query = "SELECT tenant_id, email FROM public.tenant_user";

        Spi::connect(|client| {
            Ok(client
                .select(query, None, None)?
                .map(|row| (row["tenant_id"].value(), row["email"].value()))
                .collect::<Vec<_>>())
        })
        .map(TableIterator::new)
    }

}
