use crate::config::{AppConfig, CollectionConfig};
use crate::models::GetItemsParams;
use crate::storage::{Storage, store::FeaturesWithCount};
use async_trait::async_trait;
use axum::http::StatusCode;
use geojson::Feature;
use serde_json::Value;
use sqlx::{PgPool, Row, postgres::PgRow};
use std::sync::Arc;

struct FeatureQueryParts<'a> {
    where_sql: String,
    placeholder_count: usize,
    params: &'a GetItemsParams,
    collection: &'a CollectionConfig,
}

impl<'a> FeatureQueryParts<'a> {
    fn new(collection: &'a CollectionConfig, params: &'a GetItemsParams) -> Self {
        let mut where_clauses = Vec::new();
        let mut placeholder_count = 1;

        if let Some(bbox) = &params.bbox {
            if bbox.len() == 4 {
                where_clauses.push(format!(
                    "ST_Intersects({}, ST_MakeEnvelope(${}, ${}, ${}, ${}, 4326))",
                    collection.geometry_column,
                    placeholder_count,
                    placeholder_count + 1,
                    placeholder_count + 2,
                    placeholder_count + 3
                ));
                placeholder_count += 4;
            }
        }

        where_clauses.push(format!("{} > ${}", collection.id_column, placeholder_count));

        let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

        Self {
            where_sql,
            placeholder_count,
            params,
            collection,
        }
    }
}

fn get_properties_columns_sql(collection: &CollectionConfig) -> String {
    collection
        .properties
        .iter()
        .map(|p| format!("'{}', {}", p, p))
        .collect::<Vec<_>>()
        .join(", ")
}

fn build_single_feature_sql(collection: &CollectionConfig) -> String {
    format!(
        "SELECT '{}' as type, ST_AsGeoJSON({})::jsonb as geometry, json_build_object({}) as properties, {} as id from {} WHERE {} = $1",
        "Feature",
        collection.geometry_column,
        get_properties_columns_sql(collection),
        collection.id_column,
        collection.table,
        collection.id_column
    )
}

fn build_feature_list_sql(
    collection: &CollectionConfig,
    query_parts: &FeatureQueryParts<'_>,
) -> String {
    format!(
        "SELECT '{}' as type, ST_AsGeoJSON({})::jsonb as geometry, json_build_object({}) as properties, {} as id from {} {} order by {} LIMIT ${}",
        "Feature",
        collection.geometry_column,
        get_properties_columns_sql(collection),
        collection.id_column,
        collection.table,
        query_parts.where_sql,
        collection.id_column,
        query_parts.placeholder_count + 1
    )
}

fn build_count_sql(collection: &CollectionConfig, query_parts: &FeatureQueryParts<'_>) -> String {
    format!(
        "SELECT count(*) from {} {}",
        collection.table, query_parts.where_sql
    )
}

pub struct Postgis {
    pool: PgPool,
    config: Arc<AppConfig>,
}

impl Postgis {
    pub fn new(pool: PgPool, config: Arc<AppConfig>) -> Self {
        Self { pool, config }
    }

    fn row_to_feature(&self, row: &PgRow) -> Result<Feature, (StatusCode, String)> {
        let geometry: Value = row.get("geometry");
        let properties: Value = row.get("properties");
        let id: i32 = row.get("id");

        let feature = geojson::Feature {
            bbox: None,
            geometry: Some(
                geojson::Geometry::from_json_value(geometry)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            ),
            id: Some(geojson::feature::Id::Number(id.into())),
            properties: Some(
                serde_json::from_value(properties)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
            ),
            foreign_members: None,
        };
        Ok(feature)
    }

    fn get_collection(
        &self,
        collection_id: &str,
    ) -> Result<&CollectionConfig, (StatusCode, String)> {
        self.config.collections.get(collection_id).ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Collection {} not found", collection_id),
            )
        })
    }

    async fn fetch_total_count(
        &self,
        query_parts: &FeatureQueryParts<'_>,
    ) -> Result<i64, (StatusCode, String)> {
        let count_sql = build_count_sql(query_parts.collection, query_parts);
        let mut count_query = sqlx::query_scalar(&count_sql);

        if let Some(bbox) = &query_parts.params.bbox {
            if bbox.len() == 4 {
                count_query = count_query
                    .bind(bbox[0])
                    .bind(bbox[1])
                    .bind(bbox[2])
                    .bind(bbox[3]);
            }
        }
        count_query = count_query.bind(query_parts.params.offset.unwrap_or(0) as i64);

        count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }

    async fn fetch_feature_list(
        &self,
        query_parts: &FeatureQueryParts<'_>,
    ) -> Result<Vec<Feature>, (StatusCode, String)> {
        let features_sql = build_feature_list_sql(query_parts.collection, query_parts);
        let mut features_query = sqlx::query(&features_sql);

        if let Some(bbox) = &query_parts.params.bbox {
            if bbox.len() == 4 {
                features_query = features_query
                    .bind(bbox[0])
                    .bind(bbox[1])
                    .bind(bbox[2])
                    .bind(bbox[3]);
            }
        }
        features_query = features_query.bind(query_parts.params.offset.unwrap_or(0) as i64);
        features_query = features_query.bind(query_parts.params.limit.unwrap_or(10) as i64);

        let rows = features_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        rows.iter()
            .map(|row| self.row_to_feature(row))
            .collect::<Result<Vec<_>, _>>()
    }
}

#[async_trait]
impl Storage for Postgis {
    async fn get_features(
        &self,
        collection_id: &str,
        params: &GetItemsParams,
    ) -> Result<FeaturesWithCount, (StatusCode, String)> {
        let collection = self.get_collection(collection_id)?;

        let items_params_for_count = GetItemsParams {
            limit: None,
            offset: None,
            bbox: params.bbox.clone(),
        };
        let query_parts_for_count = FeatureQueryParts::new(collection, &items_params_for_count);
        let total_count = self.fetch_total_count(&query_parts_for_count).await?;

        let query_parts = FeatureQueryParts::new(collection, params);
        let features = self.fetch_feature_list(&query_parts).await?;

        let number_returned = features.len() as u64;

        Ok(FeaturesWithCount::new(
            features,
            total_count as u64,
            number_returned,
        ))
    }

    async fn get_feature(
        &self,
        collection_id: &str,
        id: &str,
    ) -> Result<geojson::Feature, (StatusCode, String)> {
        let collection = self.get_collection(collection_id)?;

        let feature_id: i32 = id
            .parse()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid feature ID".to_string()))?;

        let feature_sql = build_single_feature_sql(collection);

        let row = sqlx::query(&feature_sql)
            .bind(feature_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        self.row_to_feature(&row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CollectionConfig;
    use crate::models::GetItemsParams;

    fn get_test_collection() -> CollectionConfig {
        CollectionConfig {
            table: "naturalearth_lowres".to_string(),
            id_column: "ogc_fid".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            properties: vec!["name".to_string(), "pop_est".to_string()],
        }
    }

    #[test]
    fn test_feature_query_parts_new_no_bbox() {
        let collection = get_test_collection();
        let params = GetItemsParams {
            limit: Some(10),
            offset: Some(0),
            bbox: None,
        };
        let query_parts = FeatureQueryParts::new(&collection, &params);

        assert_eq!(query_parts.where_sql, "WHERE ogc_fid > $1");
        assert_eq!(query_parts.placeholder_count, 1);
    }

    #[test]
    fn test_feature_query_parts_new_with_bbox() {
        let collection = get_test_collection();
        let params_with_bbox = GetItemsParams {
            limit: Some(10),
            offset: Some(0),
            bbox: Some(vec![0.0, 0.0, 10.0, 10.0]),
        };
        let query_parts_with_bbox = FeatureQueryParts::new(&collection, &params_with_bbox);

        assert_eq!(
            query_parts_with_bbox.where_sql,
            "WHERE ST_Intersects(wkb_geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326)) AND ogc_fid > $5"
        );
        assert_eq!(query_parts_with_bbox.placeholder_count, 5);
    }

    #[test]
    fn test_get_properties_columns_sql() {
        let collection = get_test_collection();
        let sql = get_properties_columns_sql(&collection);
        assert_eq!(sql, "'name', name, 'pop_est', pop_est");
    }

    #[test]
    fn test_build_single_feature_sql() {
        let collection = get_test_collection();
        let sql = build_single_feature_sql(&collection);
        let expected_sql = "SELECT 'Feature' as type, ST_AsGeoJSON(wkb_geometry)::jsonb as geometry, json_build_object('name', name, 'pop_est', pop_est) as properties, ogc_fid as id from naturalearth_lowres WHERE ogc_fid = $1";
        assert_eq!(sql, expected_sql);
    }

    #[test]
    fn test_build_feature_list_sql() {
        let collection = get_test_collection();
        let params = GetItemsParams {
            limit: Some(10),
            offset: Some(0),
            bbox: None,
        };
        let query_parts = FeatureQueryParts::new(&collection, &params);
        let sql = build_feature_list_sql(&collection, &query_parts);
        let expected_sql = "SELECT 'Feature' as type, ST_AsGeoJSON(wkb_geometry)::jsonb as geometry, json_build_object('name', name, 'pop_est', pop_est) as properties, ogc_fid as id from naturalearth_lowres WHERE ogc_fid > $1 order by ogc_fid LIMIT $2";
        assert_eq!(sql, expected_sql);
    }

    #[test]
    fn test_build_feature_list_sql_with_bbox() {
        let collection = get_test_collection();
        let params = GetItemsParams {
            limit: Some(10),
            offset: Some(0),
            bbox: Some(vec![0.0, 0.0, 10.0, 10.0]),
        };
        let query_parts = FeatureQueryParts::new(&collection, &params);
        let sql = build_feature_list_sql(&collection, &query_parts);
        let expected_sql = "SELECT 'Feature' as type, ST_AsGeoJSON(wkb_geometry)::jsonb as geometry, json_build_object('name', name, 'pop_est', pop_est) as properties, ogc_fid as id from naturalearth_lowres WHERE ST_Intersects(wkb_geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326)) AND ogc_fid > $5 order by ogc_fid LIMIT $6";
        assert_eq!(sql, expected_sql);
    }

    #[test]
    fn test_build_count_sql() {
        let collection = get_test_collection();
        let params = GetItemsParams {
            limit: Some(10),
            offset: Some(0),
            bbox: None,
        };
        let query_parts = FeatureQueryParts::new(&collection, &params);
        let sql = build_count_sql(&collection, &query_parts);
        let expected_sql = "SELECT count(*) from naturalearth_lowres WHERE ogc_fid > $1";
        assert_eq!(sql, expected_sql);
    }
}
