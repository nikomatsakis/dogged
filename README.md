Experimental persistent vector in Rust. Based on a digit-indexed trie,
as in Clojure. Supports `push()`, `get()`, and `get_mut()` as its
primitive operations for now. All O(1)-in-practice, if not in theory,
but obviously not as far as a non-persistent vector.
