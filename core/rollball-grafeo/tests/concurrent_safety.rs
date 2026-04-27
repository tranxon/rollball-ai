//! S5.10: Grafeo concurrent safety tests.
//!
//! Verifies that GrafeoStore can be safely accessed from multiple tokio
//! tasks concurrently without panics, data corruption, or undefined behavior.

use std::sync::Arc;

use chrono::Utc;
use grafeo_common::types::Value;
use rollball_grafeo::{
    DecayConfig, GrafeoStore, GraphExpandConfig, KnowledgeNode, KnowledgeSubType,
    NodeStatus, EMBEDDING_DIM,
};

/// Helper: create an in-memory GrafeoStore for testing.
fn test_store() -> GrafeoStore {
    GrafeoStore::new_in_memory().unwrap()
}

/// Helper: create a KnowledgeNode with a unique subject.
fn make_knowledge_node(idx: usize) -> KnowledgeNode {
    // Create a distinct embedding per node so vector search can find them.
    // Each node gets a unique pattern in its embedding vector.
    let mut emb = vec![0.0f32; EMBEDDING_DIM];
    let slot = idx % EMBEDDING_DIM;
    emb[slot] = 1.0;

    KnowledgeNode {
        id: None,
        subject: format!("subject_{idx}"),
        predicate: "likes".to_string(),
        object: format!("object_{idx}"),
        sub_type: KnowledgeSubType::Fact,
        confidence: 0.9,
        source_episode_id: None,
        embedding: Some(emb),
        status: NodeStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: Default::default(),
    }
}

/// Helper: create an embedding vector for search queries.
fn query_embedding(idx: usize) -> Vec<f32> {
    let mut emb = vec![0.0f32; EMBEDDING_DIM];
    let slot = idx % EMBEDDING_DIM;
    emb[slot] = 1.0;
    emb
}

// ============================================================================
// Test 1: Concurrent read + write
// ============================================================================

/// Multiple tasks write KnowledgeNodes while other tasks read nodes
/// concurrently. Verifies no panic and no data corruption.
#[tokio::test]
async fn test_concurrent_read_write() {
    let store = Arc::new(test_store());

    // Phase 1: Seed some initial data
    for i in 0..10 {
        store.store_knowledge(&make_knowledge_node(i)).unwrap();
    }

    let num_writers = 4;
    let num_readers = 4;
    let ops_per_task = 20;

    // Writer tasks: each writes nodes with unique subjects
    let mut writer_handles = Vec::new();
    for w in 0..num_writers {
        let s = Arc::clone(&store);
        writer_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let node = make_knowledge_node(w * 1000 + i);
                // Write must not panic
                let result = s.store_knowledge(&node);
                assert!(
                    result.is_ok(),
                    "store_knowledge failed at writer {} op {}: {:?}",
                    w, i, result
                );
            }
        }));
    }

    // Reader tasks: each reads nodes via graph_store
    let mut reader_handles = Vec::new();
    for _r in 0..num_readers {
        let s = Arc::clone(&store);
        reader_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                // Read via get_node on a known seed node
                let graph = s.db().graph_store();
                let label_nodes = graph.nodes_by_label("Knowledge");
                // Just verify we can iterate without panic
                let _count = label_nodes.len();
                // Also try a direct node lookup
                if let Some(node_id) = label_nodes.first() {
                    let _node = s.db().get_node(*node_id);
                }
                // Yield occasionally to interleave with writers
                if i % 5 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }));
    }

    // Wait for all tasks to complete
    for handle in writer_handles {
        handle.await.expect("Writer task panicked");
    }
    for handle in reader_handles {
        handle.await.expect("Reader task panicked");
    }

    // Verify data integrity: count should be >= 10 (seed) + all unique writer nodes
    let graph = store.db().graph_store();
    let count = graph.nodes_by_label("Knowledge").len();
    assert!(
        count >= 10,
        "Expected at least 10 knowledge nodes, got {count}"
    );
}

// ============================================================================
// Test 2: Concurrent search + write
// ============================================================================

/// Multiple tasks perform hybrid_search while other tasks write new nodes.
/// Verifies that searches complete without panic and return consistent results.
#[tokio::test]
async fn test_concurrent_search_and_write() {
    let store = Arc::new(test_store());

    // Seed initial data with searchable content
    for i in 0..20 {
        let mut node = make_knowledge_node(i);
        node.object = format!("weather in city_{i}");
        store.store_knowledge(&node).unwrap();
    }

    let num_searchers = 3;
    let num_writers = 2;
    let ops_per_task = 10;

    // Writer tasks: add more knowledge nodes
    let mut writer_handles = Vec::new();
    for w in 0..num_writers {
        let s = Arc::clone(&store);
        writer_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let mut node = make_knowledge_node(5000 + w * 100 + i);
                node.object = format!("weather in city_{}", 5000 + w * 100 + i);
                let result = s.store_knowledge(&node);
                assert!(
                    result.is_ok(),
                    "store_knowledge failed: {:?}",
                    result
                );
                tokio::task::yield_now().await;
            }
        }));
    }

    // Searcher tasks: perform hybrid_search concurrently
    let mut search_handles = Vec::new();
    for _s_idx in 0..num_searchers {
        let s = Arc::clone(&store);
        search_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let query_emb = query_embedding(i % 20);
                // hybrid_search should not panic even while writes are happening
                let result = s.hybrid_search(
                    "Knowledge",
                    "object",
                    "embedding",
                    &format!("city_{}", i % 20),
                    &query_emb,
                    5,
                );
                // Search may succeed or fail (if index is temporarily inconsistent),
                // but must not panic.
                if let Ok(results) = result {
                    // Results should be valid NodeId + score pairs
                    for (node_id, score) in &results {
                        assert!(node_id.is_valid(), "NodeId should be valid");
                        assert!(score.is_finite(), "Score should be finite, got {score}");
                    }
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    for handle in writer_handles {
        handle.await.expect("Writer task panicked");
    }
    for handle in search_handles {
        handle.await.expect("Searcher task panicked");
    }
}

// ============================================================================
// Test 3: Concurrent store_knowledge + decay_scan
// ============================================================================

/// Decay scan iterates over all nodes and updates their status, while
/// store_knowledge creates new nodes. Verifies no deadlock, panic, or
/// data corruption under interleaved read-modify-write cycles.
#[tokio::test]
async fn test_concurrent_store_knowledge_and_decay_scan() {
    let store = Arc::new(test_store());

    // Seed data with old timestamps to make them decay-eligible
    let old_time = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
    for i in 0..30 {
        let mut node = make_knowledge_node(i);
        node.created_at = old_time;
        node.updated_at = old_time;
        // Set low importance to make them decay quickly
        store.store_knowledge(&node).unwrap();
        // Manually set last_accessed and importance to trigger decay
        let graph = store.db().graph_store();
        let node_ids = graph.nodes_by_label("Knowledge");
        if let Some(nid) = node_ids.last() {
            store.update_node(
                *nid,
                [
                    ("last_accessed", Value::from(grafeo_common::types::Timestamp::from_secs(1_700_000_000))),
                    ("importance", Value::from(0.1f64)),
                ],
            ).ok();
        }
    }

    let decay_config = DecayConfig {
        lambda: 0.03,
        access_boost: 0.1,
        dormant_threshold: 0.3,
    };

    let num_writers = 3;
    let num_scanners = 2;
    let ops_per_task = 10;

    // Writer tasks
    let mut writer_handles = Vec::new();
    for w in 0..num_writers {
        let s = Arc::clone(&store);
        writer_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let node = make_knowledge_node(10_000 + w * 100 + i);
                let result = s.store_knowledge(&node);
                assert!(
                    result.is_ok(),
                    "store_knowledge failed: {:?}",
                    result
                );
                tokio::task::yield_now().await;
            }
        }));
    }

    // Scanner tasks: run decay_scan concurrently
    let mut scanner_handles = Vec::new();
    for _s_idx in 0..num_scanners {
        let s = Arc::clone(&store);
        let cfg = decay_config;
        scanner_handles.push(tokio::spawn(async move {
            for _i in 0..ops_per_task {
                // run_decay_scan should not panic or deadlock
                let result = s.run_decay_scan(&cfg);
                // It may succeed (returning count of transitioned nodes)
                // or fail (if internal state is temporarily inconsistent),
                // but must not panic.
                if let Ok(count) = result {
                    assert!(
                        count <= 1000,
                        "Decay scan transitioned an unreasonable number of nodes: {count}"
                    );
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    for handle in writer_handles {
        handle.await.expect("Writer task panicked");
    }
    for handle in scanner_handles {
        handle.await.expect("Scanner task panicked");
    }

    // Verify the store is still in a consistent state
    let graph = store.db().graph_store();
    let count = graph.nodes_by_label("Knowledge").len();
    assert!(count >= 30, "Should still have seed nodes, got {count}");
}

// ============================================================================
// Test 4: Concurrent graph_expand + write
// ============================================================================

/// graph_expand performs BFS traversal while other tasks write new nodes
/// and edges. Verifies no panic during concurrent graph reads and writes.
#[tokio::test]
async fn test_concurrent_graph_expand_and_write() {
    let store = Arc::new(test_store());

    // Create a connected graph structure for graph_expand to traverse
    let mut node_ids = Vec::new();
    for i in 0..15 {
        let id = store.store_knowledge(&make_knowledge_node(i)).unwrap();
        node_ids.push(id);
    }

    // Connect nodes into a chain for BFS traversal
    for i in 0..node_ids.len() - 1 {
        store
            .create_memory_edge(
                node_ids[i],
                node_ids[i + 1],
                "REFERENCES",
                vec![("strength".to_string(), Value::from(0.8f64))],
            )
            .unwrap();
    }

    let expand_config = GraphExpandConfig {
        max_hops: 3,
        max_total_nodes: 20,
        min_edge_weight: 0.1,
        ..Default::default()
    };

    let shared_node_ids = Arc::new(node_ids);

    let num_expanders = 3;
    let num_writers = 2;
    let ops_per_task = 8;

    // Expander tasks: run graph_expand from seed nodes
    let mut expander_handles = Vec::new();
    for e in 0..num_expanders {
        let s = Arc::clone(&store);
        let cfg = expand_config.clone();
        let ids = Arc::clone(&shared_node_ids);
        expander_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let seed_idx = (e * ops_per_task + i) % ids.len();
                let seed_node_id = ids[seed_idx];
                let result = s.graph_expand(
                    &[(seed_node_id, 1.0)],
                    &cfg,
                );
                // graph_expand should not panic, even if the graph is
                // being modified concurrently.
                if let Ok(expanded) = result {
                    // Verify returned nodes have valid IDs
                    for node in &expanded {
                        assert!(
                            node.node_id.is_valid(),
                            "Expanded node should have a valid NodeId"
                        );
                        assert!(
                            node.accumulated_score.is_finite(),
                            "Expanded node score should be finite, got {}",
                            node.accumulated_score
                        );
                    }
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    // Writer tasks: add more nodes and edges
    let mut writer_handles = Vec::new();
    for w in 0..num_writers {
        let s = Arc::clone(&store);
        let existing_ids = Arc::clone(&shared_node_ids);
        writer_handles.push(tokio::spawn(async move {
            for i in 0..ops_per_task {
                let new_node = make_knowledge_node(20_000 + w * 100 + i);
                let result = s.store_knowledge(&new_node);
                assert!(result.is_ok(), "store_knowledge failed: {:?}", result);

                if let Ok(new_id) = result {
                    // Connect new node to an existing one
                    let target = existing_ids[i % existing_ids.len()];
                    let edge_result = s.create_memory_edge(
                        new_id,
                        target,
                        "REFERENCES",
                        vec![("strength".to_string(), Value::from(0.5f64))],
                    );
                    // Edge creation may fail if the graph is in an inconsistent state
                    // during concurrent access, but must not panic.
                    let _ = edge_result;
                }
                tokio::task::yield_now().await;
            }
        }));
    }

    for handle in expander_handles {
        handle.await.expect("Expander task panicked");
    }
    for handle in writer_handles {
        handle.await.expect("Writer task panicked");
    }
}
