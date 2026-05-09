---
PR: #714
제목: Task #712 — wrap=TopAndBottom 음수 vert_offset 표 침범 정정
컨트리뷰터: @planet6897 (Jaeuk Ryu) — 30+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — 6 commits 단계별 보존 cherry-pick + no-ff merge
처리일: 2026-05-09
머지 commit: f4663fe0
---

# PR #714 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (6 commits 단계별 보존 cherry-pick + no-ff merge `f4663fe0`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `f4663fe0` (--no-ff merge) |
| Issue #712 | close 자동 정합 (closes #712) |
| 시각 판정 | ★ **통과 (작업지시자 직접)** |
| 자기 검증 | lib **1173** + 통합 ALL GREEN + issue_712 1/1 + clippy clean |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** |
| WASM 빌드 | 4,606,183 bytes (md5 `9ed80056...`) |

## 2. 정정 본질 — `HwpUnit=u32` signed 캐스트 누락

`vertical_offset` 음수 (예: -1796 HU) 영역의 unsigned 비트표현 = `0xFFFFF8FC` = `4294965500u32`. `> 0` 게이트 영역에서 unsigned 양수로 통과 → 후속 `as i32` 캐스트 영역에서 음수 영역 적용 → 표가 위로 점프, 직전 인라인 표 침범.

비-Partial 경로 (`table_layout.rs:1069+`) 영역에는 `raw_y.max(y_start)` 클램프 영역으로 음수 무력화. **Partial 경로** (`table_partial.rs:59-78`) 영역에는 클램프 부재 영역 → 결함 노출.

### 2.1 영향 샘플
`samples/2022년 국립국어원 업무계획.hwp` 영역의 페이지 31 영역 (본 환경 동적 탐색 정합):
- pi=585: 1×3 인라인 TAC 제목 표 ("붙임 / / 과제별 추진일정"), wrap=TopAndBottom
- pi=586: 12×5 일정 표 (vert=문단 **-1796 HU 음수**)
- BEFORE: pi=586 외곽 상단이 pi=585 안쪽으로 ~15.94 px 침범
- AFTER: 침범 0 px (PDF 권위 정합)

### 2.2 정정 (signed 비교 14 라인)

`src/renderer/layout/table_partial.rs:59-78` (Partial 경로, 본질):
```rust
let vert_off_signed = table.common.vertical_offset as i32;
&& vert_off_signed > 0   // [Task #712] signed 비교
{
    y_start + hwpunit_to_px(vert_off_signed, self.dpi)
}
```

`src/renderer/layout.rs:2687+` (비-Partial 경로 게이트 동기):
```rust
&& (t.common.vertical_offset as i32) > 0   // [Task #712]
```

→ `feedback_image_renderer_paths_separate` 권위 룰 정합 (Partial / 비-Partial 두 경로 동기 정정).

### 2.3 영향 좁힘
- `is_continuation=true` 분할 표 연결 페이지 영역 무영향
- `vertical_offset >= 0` 영역 (signed 양수/0) 무영향
- 비-TopAndBottom wrap / TAC 표 / 비-Para vert_rel_to 영역 무관

## 3. 본 환경 cherry-pick + 검증

### 3.1 cherry-pick (6 commits)
```
770c9408 Task #712 Stage 0/1: 수행 계획서
be5efb1a Task #712 Stage 0/2: 구현 계획서
32b162bf Task #712 Stage 1 (RED): pi=585/586 침범 회귀 테스트 — 현재 FAIL 확인
e8e4a31b Task #712 Stage 2-3 (GREEN): u32 vert_offset 게이트 signed 정정
12d20bcd Task #712 Stage 4-5: 회귀 + 광범위 검증 (회귀 0)
0873aba5 Task #712 Stage 6: 최종 결과 보고서 (closes #712)
```
충돌 0건 (auto-merging layout.rs + table_partial.rs).

### 3.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (27.80s) |
| `cargo test --release --test issue_712` | ✅ **1/1 PASS** (회귀 가드, 동적 페이지 탐색) |
| `cargo test --release --test svg_snapshot` | ✅ 8/8 (form-002 PR #706 영역 보존) |
| `cargo test --release test_705` | ✅ 6/6 (PR #711 영역 보존) |
| `cargo test --release test_634` | ✅ 8/8 (가드 갱신 후 보존) |
| `cargo test --release` | ✅ lib **1173** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| 광범위 sweep | 7 fixture / **170 페이지 / 회귀 0** ✅ |
| WASM 빌드 (Docker) | ✅ 4,606,183 bytes |

### 3.3 본 환경 직접 페이지 탐색

`./target/release/rhwp dump-pages` 영역의 pi=585/586 동시 등장 페이지: **페이지 31** (global_idx=30, page_num=29).

→ 작업지시자 안내 정합 ("PR에 언급된 36페이지는 31 페이지에 위치"). 회귀 가드 영역의 동적 페이지 탐색 영역 자동 적응 (PR #644/#711 머지 후 페이지네이션 변동 영역).

### 3.4 시각 판정 ★ 통과

작업지시자 직접 시각 판정 (2026-05-09):
- BEFORE / AFTER SVG 비교 — pi=586 외곽 상단 영역 의 pi=585 안쪽 침범 영역 해소 정합
- 한컴 PDF 권위본 (`pdf/2022년 국립국어원 업무계획-2022.pdf`) 정합

### 3.5 머지 commit
`f4663fe0` — `git merge --no-ff local/task714` 단일 머지 commit. PR #694/#693/#695/#699/#706/#707/#710/#711 패턴 일관.

## 4. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @planet6897 30+ 사이클 핵심 컨트리뷰터 |
| `feedback_hancom_compat_specific_over_general` | signed 비교 + is_continuation/wrap/rel_to 가드 영역 영향 좁힘 |
| `feedback_image_renderer_paths_separate` | Partial / 비-Partial 두 경로 동기 정정 (signed 캐스트) |
| `feedback_process_must_follow` | TDD Stage 0/1 (계획) → Stage 1 RED → Stage 2-3 GREEN → Stage 4-5 sweep → Stage 6 보고서 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI ALL SUCCESS + 회귀 가드 + 광범위 sweep) + 작업지시자 시각 판정 ★ 통과 |
| `feedback_visual_regression_grows` | 동적 페이지 탐색 — 페이지네이션 변동 영역 견고 회귀 가드 신규 사례 |

## 5. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- HwpUnit=u32 영역 의 signed 캐스트 누락 영역 의 다른 영역 (audit 가능) — 본 PR 범위 외, 향후 sweep 가능

---

작성: 2026-05-09
