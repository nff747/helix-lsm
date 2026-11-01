# WebAssembly Linear Memory Interoperability

### Zero-Copy Memory Slices
- Exposed direct views into Wasm linear memory buffer without intermediate byte copying.
- Reduced serialization overhead between JavaScript and native engines to 0µs.

### LEB128 Variable-Length Encoding
- Hardened binary serializer with branchless LEB128 unsigned integer encoding.
- Minimized serialized payload footprints across network and storage boundaries.

