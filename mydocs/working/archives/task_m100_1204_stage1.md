# Stage 1 완료보고서 — Task M100 #1204

**단계**: 토크나이저 — root/sqrt(A), prime(C) glued 분리
**브랜치**: `local/task1204`

## 변경

`src/renderer/equation/tokenizer.rs` `read_command`:
- **A**: `["root","sqrt","ROOT","SQRT"]` — 뒤가 **숫자**면 키워드만 소비 후 분리. (over/atop digit-guard 패턴, letter 변수 충돌 회피.)
- **C**: `["prime","PRIME"]` — 뒤가 **alnum**이면 분리. (bold/it/rm 패턴.)

## 검증

- `cargo build --release` 성공.
- 토크나이저 단위 테스트: `root3 y`→[root,3,y], `sqrt5`→[sqrt,5], `primeF`→[prime,F], `rootn`(letter)→[rootn] 유지, `root {4} of {x}`(공백/중괄호) 유지.
