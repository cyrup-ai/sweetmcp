# SurrealDB Graph Relationships Example

This example demonstrates how to use SurrealDB's graph capabilities with SurrealKV engine, following best practices for async Rust by hiding complexity behind synchronous interfaces.

## Entity Definitions

```rust
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use crate::db::{
    DatabaseClient, DatabaseConfig, StorageEngine, Error, Result, Entity,
    connect_database
};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use uuid::Uuid;

// Helper function for timestamps
fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

// Base entity with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaseEntity {
    /// Entity ID
    pub id: Option<String>,
    /// Creation timestamp
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        Self {
            id: None,
            created_at: utc_now(),
            updated_at: utc_now(),
        }
    }
}

// Define a Person entity for our graph
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    #[serde(flatten)]
    base: BaseEntity,
    name: String,
    age: u32,
}

impl Entity for Person {
    fn table_name() -> &'static str {
        "person"
    }

    fn id(&self) -> Option<String> {
        self.base.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.base.id = Some(id);
    }

    fn generate_id() -> String {
        format!("{}:{}", Self::table_name(), Uuid::new_v4())
    }
}

impl Person {
    fn new(name: impl Into<String>, age: u32) -> Self {
        Self {
            base: BaseEntity::new(),
            name: name.into(),
            age,
        }
    }
}

// Domain-specific types for graph operations
struct PersonStream {
    rx: mpsc::Receiver<Result<Person>>,
    _handle: JoinHandle<()>,
}

struct RelationshipStream {
    rx: mpsc::Receiver<Result<(String, Person)>>,
    _handle: JoinHandle<()>,
}

struct PathStream {
    rx: mpsc::Receiver<Result<Vec<String>>>,
    _handle: JoinHandle<()>,
}

struct PeopleStream {
    rx: mpsc::Receiver<Result<Vec<Person>>>,
    _handle: JoinHandle<()>,
}

struct RelationshipPairStream {
    rx: mpsc::Receiver<Result<Vec<(Person, Person)>>>,
    _handle: JoinHandle<()>,
}

// Implementation for the domain-specific types
impl PersonStream {
    async fn get(mut self) -> Result<Person> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl RelationshipStream {
    async fn get(mut self) -> Result<Vec<(String, Person)>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl PathStream {
    async fn get(mut self) -> Result<Vec<String>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl PeopleStream {
    async fn get(mut self) -> Result<Vec<Person>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

impl RelationshipPairStream {
    async fn get(mut self) -> Result<Vec<(Person, Person)>> {
        self.rx.recv().await.unwrap_or_else(|| Err(Error::other("Channel closed unexpectedly")))
    }
}

// Graph operations manager
struct GraphOps {
    client: DatabaseClient,
}

impl GraphOps {
    fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    // Create a person (synchronous interface)
    fn create_person(&self, name: &str, age: u32) -> PersonStream {
        let client = self.client.clone();
        let name = name.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                let mut person = Person::new(name, age);
                let id = Person::generate_id();
                person.set_id(id);

                client.create(Person::table_name(), &person).await
            }.await;

            let _ = tx.send(result).await;
        });

        PersonStream { rx, _handle: handle }
    }

    // Create a relationship (synchronous interface)
    fn create_relationship(
        &self,
        from: &Person,
        relationship_type: &str,
        to: &Person,
        data: Option<serde_json::Value>,
    ) -> RelationshipStream {
        let client = self.client.clone();
        let from_id = from.id().unwrap();
        let to_id = to.id().unwrap();
        let relationship_type = relationship_type.to_string();
        let data = data.clone();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // SurrealDB has a special syntax for creating relationships
                let query = format!(
                    "RELATE {}->{}->{}",
                    from_id, relationship_type, to_id
                );

                let params = match data {
                    Some(data) => serde_json::json!({ "data": data }),
                    None => serde_json::json!({}),
                };

                let _: serde_json::Value = client.query_with_params(&query, params).await?;

                // Return a placeholder since we don't need the actual result
                Ok(Vec::new())
            }.await;

            let _ = tx.send(result).await;
        });

        RelationshipStream { rx, _handle: handle }
    }

    // Get direct relationships (synchronous interface)
    fn get_direct_relationships(&self, person: &Person) -> RelationshipStream {
        let client = self.client.clone();
        let person_id = person.id().unwrap();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Query for all outgoing relationships
                let query = format!(
                    "SELECT ->->person.* AS person, type::string(->edge) AS relationship FROM person:{id}",
                    id = person_id.split(':').nth(1).unwrap_or(&person_id)
                );

                // Execute the query and parse the results
                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut relationships = Vec::new();
                for result in results {
                    if let (Some(relationship), Some(person_obj)) = (
                        result.get("relationship").and_then(|v| v.as_str()),
                        result.get("person")
                    ) {
                        // Convert to Person object
                        if let Ok(related_person) = serde_json::from_value::<Person>(person_obj.clone()) {
                            relationships.push((relationship.to_string(), related_person));
                        }
                    }
                }

                Ok(relationships)
            }.await;

            let _ = tx.send(result).await;
        });

        RelationshipStream { rx, _handle: handle }
    }

    // Get relationships with depth (synchronous interface)
    fn get_relationships_with_depth(&self, person: &Person, depth: u32) -> PeopleStream {
        let client = self.client.clone();
        let person_id = person.id().unwrap();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Query for relationships with a specific depth
                let query = format!(
                    "SELECT ->({depth})->person.* AS people FROM person:{id}",
                    depth = depth,
                    id = person_id.split(':').nth(1).unwrap_or(&person_id)
                );

                // Execute the query and parse the results
                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut people = Vec::new();
                for result in results {
                    if let Some(people_arr) = result.get("people").and_then(|v| v.as_array()) {
                        for person_obj in people_arr {
                            if let Ok(related_person) = serde_json::from_value::<Person>(person_obj.clone()) {
                                people.push(related_person);
                            }
                        }
                    }
                }

                Ok(people)
            }.await;

            let _ = tx.send(result).await;
        });

        PeopleStream { rx, _handle: handle }
    }

    // Get specific relationship types (synchronous interface)
    fn get_specific_relationship(&self, relationship_type: &str) -> RelationshipPairStream {
        let client = self.client.clone();
        let relationship_type = relationship_type.to_string();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Query for specific relationship types
                let query = format!(
                    "SELECT person:in.* AS person1, person:out.* AS person2 FROM {relation_type}",
                    relation_type = relationship_type
                );

                // Execute the query and parse the results
                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut relationships = Vec::new();
                for result in results {
                    if let (Some(person1_obj), Some(person2_obj)) = (
                        result.get("person1"),
                        result.get("person2")
                    ) {
                        // Convert to Person objects
                        if let (Ok(person1), Ok(person2)) = (
                            serde_json::from_value::<Person>(person1_obj.clone()),
                            serde_json::from_value::<Person>(person2_obj.clone())
                        ) {
                            relationships.push((person1, person2));
                        }
                    }
                }

                Ok(relationships)
            }.await;

            let _ = tx.send(result).await;
        });

        RelationshipPairStream { rx, _handle: handle }
    }

    // Find shortest path (synchronous interface)
    fn find_shortest_path(&self, from: &Person, to: &Person) -> PathStream {
        let client = self.client.clone();
        let from_id = from.id().unwrap();
        let to_id = to.id().unwrap();

        let (tx, rx) = mpsc::channel(1);

        let handle = tokio::spawn(async move {
            let result = async {
                // Query for the shortest path
                let query = format!(
                    "SELECT array::flatten(array::transform(->shortest_path->{to_id}, function($v) {{
                        return string::concat($v.in.name, ' ', type::string($v.edge), ' ', $v.out.name);
                    }})) AS path FROM {from_id}",
                    from_id = from_id,
                    to_id = to_id
                );

                // Execute the query and parse the results
                let results: Vec<serde_json::Value> = client.query(&query).await?;

                let mut path = Vec::new();
                if let Some(result) = results.first() {
                    if let Some(path_arr) = result.get("path").and_then(|v| v.as_array()) {
                        for step in path_arr {
                            if let Some(step_str) = step.as_str() {
                                path.push(step_str.to_string());
                            }
                        }
                    }
                }

                Ok(path)
            }.await;

            let _ = tx.send(result).await;
        });

        PathStream { rx, _handle: handle }
    }
}
```

## Main Program (Using Synchronous Interface with Hidden Async)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    println!("SurrealDB Graph Relationships Example\n");

    // Set up the database configuration for SurrealKV
    let config = DatabaseConfig {
        engine: StorageEngine::SurrealKv,
        path: "./.data/graph_db".to_string(),
        namespace: "graph_example".to_string(),
        database: "relationships".to_string(),
        check_migrations: false,
        ..Default::default()
    };

    // Connect to the database
    let client = connect_database(config).await?;

    // Create our graph operations wrapper
    let graph_ops = GraphOps::new(client);

    // Create some people using the synchronous interface
    let alice = graph_ops.create_person("Alice", 30).get().await?;
    let bob = graph_ops.create_person("Bob", 32).get().await?;
    let charlie = graph_ops.create_person("Charlie", 28).get().await?;
    let diana = graph_ops.create_person("Diana", 35).get().await?;
    let eve = graph_ops.create_person("Eve", 29).get().await?;

    println!("Created people: Alice, Bob, Charlie, Diana, Eve");

    // Create relationships using the synchronous interface
    graph_ops.create_relationship(&alice, "KNOWS", &bob, None).get().await?;
    graph_ops.create_relationship(&alice, "KNOWS", &charlie, None).get().await?;
    graph_ops.create_relationship(&bob, "MARRIED_TO", &diana, Some(serde_json::json!({
        "since": 2018
    }))).get().await?;
    graph_ops.create_relationship(&charlie, "WORKS_WITH", &eve, Some(serde_json::json!({
        "company": "SurrealDB Inc",
        "since": 2021
    }))).get().await?;
    graph_ops.create_relationship(&diana, "KNOWS", &eve, None).get().await?;
    graph_ops.create_relationship(&eve, "KNOWS", &alice, None).get().await?;

    println!("Created relationships between people");

    // Find direct relationships for a person
    let direct_relationships = graph_ops.get_direct_relationships(&alice).get().await?;
    println!("\nDirect relationships for Alice:");
    for (relationship, person) in direct_relationships {
        println!("- Alice {} {}", relationship, person.name);
    }

    // Find relationships with depth for a person
    let depth_relationships = graph_ops.get_relationships_with_depth(&alice, 2).get().await?;
    println!("\nRelationships with depth 2 for Alice:");
    for person in depth_relationships {
        println!("- {}", person.name);
    }

    // Find specific relationship types
    let specific_relationships = graph_ops.get_specific_relationship("MARRIED_TO").get().await?;
    println!("\nMarried couples:");
    for (person1, person2) in specific_relationships {
        println!("- {} is married to {}", person1.name, person2.name);
    }

    // Find the shortest path between two people
    let path = graph_ops.find_shortest_path(&alice, &eve).get().await?;
    println!("\nShortest path from Alice to Eve:");
    for step in path {
        println!("- {}", step);
    }

    println!("\nExample completed");
    Ok(())
}
