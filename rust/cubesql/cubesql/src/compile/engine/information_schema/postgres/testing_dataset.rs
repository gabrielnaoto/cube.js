use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use datafusion::arrow::array::UInt32Builder;
use datafusion::{
    arrow::{
        array::{Array, StringBuilder},
        datatypes::{DataType, Field, Schema, SchemaRef},
        record_batch::RecordBatch,
    },
    datasource::{datasource::TableProviderFilterPushDown, TableProvider, TableType},
    error::DataFusionError,
    logical_plan::Expr,
    physical_plan::{memory::MemoryExec, ExecutionPlan},
};

struct InfoSchemaTestingDatasetProviderBuilder {
    start: usize,
    capacity: usize,
    id: UInt32Builder,
    random_str: StringBuilder,
}

impl InfoSchemaTestingDatasetProviderBuilder {
    fn new(start: usize, capacity: usize) -> Self {
        Self {
            start,
            capacity,
            id: UInt32Builder::new(capacity),
            random_str: StringBuilder::new(capacity),
        }
    }

    fn finish(mut self) -> Vec<Arc<dyn Array>> {
        for i in self.start..(self.start + self.capacity) {
            self.id.append_value(i as u32).unwrap();
            self.random_str.append_value("test".to_string()).unwrap();
        }

        let mut columns: Vec<Arc<dyn Array>> = vec![];
        columns.push(Arc::new(self.id.finish()));
        columns.push(Arc::new(self.random_str.finish()));

        columns
    }
}

pub struct InfoSchemaTestingDatasetProvider {}

impl InfoSchemaTestingDatasetProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TableProvider for InfoSchemaTestingDatasetProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt32, false),
            Field::new("random_str", DataType::Utf8, false),
        ]))
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        projection: &Option<Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        let p1 = InfoSchemaTestingDatasetProviderBuilder::new(0, 1000);

        let p2 = InfoSchemaTestingDatasetProviderBuilder::new(1000, 1000);

        let p3 = InfoSchemaTestingDatasetProviderBuilder::new(2000, 1000);

        Ok(Arc::new(MemoryExec::try_new(
            &[vec![
                RecordBatch::try_new(self.schema(), p1.finish().to_vec())?,
                RecordBatch::try_new(self.schema(), p2.finish().to_vec())?,
                RecordBatch::try_new(self.schema(), p3.finish().to_vec())?,
            ]],
            self.schema(),
            projection.clone(),
        )?))
    }

    fn supports_filter_pushdown(
        &self,
        _filter: &Expr,
    ) -> Result<TableProviderFilterPushDown, DataFusionError> {
        Ok(TableProviderFilterPushDown::Unsupported)
    }
}
