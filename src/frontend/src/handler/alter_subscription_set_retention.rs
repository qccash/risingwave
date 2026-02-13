// Copyright 2025 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use pgwire::pg_response::{PgResponse, StatementType};
use risingwave_sqlparser::ast::ObjectName;

use super::{HandlerArgs, RwPgResponse};
use crate::Binder;
use crate::catalog::root_catalog::SchemaPath;
use crate::error::Result;
use crate::handler::util::convert_interval_to_u64_seconds;

pub async fn handle_alter_subscription_set_retention(
    handler_args: HandlerArgs,
    subscription_name: ObjectName,
    retention: String,
) -> Result<RwPgResponse> {
    let session = handler_args.session;
    let db_name = &session.database();
    let (schema_name, real_subscription_name) =
        Binder::resolve_schema_qualified_name(db_name, &subscription_name)?;
    let search_path = session.config().search_path();
    let user_name = &session.user_name();
    let schema_path = SchemaPath::new(schema_name.as_deref(), &search_path, user_name);
    let retention_seconds = convert_interval_to_u64_seconds(&retention)?;

    let subscription_id = {
        let catalog_reader = session.env().catalog_reader().read_guard();
        let (subscription, schema_name) = catalog_reader.get_subscription_by_name(
            db_name,
            schema_path,
            &real_subscription_name,
        )?;
        session.check_privilege_for_drop_alter(schema_name, &**subscription)?;
        if subscription.retention_seconds == retention_seconds {
            return Ok(PgResponse::empty_result(StatementType::ALTER_SUBSCRIPTION));
        }
        subscription.id
    };

    let catalog_writer = session.catalog_writer()?;
    catalog_writer
        .alter_subscription_retention(subscription_id, retention_seconds)
        .await?;

    Ok(PgResponse::empty_result(StatementType::ALTER_SUBSCRIPTION))
}

#[cfg(test)]
mod tests {
    use risingwave_common::catalog::{DEFAULT_DATABASE_NAME, DEFAULT_SCHEMA_NAME};

    use crate::catalog::root_catalog::SchemaPath;
    use crate::test_utils::LocalFrontend;

    #[tokio::test]
    async fn test_alter_subscription_set_retention_to() {
        let frontend = LocalFrontend::new(Default::default()).await;
        frontend.run_sql("create table t1 (v1 int);").await.unwrap();
        frontend
            .run_sql("create subscription sub1 from t1 with(retention = '1H');")
            .await
            .unwrap();
        frontend
            .run_sql("alter subscription sub1 set retention to '2H';")
            .await
            .unwrap();

        let session = frontend.session_ref();
        let catalog_reader = session.env().catalog_reader().read_guard();
        let schema_path = SchemaPath::Name(DEFAULT_SCHEMA_NAME);
        let (subscription, _) = catalog_reader
            .get_subscription_by_name(DEFAULT_DATABASE_NAME, schema_path, "sub1")
            .unwrap();
        assert_eq!(subscription.retention_seconds, 2 * 3600);
    }

    #[tokio::test]
    async fn test_alter_subscription_set_retention_eq() {
        let frontend = LocalFrontend::new(Default::default()).await;
        frontend.run_sql("create table t1 (v1 int);").await.unwrap();
        frontend
            .run_sql("create subscription sub1 from t1 with(retention = '1H');")
            .await
            .unwrap();
        frontend
            .run_sql("alter subscription sub1 set retention = '1D';")
            .await
            .unwrap();

        let session = frontend.session_ref();
        let catalog_reader = session.env().catalog_reader().read_guard();
        let schema_path = SchemaPath::Name(DEFAULT_SCHEMA_NAME);
        let (subscription, _) = catalog_reader
            .get_subscription_by_name(DEFAULT_DATABASE_NAME, schema_path, "sub1")
            .unwrap();
        assert_eq!(subscription.retention_seconds, 24 * 3600);
    }
}
