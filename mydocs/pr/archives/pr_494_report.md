# PR #494 처리 보고서

**제목**: [api] Paragraph::utf16_pos_to_char_idx 추가 (옵션 A, #484)
**작성자**: DanMeon (외부 컨트리뷰터)
**처리 결과**: cherry-pick 머지 (3 commits)
**연결 이슈**: #484 (이미 closed)

## 처리 영역

PR #405 (DanMeon, v0.7.8) 와 같은 결의 외부 노출 작업. 이슈 #484 옵션 A 채택 — `helpers::utf16_pos_to_char_idx` 의 알고리즘을 `Paragraph` 인스턴스 메서드로 캡슐화.

## cherry-pick 정합

| Commit (PR) | Cherry-picked | 영역 |
|------------|---------------|------|
| `99d0290` Stage 1 | `a068645` | `paragraph.rs` 메서드 신설 + plans 문서 2 |
| `cd547bf` Stage 2 | `36631fd` | 단위 테스트 6건 추가 |
| `8d26ce0` 보고서 | `efa0054` | 최종 보고서 |

3 commits 모두 DanMeon author 보존 (cherry-pick).

## 변경 영역 (3 파일 + docs)

| 파일 | 변경 |
|------|------|
| `src/model/paragraph.rs` | +22 — `utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` 신설 |
| `src/model/paragraph/tests.rs` | +72 — 단위 테스트 6건 |
| `mydocs/plans/task_m100_484.md` | +50 |
| `mydocs/plans/task_m100_484_impl.md` | +64 |
| `mydocs/report/task_m100_484_report.md` | +55 |

총 +263 / -0.

## 핵심 코드

```rust
pub fn utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize {
    self.char_offsets
        .iter()
        .position(|&off| off >= utf16_pos)
        .unwrap_or(self.char_offsets.len())
}
```

알고리즘 1줄 — 기존 `helpers::utf16_pos_to_char_idx` 와 동일. 시그니처 차이 (`&[u32]` vs `&self`) 때문에 본체 자체 보유 (의존성 방향 model ← document_core 보존).

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1086 passed** (1080 + 6 신규) ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 영향

- **알고리즘 변경**: 없음
- **API surface**: +1 method (semver MINOR)
- **기존 helpers 호출자**: 영향 없음 (cursor_nav, clipboard 그대로)
- **렌더링 / 출력**: 영향 없음
- **WASM 노출**: 추가 검토 영역 (본 PR 은 Rust API 만)

## 후속 작업 가능성

본 PR 흡수 후 — `helpers::utf16_pos_to_char_idx` 의 내부 호출자 (cursor_nav, clipboard) 를 점진적으로 `Paragraph` 메서드로 전환 가능. 본 PR 범위 외, 후속 task 영역 (작성자가 PR 본문에 명시).

## 다음 단계

- local/devel 머지
- PR #494 댓글 + close
- 이슈 #484 자동 close (이미 CLOSED 상태이므로 댓글만)
