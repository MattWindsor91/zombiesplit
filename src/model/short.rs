//! Type aliases for short names and associated types.

use bimap::BiHashMap;

/// Type alias for short names.
pub type Name = String;

/// Type alias for bidirectional maps from short names to items.
pub type Bimap<T> = BiHashMap<Name, T>;

/// Type alias for maps from short names to items.
pub type Map<T> = std::collections::HashMap<Name, T>;

/// Type alias for insertion-ordered maps from short names to items.
pub type LinkedMap<T> = linked_hash_map::LinkedHashMap<Name, T>;
