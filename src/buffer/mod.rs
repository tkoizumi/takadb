pub mod lru_k_replacer;

pub enum AccessType {
    Lookup,
    Scan,
    Write,
}
