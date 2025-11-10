// -*- coding: utf-8 -*-
//! Vertex-Centric KV-Cache Manager
//! 
//! Efficient caching of graph vertex computations with reuse optimization.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache entry for vertex computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub vertex_id: String,
    pub key: String,
    pub value: Vec<f64>,
    pub timestamp: u64,
    pub access_count: usize,
    pub computation_cost: f64,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
    pub total_misses: usize,
    pub hit_rate: f64,
    pub avg_access_count: f64,
    pub memory_usage_mb: f64,
}

/// Vertex-centric cache with intelligent reuse
pub struct VertexCentricCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    vertex_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    max_entries: usize,
    hits: Arc<RwLock<usize>>,
    misses: Arc<RwLock<usize>>,
}

impl VertexCentricCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            vertex_index: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get cached value for vertex
    pub async fn get(&self, vertex_id: &str, key: &str) -> Option<Vec<f64>> {
        let cache_key = self.make_cache_key(vertex_id, key);
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get_mut(&cache_key) {
            // Update access count
            entry.access_count += 1;
            entry.timestamp = self.current_timestamp();
            
            // Record hit
            let mut hits = self.hits.write().await;
            *hits += 1;
            
            Some(entry.value.clone())
        } else {
            // Record miss
            let mut misses = self.misses.write().await;
            *misses += 1;
            None
        }
    }

    /// Store value in cache
    pub async fn put(
        &self,
        vertex_id: &str,
        key: &str,
        value: Vec<f64>,
        computation_cost: f64,
    ) -> Result<()> {
        let cache_key = self.make_cache_key(vertex_id, key);
        
        // Check if cache is full
        let mut cache = self.cache.write().await;
        if cache.len() >= self.max_entries {
            self.evict_lru(&mut cache).await;
        }
        
        let entry = CacheEntry {
            vertex_id: vertex_id.to_string(),
            key: key.to_string(),
            value,
            timestamp: self.current_timestamp(),
            access_count: 1,
            computation_cost,
        };
        
        cache.insert(cache_key.clone(), entry);
        
        // Update vertex index
        let mut index = self.vertex_index.write().await;
        index.entry(vertex_id.to_string())
            .or_insert_with(Vec::new)
            .push(cache_key);
        
        Ok(())
    }

    /// Get all cached entries for a vertex
    pub async fn get_vertex_entries(&self, vertex_id: &str) -> Vec<CacheEntry> {
        let index = self.vertex_index.read().await;
        let cache = self.cache.read().await;
        
        if let Some(keys) = index.get(vertex_id) {
            keys.iter()
                .filter_map(|k| cache.get(k).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Invalidate cache for vertex
    pub async fn invalidate_vertex(&self, vertex_id: &str) -> Result<()> {
        let mut index = self.vertex_index.write().await;
        let mut cache = self.cache.write().await;
        
        if let Some(keys) = index.remove(vertex_id) {
            for key in keys {
                cache.remove(&key);
            }
        }
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        
        let total_requests = hits + misses;
        let hit_rate = if total_requests > 0 {
            hits as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let avg_access_count = if !cache.is_empty() {
            cache.values()
                .map(|e| e.access_count as f64)
                .sum::<f64>() / cache.len() as f64
        } else {
            0.0
        };
        
        // Estimate memory usage (rough approximation)
        let memory_usage_mb = (cache.len() * 1024) as f64 / (1024.0 * 1024.0);
        
        CacheStats {
            total_entries: cache.len(),
            total_hits: hits,
            total_misses: misses,
            hit_rate,
            avg_access_count,
            memory_usage_mb,
        }
    }

    /// Clear entire cache
    pub async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let mut index = self.vertex_index.write().await;
        let mut hits = self.hits.write().await;
        let mut misses = self.misses.write().await;
        
        cache.clear();
        index.clear();
        *hits = 0;
        *misses = 0;
        
        Ok(())
    }

    fn make_cache_key(&self, vertex_id: &str, key: &str) -> String {
        format!("{}:{}", vertex_id, key)
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    async fn evict_lru(&self, cache: &mut HashMap<String, CacheEntry>) {
        // Find least recently used entry
        if let Some((key_to_remove, _)) = cache.iter()
            .min_by_key(|(_, entry)| entry.timestamp)
        {
            let key_to_remove = key_to_remove.clone();
            cache.remove(&key_to_remove);
        }
    }

    /// Prefetch entries for vertices
    pub async fn prefetch(&self, vertex_ids: &[String]) -> Result<usize> {
        let mut prefetched = 0;
        
        for vertex_id in vertex_ids {
            let entries = self.get_vertex_entries(vertex_id).await;
            prefetched += entries.len();
        }
        
        Ok(prefetched)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = VertexCentricCache::new(100);
        
        cache.put("v1", "key1", vec![1.0, 2.0, 3.0], 0.5).await.unwrap();
        let value = cache.get("v1", "key1").await;
        
        assert!(value.is_some());
        assert_eq!(value.unwrap(), vec![1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = VertexCentricCache::new(100);
        
        cache.put("v1", "key1", vec![1.0], 0.5).await.unwrap();
        cache.get("v1", "key1").await;
        cache.get("v1", "key2").await;
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
