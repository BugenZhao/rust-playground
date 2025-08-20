use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

/// A set of features that are available based on the license.
///
/// To define a new feature, add a new entry in the macro [`for_all_features`].
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum Feature3 {
    /// A dummy feature that's only available on paid tier for testing purposes.
    TestPaid,
    /// Query historical data within the retention period.
    TimeTravel,
    /// Use Schema Registry from AWS Glue rather than Confluent.
    GlueSchemaRegistry,
    /// Delivering data to SnowFlake.
    SnowflakeSink,
    /// Delivering data to DynamoDb.
    DynamoDbSink,
    /// Delivering data to OpenSearch.
    OpenSearchSink,
    /// Delivering data to BigQuery.
    BigQuerySink,
    /// Delivering data to Shared tree on clickhouse cloud
    ClickHouseSharedEngine,
    /// Secret management.
    SecretManagement,
    /// Sink data from RisingWave to SQL Server.
    SqlServerSink,
    /// CDC source connector for Sql Server.
    SqlServerCdcSource,
    /// Auto replicate upstream DDL to CDC Table.
    CdcAutoSchemaChange,
    /// Delivering data to Iceberg with Glue catalog.
    IcebergSinkWithGlue,
    /// Disk cache and refilling to boost performance and reduce object store access cost.
    ElasticDiskCache,
    /// Resource group to isolate workload and failure.
    ResourceGroup,
    /// Failure isolation between databases.
    DatabaseFailureIsolation,
    /// Auto iceberg compaction.
    IcebergCompaction,
}

typify::import_types!(schema = "src/bin/typify_schema.json");

fn main() {
    let schema = schema_for!(Feature3);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
