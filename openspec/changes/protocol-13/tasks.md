## 1. Protocol version

- [ ] 1.1 Set `PROTOCOL_VERSION` to 13 in `crates/protocol/src/message.rs`, and rename `protocol_version_is_twelve` in `crates/protocol/tests/messages.rs` to `protocol_version_is_thirteen`, asserting 13. Verify with `cargo test -p gband-protocol --test messages`
- [ ] 1.2 Add `version_twelve_client_is_rejected_by_version_thirteen` to `crates/server/tests/sync.rs`, sending a version 12 hello and expecting `HelloReply::Rejected { version: 13 }` and a closed connection, and change `version_eleven_client_is_rejected_by_version_twelve` to expect version 13 under the name `version_eleven_client_is_rejected_by_version_thirteen`. Verify with `cargo test -p gband-server --test sync`

## 2. Gate

- [ ] 2.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Verify all three exit with status 0
