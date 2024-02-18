# PG MULTITENANT

**pg-multitenant** is a PostgreSQL extension designed to facilitate multi-tenant database usage through the Shared Database Shared Schema approach. For more in-depth information on the multi-tenancy approach, please refer to the [Multi-tenancy](https://www.citusdata.com/blog/2023/05/09/evolving-django-multitenant-to-build-scalable-saas-apps-on-postgres-and-citus/) article on the Citus Blog Post.

This extension leverages PostgreSQL's row-level security feature to filter data for a specific tenant. It provides a set of functions to manage multi-tenant databases. A foundational understanding of the extension can be found in the following blog post: [Multi-tenant Data Isolation with PostgreSQL Row-Level Security](https://aws.amazon.com/tr/blogs/database/multi-tenant-data-isolation-with-postgresql-row-level-security/).

## Installation

As of now, the extension is not available in binary format, so you need to build it from source using the following steps:

1. **Install Rust**

    ```bash
    (curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | bash -s -- -y) && \
    echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
    ```

2. **Install pgrx extension**

    ```bash
    cargo install pgrx
    cargo pgrx init --pg16 download
    ```

3. **Clone the repository**

    ```bash
    https://github.com/gurkanindibay/pg_multitenant
    ```

4. **Build and run the extension**

    ```bash
    cargo pgrx run pg16
    ```

5. **Install the extension**

    ```sql
    CREATE EXTENSION pg_multitenant;
    ```

## Usage

The extension provides a set of functions to manage multi-tenant databases. The following are the list of User-Defined Functions (UDFs) available:

1. **mark_tenant_column(schema_name text, table_name text, column_name text):** 
    This function marks a column as a tenant column. The tenant column is used to identify the tenant for a given row in a table. The tenant column is used to filter the data for a given tenant.

2. **unmark_tenant_column(schema_name text, table_name text, column_name text):**
    This function unmarks a column as a tenant column. That means the tenant column is no longer used to filter the data for a given tenant.

An example scenario to use the extension exists in the [`scripts/test.sql`](scripts/test.sql) file.

