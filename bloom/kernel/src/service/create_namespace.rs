use super::{CreateNamespaceInput, Service};
use crate::{consts::BillingPlan, entities::Namespace};
use stdx::{
    chrono::Utc,
    sqlx::{Postgres, Transaction},
    ulid::Ulid,
};

impl Service {
    pub async fn create_namespace<'c>(
        &self,
        tx: &mut Transaction<'c, Postgres>,
        input: CreateNamespaceInput,
    ) -> Result<Namespace, crate::Error> {
        self.validate_namespace(&input.path)?;

        let now = Utc::now();
        let namespace = Namespace {
            id: Ulid::new().into(),
            created_at: now,
            updated_at: now,
            path: input.path,
            r#type: input.namespace_type,
            parent_id: None,
            used_storage: 0,
            plan: BillingPlan::Free,
        };

        {
            // we do this because otherwise we can't use tx in init_namespace below
            let tx: &mut Transaction<'c, Postgres> = tx;
            self.repo.create_namespace(tx, &namespace).await?;
        }

        self.files_service
            .as_ref()
            .expect("kernel.create_namespace: unwrapping files_service")
            .init_namespace(tx, namespace.id)
            .await?;

        Ok(namespace)
    }
}
