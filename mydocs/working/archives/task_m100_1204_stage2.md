# Stage 2 완료보고서 — Task M100 #1204

**단계**: 파서 — 글꼴명령 body 의 decoration/구조 명령 정상 처리 (B)
**브랜치**: `local/task1204`

## 변경

`src/renderer/equation/parser.rs` `parse_single_or_group`:
- Command 분기에서 symbol/function 외 명령을 `EqNode::Text` 로 처리하던 것을 `self.parse_command(&val)` 재귀로 변경.
- `parse_command` 의 fall-through 가 미지 명령을 `Text` 로 처리하므로 truly-unknown 은 기존 동작 유지(안전), `bar`/`sqrt` 등 decoration/구조 명령은 정상 파싱.

## 검증

- `cargo build --release` 성공.
- `rm bar {F prime F}` → `FontStyle(Roman, Decoration(Bar, ...))`, "bar" Text leak 없음.
- `root3 y` → `Sqrt(3)`, "root3" Text leak 없음.
- 기존 수식 파서 테스트 회귀 0.
