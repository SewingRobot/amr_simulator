/// Redis cache for frequently accessed map data.
///
/// Placeholder for future implementation using the `fred` crate.
///
/// Planned caching strategies:
/// - Map metadata: cache with TTL on read, invalidate on write
/// - Roadmap graph: cache full graph for pathfinding, invalidate on node/edge changes
/// - Tile metadata: cache Potree hierarchy info
/// - Processing status: cache with short TTL for polling
///
/// # Example usage (future):
/// ```ignore
/// let cache = MapCache::new(redis_client).await?;
/// cache.get_map(map_id).await?;
/// cache.invalidate_map(map_id).await?;
/// ```
pub struct MapCache {
    // TODO: Add fred::clients::RedisClient when Redis integration is implemented
}

impl MapCache {
    /// Create a new cache instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MapCache {
    fn default() -> Self {
        Self::new()
    }
}
