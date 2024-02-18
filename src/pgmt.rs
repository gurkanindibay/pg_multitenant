use lazy_static::lazy_static;
use pgrx::*;
use std::ffi::CStr;

pgrx::pg_module_magic!();

extension_sql_file!("../sql/bootstrap.sql", bootstrap);

lazy_static! {
    static ref TENANT_STRATEGY_GUC: GucSetting<Option<&'static CStr>> = {
        let default_tenant_strategy = GucSetting::<Option<&'static CStr>>::new(Some(unsafe {
            CStr::from_bytes_with_nul_unchecked(b"user\0")
        }));

        GucRegistry::define_string_guc(
            "pgmt.tenant_strategy",
            "Tenant strategy",
            "pgmt.tenant_strategy defines the strategy to use for tenant isolation. Valid values are 'user' and 'value'.",
            &default_tenant_strategy,
            GucContext::Userset,
            GucFlags::default(),
        );

        default_tenant_strategy
    };
    static ref TENANT_VALUE_GUC: GucSetting<Option<&'static CStr>> = {
        let default_tenant_value = GucSetting::<Option<&'static CStr>>::new(Some(unsafe {
            CStr::from_bytes_with_nul_unchecked(b"\0")
        }));

        GucRegistry::define_string_guc(
            "pgmt.tenant_strategy",
            "Tenant strategy",
            "pgmt.tenant_strategy defines the strategy to use for tenant isolation. Valid values are 'user' and 'value'.",
            &default_tenant_value,
            GucContext::Userset,
            GucFlags::default(),
        );

        default_tenant_value
    };
}

#[pg_schema]
pub mod pgmt {
    use super::*;

    #[pg_extern]
    pub fn mark_tenant_column(schema_name: &str, table_name: &str, tenant_column: &str) {
        Spi::get_one::<bool>("SELECT rolsuper FROM pg_roles WHERE rolname = current_user")
            .expect("Only superusers can mark a table for tenant isolation");
        let tenant_strategy = TENANT_STRATEGY_GUC
            .get()
            .expect("Failed to get GUC value")
            .to_string_lossy()
            .into_owned();

        if tenant_strategy.is_empty() {
            panic!("pgmt.tenant_strategy is not set");
        }

        let mut tenant_value: String = String::new();

        if tenant_strategy == "value" {
            tenant_value = TENANT_VALUE_GUC
                .get()
                .expect("Failed to get GUC value")
                .to_string_lossy()
                .into_owned();
            if tenant_value.is_empty() {
                panic!("pgmt.tenant_value is not set");
            }
        }

        let tenant_phrase: String = match tenant_strategy.as_str() {
            "user" => "current_user".to_string(),
            "value" => format!("'{}'", tenant_value),
            _ => panic!("Invalid tenant strategy: {} ", tenant_strategy),
        };
        enable_row_level_security(
            schema_name,
            table_name,
            tenant_column,
            tenant_phrase.as_str(),
        );

        insert_table_tenant_column(schema_name, table_name, tenant_column);
    }

    pub fn enable_row_level_security(
        schema_name: &str,
        table_name: &str,
        tenant_column: &str,
        tenant_value: &str,
    ) {
        let sql = format!(
            "ALTER TABLE {}.{} ENABLE ROW LEVEL SECURITY",
            schema_name, table_name
        );
        Spi::run(&sql).expect("Failed to enable row level security");

        let policy_sql = format!(
            "CREATE POLICY tenant_isolation_policy ON {}.{} USING ({}::TEXT = {})",
            schema_name, table_name, tenant_column, tenant_value
        );
        Spi::run(&policy_sql).expect("Failed to create policy");
    }

    fn insert_table_tenant_column(schema_name: &str, table_name: &str, column_name: &str) {
        let table_oid = get_table_oid(schema_name, table_name).expect("Failed to get table OID");
        let sql = format!(
            "INSERT INTO pgmt.table_tenant_column (table_oid, column_name) VALUES ({},'{}')",
            table_oid, column_name
        );
        Spi::run(&sql).expect("Failed to insert metadata");
    }

    fn get_table_oid(schema_name: &str, table_name: &str) -> Option<u32> {
        let table_oid: Option<u32> = Spi::get_one::<u32>(&format!(
            "SELECT c.oid
            FROM pg_catalog.pg_class c
            INNER JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relname = '{}'
            AND n.nspname = '{}'",
            table_name, schema_name
        ))
        .expect("Failed to get table OID");
        table_oid
    }

    #[pg_extern]
    fn unmark_tenant_column(schema_name: &str, table_name: &str) {
        disable_row_level_security(schema_name, table_name);

        let table_oid= get_table_oid(schema_name, table_name).expect("Failed to get table OID");

        let delete_sql = format!(
            "DELETE FROM pgmt.table_tenant_column WHERE table_oid={}",table_oid
        );
        Spi::run(&delete_sql).expect("Failed to DELETE metadata");
    }

    fn disable_row_level_security(schema_name: &str, table_name: &str) {
        let sql = format!(
            "ALTER TABLE {}.{} DISABLE ROW LEVEL SECURITY",
            schema_name, table_name
        );
        Spi::run(&sql).expect("Failed to disable row level security");

        let policy_sql = format!(
            "DROP POLICY tenant_isolation_policy ON {}.{}",
            schema_name, table_name
        );
        Spi::run(&policy_sql).expect("Failed to drop policy");

    }
}

