use camino::Utf8PathBuf;
use serde_json::Value;
use sqlx::SqlitePool;
use starknet::core::types::FieldElement;

use crate::graphql::schema::build_schema;
use crate::state::sql::{Executable, Sql};
use crate::state::State;

#[allow(dead_code)]
pub async fn run_graphql_query(pool: &SqlitePool, query: &str) -> Value {
    let schema = build_schema(pool).await.unwrap();
    let res = schema.execute(query).await;

    assert!(res.errors.is_empty(), "GraphQL query returned errors: {:?}", res.errors);
    serde_json::to_value(res.data).expect("Failed to serialize GraphQL response")
}

pub async fn entity_fixtures(pool: &SqlitePool) {
    let manifest = dojo_world::manifest::Manifest::load_from_path(
        Utf8PathBuf::from_path_buf("../../examples/ecs/target/dev/manifest.json".into()).unwrap(),
    )
    .unwrap();

    let state = Sql::new(pool.clone(), FieldElement::ZERO).await.unwrap();
    state.load_from_manifest(manifest).await.unwrap();

    // Set entity with one moves component
    let key = vec![FieldElement::ONE];
    let partition = FieldElement::from_hex_be("0x0").unwrap();
    let moves_values = vec![FieldElement::from_hex_be("0xa").unwrap()];
    state.set_entity("Moves".to_string(), partition, key, moves_values.clone()).await.unwrap();

    // Set entity with one position component
    let key = vec![FieldElement::TWO];
    let partition = FieldElement::from_hex_be("0x0").unwrap();
    let position_values = vec![
        FieldElement::from_hex_be("0x2a").unwrap(),
        FieldElement::from_hex_be("0x45").unwrap(),
    ];
    state
        .set_entity("Position".to_string(), partition, key, position_values.clone())
        .await
        .unwrap();

    // Set an entity with both moves and position components
    let key = vec![FieldElement::THREE];
    let partition = FieldElement::from_hex_be("0x0").unwrap();
    state.set_entity("Moves".to_string(), partition, key.clone(), moves_values).await.unwrap();
    state.set_entity("Position".to_string(), partition, key, position_values).await.unwrap();

    state.execute().await.unwrap();
}
