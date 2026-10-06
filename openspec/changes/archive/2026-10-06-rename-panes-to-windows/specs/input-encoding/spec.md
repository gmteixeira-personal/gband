## MODIFIED Requirements

### Requirement: Encoding inputs
gband SHALL encode a key from its key code and its Shift, Alt and Ctrl modifiers. gband SHALL encode a key or a paste from two window input modes: application cursor keys (DECCKM, DEC private mode 1) and bracketed paste (DEC private mode 2004). The encoding SHALL depend on nothing else. In this spec, byte sequences use Rust byte-string escapes, so `\x1b` is ESC.

#### Scenario: Same input, same bytes
- **WHEN** gband encodes the same key with the same modes twice
- **THEN** both encodings produce the same bytes
