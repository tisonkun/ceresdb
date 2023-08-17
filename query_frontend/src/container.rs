// Copyright 2023 The CeresDB Authors
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

//! Table container

use std::{borrow::Cow, collections::HashMap};

pub use datafusion::catalog::{ResolvedTableReference, TableReference};
use table_engine::table::TableRef;

use crate::provider::ResolvedTable;

// Rust has poor support of using tuple as map key, so we use a 3 level
// map to store catalog -> schema -> table mapping
type CatalogMap = HashMap<String, SchemaMap>;
type SchemaMap = HashMap<String, TableMap>;
// TODO(chenxiang): change to LRU to evict deleted/migrated tables
type TableMap = HashMap<String, ResolvedTable>;

/// Container to hold table adapters
///
/// Optimized for default catalog and schema
#[derive(Default)]
pub struct TableContainer {
    default_catalog: String,
    default_schema: String,
    default_tables: TableMap,
    other_tables: CatalogMap,
}

impl TableContainer {
    pub fn new(default_catalog: String, default_schema: String) -> Self {
        Self {
            default_catalog,
            default_schema,
            default_tables: Default::default(),
            other_tables: Default::default(),
        }
    }

    /// Catalog num
    pub fn num_catalogs(&self) -> usize {
        if self.other_tables.is_empty() {
            1
        } else {
            self.other_tables.len() + 1
        }
    }

    pub fn get(&self, name: TableReference) -> Option<ResolvedTable> {
        match name {
            TableReference::Bare { table } => self.get_default(table.as_ref()),
            TableReference::Partial { schema, table } => {
                if schema == self.default_schema {
                    self.get_default(table.as_ref())
                } else {
                    self.get_other(&self.default_catalog, schema.as_ref(), table.as_ref())
                }
            }
            TableReference::Full {
                catalog,
                schema,
                table,
            } => {
                if catalog == self.default_catalog && schema == self.default_schema {
                    self.get_default(table.as_ref())
                } else {
                    self.get_other(catalog.as_ref(), schema.as_ref(), table.as_ref())
                }
            }
        }
    }

    fn get_default(&self, table: &str) -> Option<ResolvedTable> {
        self.default_tables.get(table).cloned()
    }

    fn get_other(&self, catalog: &str, schema: &str, table: &str) -> Option<ResolvedTable> {
        self.other_tables
            .get(catalog)
            .and_then(|schemas| schemas.get(schema))
            .and_then(|tables| tables.get(table))
            .cloned()
    }

    pub fn insert(&mut self, name: TableReference, resolved_table: ResolvedTable) {
        match name {
            TableReference::Bare { table } => self.insert_default(table.as_ref(), resolved_table),
            TableReference::Partial { schema, table } => {
                if schema == self.default_schema {
                    self.insert_default(table.as_ref(), resolved_table)
                } else {
                    self.insert_other(
                        self.default_catalog.clone(),
                        schema.to_string(),
                        table.to_string(),
                        resolved_table,
                    )
                }
            }
            TableReference::Full {
                catalog,
                schema,
                table,
            } => {
                if catalog == self.default_catalog && schema == self.default_schema {
                    self.insert_default(table.as_ref(), resolved_table)
                } else {
                    self.insert_other(
                        catalog.to_string(),
                        schema.to_string(),
                        table.to_string(),
                        resolved_table,
                    )
                }
            }
        }
    }

    fn insert_default(&mut self, table: &str, resolved_table: ResolvedTable) {
        self.default_tables
            .insert(table.to_string(), resolved_table);
    }

    fn insert_other(
        &mut self,
        catalog: String,
        schema: String,
        table: String,
        resolved_table: ResolvedTable,
    ) {
        self.other_tables
            .entry(catalog)
            .or_insert_with(HashMap::new)
            .entry(schema)
            .or_insert_with(HashMap::new)
            .insert(table, resolved_table);
    }

    /// Visit all tables
    ///
    /// If f returns error, stop iteration and return the error
    pub fn visit<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(ResolvedTableReference, &TableRef) -> Result<(), E>,
    {
        // Visit default tables first
        for (table, resolved) in &self.default_tables {
            // default_catalog/default_schema can be empty string, but that's
            // ok since we have table under them
            let table_ref = ResolvedTableReference {
                catalog: Cow::from(&self.default_catalog),
                schema: Cow::from(&self.default_schema),
                table: Cow::from(table),
            };
            f(table_ref, &resolved.table)?;
        }

        // Visit other tables
        for (catalog, schemas) in &self.other_tables {
            for (schema, tables) in schemas {
                for (table, resolved) in tables {
                    let table_ref = ResolvedTableReference {
                        catalog: Cow::from(catalog),
                        schema: Cow::from(schema),
                        table: Cow::from(table),
                    };
                    f(table_ref, &resolved.table)?;
                }
            }
        }

        Ok(())
    }
}
