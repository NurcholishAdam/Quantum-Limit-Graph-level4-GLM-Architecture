// -*- coding: utf-8 -*-
//! Streaming Inference + Parallel Graph Access
//! 
//! Real-time streaming of inference results with concurrent graph operations.

use crate::error::Result;
use crate::level4::agents::{GLMReasoning, VertexCentricCache, QueryType};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use std::sync::Arc;

/// Stream chunk with partial results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub chunk_id: usize,
    pub content: String,
    pub is_final: bool,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub timestamp_ms: u64,
    pub graph_nodes_accessed: Vec<String>,
    pub cache_hits: usize,
    pub confidence: f64,
}

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub chunk_size: usize,
    pub chunk_delay_ms: u64,
    pub enable_parallel_graph: bool,
    pub max_concurrent_ops: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            chunk_size: 50,
            chunk_delay_ms: 100,
            enable_parallel_graph: true,
            max_concurrent_ops: 4,
        }
    }
}

/// Streaming inference engine
pub struct StreamingInference {
    config: StreamConfig,
    reasoning: Arc<GLMReasoning>,
    cache: Arc<VertexCentricCache>,
}

impl StreamingInference {
    pub fn new(
        config: StreamConfig,
        reasoning: Arc<GLMReasoning>,
        cache: Arc<VertexCentricCache>,
    ) -> Self {
        Self {
            config,
            reasoning,
            cache,
        }
    }

    /// Stream inference results in real-time
    pub async fn stream_inference(
        &self,
        query: &str,
        query_type: QueryType,
    ) -> Result<mpsc::Receiver<StreamChunk>> {
        let (tx, rx) = mpsc::channel(100);
        
        let query = query.to_string();
        let reasoning = self.reasoning.clone();
        let cache = self.cache.clone();
        let config = self.config.clone();
        
        // Spawn streaming task
        tokio::spawn(async move {
            if let Err(e) = Self::stream_task(
                tx,
                query,
                query_type,
                reasoning,
                cache,
                config,
            ).await {
                tracing::error!("Streaming error: {:?}", e);
            }
        });
        
        Ok(rx)
    }

    async fn stream_task(
        tx: mpsc::Sender<StreamChunk>,
        query: String,
        query_type: QueryType,
        reasoning: Arc<GLMReasoning>,
        cache: Arc<VertexCentricCache>,
        config: StreamConfig,
    ) -> Result<()> {
        // Execute reasoning
        let chain = reasoning.reason(&query, query_type).await?;
        
        // Stream results in chunks
        let full_answer = chain.final_answer;
        let chunks: Vec<&str> = full_answer
            .as_bytes()
            .chunks(config.chunk_size)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();
        
        let mut interval = interval(Duration::from_millis(config.chunk_delay_ms));
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            interval.tick().await;
            
            // Parallel graph access
            let graph_nodes = if config.enable_parallel_graph {
                Self::parallel_graph_access(&cache, i).await?
            } else {
                vec![]
            };
            
            let chunk = StreamChunk {
                chunk_id: i,
                content: chunk_content.to_string(),
                is_final: i == chunks.len() - 1,
                metadata: ChunkMetadata {
                    timestamp_ms: Self::current_timestamp_ms(),
                    graph_nodes_accessed: graph_nodes,
                    cache_hits: i % 3, // Simulated
                    confidence: 0.85 + (i as f64 * 0.01),
                },
            };
            
            if tx.send(chunk).await.is_err() {
                break; // Receiver dropped
            }
        }
        
        Ok(())
    }

    async fn parallel_graph_access(
        cache: &Arc<VertexCentricCache>,
        chunk_id: usize,
    ) -> Result<Vec<String>> {
        // Simulate parallel graph access
        let vertex_ids: Vec<String> = (0..4)
            .map(|i| format!("vertex_{}_{}", chunk_id, i))
            .collect();
        
        // Parallel cache lookups
        let mut handles = vec![];
        
        for vertex_id in &vertex_ids {
            let cache = cache.clone();
            let vertex_id = vertex_id.clone();
            
            let handle = tokio::spawn(async move {
                cache.get(&vertex_id, "embedding").await
            });
            
            handles.push(handle);
        }
        
        // Wait for all lookups
        for handle in handles {
            let _ = handle.await;
        }
        
        Ok(vertex_ids)
    }

    fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Stream multiple queries in parallel
    pub async fn stream_batch(
        &self,
        queries: Vec<(String, QueryType)>,
    ) -> Result<Vec<mpsc::Receiver<StreamChunk>>> {
        let mut receivers = Vec::new();
        
        for (query, query_type) in queries {
            let rx = self.stream_inference(&query, query_type).await?;
            receivers.push(rx);
        }
        
        Ok(receivers)
    }

    /// Collect full stream into single result
    pub async fn collect_stream(
        mut rx: mpsc::Receiver<StreamChunk>,
    ) -> Result<String> {
        let mut full_content = String::new();
        
        while let Some(chunk) = rx.recv().await {
            full_content.push_str(&chunk.content);
            
            if chunk.is_final {
                break;
            }
        }
        
        Ok(full_content)
    }

    /// Get streaming statistics
    pub async fn get_stream_stats(
        mut rx: mpsc::Receiver<StreamChunk>,
    ) -> Result<StreamStats> {
        let mut total_chunks = 0;
        let mut total_graph_nodes = 0;
        let mut total_cache_hits = 0;
        let mut start_time = 0u64;
        let mut end_time = 0u64;
        
        while let Some(chunk) = rx.recv().await {
            if total_chunks == 0 {
                start_time = chunk.metadata.timestamp_ms;
            }
            
            total_chunks += 1;
            total_graph_nodes += chunk.metadata.graph_nodes_accessed.len();
            total_cache_hits += chunk.metadata.cache_hits;
            end_time = chunk.metadata.timestamp_ms;
            
            if chunk.is_final {
                break;
            }
        }
        
        Ok(StreamStats {
            total_chunks,
            total_graph_nodes,
            total_cache_hits,
            duration_ms: end_time - start_time,
            avg_chunk_time_ms: if total_chunks > 0 {
                (end_time - start_time) / total_chunks as u64
            } else {
                0
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub total_chunks: usize,
    pub total_graph_nodes: usize,
    pub total_cache_hits: usize,
    pub duration_ms: u64,
    pub avg_chunk_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_inference() {
        let reasoning = Arc::new(GLMReasoning::new(10));
        let cache = Arc::new(VertexCentricCache::new(1000));
        
        let streaming = StreamingInference::new(
            StreamConfig::default(),
            reasoning,
            cache,
        );
        
        let mut rx = streaming.stream_inference(
            "Test query",
            QueryType::Reasoning,
        ).await.unwrap();
        
        let mut chunk_count = 0;
        while let Some(chunk) = rx.recv().await {
            chunk_count += 1;
            if chunk.is_final {
                break;
            }
        }
        
        assert!(chunk_count > 0);
    }

    #[tokio::test]
    async fn test_collect_stream() {
        let reasoning = Arc::new(GLMReasoning::new(10));
        let cache = Arc::new(VertexCentricCache::new(1000));
        
        let streaming = StreamingInference::new(
            StreamConfig::default(),
            reasoning,
            cache,
        );
        
        let rx = streaming.stream_inference(
            "Test query",
            QueryType::Factual,
        ).await.unwrap();
        
        let result = StreamingInference::collect_stream(rx).await.unwrap();
        assert!(!result.is_empty());
    }
}
