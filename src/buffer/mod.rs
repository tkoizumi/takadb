pub mod buffer_pool_manager;
pub mod lru_k_replacer;

pub enum AccessType {
    Lookup,
    Scan,
    Write,
}
