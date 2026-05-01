# PR #494 검토 문서

**제목**: [api] Paragraph::utf16_pos_to_char_idx 추가 (옵션 A, #484)
**작성자**: DanMeon (외부 컨트리뷰터, PR #405 의 동일 결 작업)
**Base/Head**: `devel ← feature/expose-utf16-pos-to-char-idx`
**상태**: OPEN, MERGEABLE
**연결 이슈**: #484 (이미 CLOSED — 등록자 = 작성자)

## PR 본질

이슈 #484 옵션 A 채택 — `helpers::utf16_pos_to_char_idx` (pub(crate)) 의 알고리즘을 `Paragraph` 인스턴스 메서드 (`pub`) 로 캡슐화하여 외부 crate (PyO3 / napi / JNI 등 binding) 에서 호출 가능하게.

### 선례 정합

PR #494 는 PR #405 와 **같은 패턴**:

| PR | helpers 함수 | Paragraph 메서드 |
|----|--------------|------------------|
| **PR #405** (v0.7.8 반영) | `find_control_text_positions` | `Paragraph::control_text_positions(&self) -> Vec<usize>` |
| **PR #494** (본 PR) | `utf16_pos_to_char_idx` | `Paragraph::utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` |

작성자 DanMeon 동일 — 외부 binding (rhwp-python) 작업 중 발견한 helpers 모듈의 외부 노출 작업 시리즈.

## 변경 영역 (3 commits, 5 files)

| Commit | 영역 |
|--------|------|
| `99d0290` Stage 1 | `src/model/paragraph.rs` (+22): `utf16_pos_to_char_idx` 메서드 신설 |
| `cd547bf` Stage 2 | `src/model/paragraph/tests.rs` (+72): 단위 테스트 6건 |
| `8d26ce0` 최종 보고서 | docs (3 파일, +169) |

총 +263 / -0 (변경 없는 신규 추가만).

## 핵심 코드

```rust
// src/model/paragraph.rs (impl Paragraph 안)
pub fn utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize {
    self.char_offsets
        .iter()
        .position(|&off| off >= utf16_pos)
        .unwrap_or(self.char_offsets.len())
}
```

알고리즘 1줄 — `helpers::utf16_pos_to_char_idx` 와 동일. 시그니처 차이 (raw `&[u32]` vs `&self`) 때문에 본체 자체 보유 (의존성 방향 model ← document_core 보존). 작성자 본인이 본문에 silent drift 위험 무시 가능 명시.

## 영향 영역

| 항목 | 정합 |
|------|------|
| **알고리즘 변경** | 없음 — 동일 single-pass scan |
| **API surface** | +1 method (semver MINOR) |
| **기존 helpers 호출자** | 영향 없음 (cursor_nav, clipboard 그대로 helper 호출) |
| **렌더링 / 출력** | 영향 없음 (model 메서드만 추가) |
| **WASM 노출** | 추가 검토 영역 (현재 PR 은 Rust API 만, WASM 노출은 후속) |

## 검증 정합

- **cargo test --lib**: 1081 passed (baseline 1075 + **6 신규 단위 테스트**)
- **cargo clippy**: 변경 파일 warning 0
- **CI**: 점검 필요

## 영역 충돌 점검

| 영역 | PR |
|------|-----|
| `src/model/paragraph.rs` | **PR #494 만** |
| `src/renderer/layout/paragraph_layout.rs` | PR #478 (다른 영역 — 충돌 없음) |

## 처리 옵션

| 옵션 | 진행 |
|------|------|
| A. **cherry-pick 머지** (PR #405 와 같은 패턴) | 3 commits 분리 cherry-pick — Stage 1 + Stage 2 + 보고서 |
| B. **squash 머지** | 단일 commit 으로 통합 |
| C. **요청 후 변경** | 추가 정정 의뢰 (현재 정합으로는 불필요) |

## 권장

**A (cherry-pick 3 commits)** — PR #405 와 동일한 패턴, 작성자가 Stage 별로 commit 분리 + Hyperfall 절차 정합. 보고서까지 명시적으로 보존하면 향후 추적 용이.

## 검증 게이트 (머지 전)

- cargo test --lib: 1080 → **1086 passed** (1080 + 6 신규)
- cargo test --test svg_snapshot: 6/6
- cargo test --test issue_418: 1/1
- cargo clippy --lib -- -D warnings: 0건
- WASM 빌드 정합

## PR 댓글 정합 (메모리 `feedback_pr_comment_tone`)

차분한 사실 중심:
- PR #405 와 같은 결의 외부 노출 작업 흡수
- 동일 패턴 (helpers → Paragraph 메서드 캡슐화)
- 알고리즘 변경 없음 + 단위 테스트 6 추가
- DanMeon 의 반복 컨트리뷰션 — 매번 같은 인사 자제

## 후속 작업 가능성

본 PR 흡수 후 — `helpers::utf16_pos_to_char_idx` 의 내부 호출자 (cursor_nav, clipboard) 를 점진적으로 `Paragraph` 메서드로 전환 가능. PR 본문에 작성자가 명시. 본 PR 범위 외, 후속 task 영역.
