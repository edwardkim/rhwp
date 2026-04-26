# PR #351 검토 — Task #347: exam_eng.hwp 표/그림 절대 좌표·Z-order·셀 padding/vpos 통합 수정

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#351](https://github.com/edwardkim/rhwp/pull/351) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#347](https://github.com/edwardkim/rhwp/issues/347) (작성자 자체 등록) |
| base/head | `devel` ← `local/task347` |
| 변경 | +1,200 / -624, **21 파일** |
| Mergeable | UNKNOWN (CI 진행 중) |
| CI | Build & Test, Analyze rust 진행 중. 다른 항목 SUCCESS |
| maintainerCanModify | ✅ true |
| 검토일 | 2026-04-26 |

## 사전 절차

### 트러블슈팅 검색 결과

| 자료 | 관련성 |
|------|--------|
| `paragraph_indent_and_table_x_position.md` | 표 x 위치 + 들여쓰기 — 본 PR 의 v1 (HorzRelTo::Page body_area 기준) 과 관련 가능성 |
| `task_76_image_layout_unification.md` | 그림 layout 통일화 — 본 PR 의 BehindText/InFrontOfText 분리와 관련 가능성 |
| `task_m100_103_attempt1_postmortem.md` | 다른 영역 |

→ 이전 트러블슈팅과의 정합성 확인 필요.

### 관련 이슈 정리

| 이슈 | 영역 | 본 PR 로 해결되는가? |
|------|------|---------------------|
| **#347** (작성자 등록) | exam_eng p2 표 좌표 오류 (HorzRelTo::Page/HorzAlign::Right) | ✅ closes #347 명시 |
| **#345** (메인테이너 등록) | exam_eng 9 → 8 **페이지 수 회귀** | ⚠️ **불명확** — 본 PR 의 6 영역에 페이지 분할 영역 없음 |

이슈 #345 는 v0.7.6 (PR #343 머지) 후속으로 메인테이너가 등록한 페이지 분할 회귀. 본 PR 의 변경이 6 영역 모두 좌표 / Z-order / 셀 padding 영역이라 페이지 수에 영향 미치지 않을 가능성. **본 PR 로 #345 해결 여부 검증 필요**.

## 변경 요약 (6 단계)

| # | 영역 | 커밋 | 파일 |
|---|------|------|------|
| 1 | 표/그림 절대 좌표 (`HorzRelTo::Page` body_area 기준 통일, `InFrontOfText` push-down 분리, BehindText 그림 y_offset 분리) | `34e432f` | `layout.rs`, `table_layout.rs`, `picture_footnote.rs` |
| 2 | TAC 그림/InFrontOfText 표 Z-order 보존 (p4 Q28) | `336cfa4` | `layout.rs` |
| 3 | 인라인 TAC 그림 x 좌표 (margin_left/indent 반영) | `039eb1e` | `layout.rs` |
| 4 | 확장 바탕쪽 `is_extension` 판별 (ext_flags 0x02 + apply_to 휴리스틱) | `5237d7d` | `body_text.rs` |
| 5 | 셀 첫 줄 y 위치 (LineSeg.vpos 우선, Q27 Center valign) | `f3ba9eb` | `table_layout.rs` |
| 6 | 셀 padding 우선순위·축소 보정 (Task #279 비대칭 케이스 호환) | `72d8245` | `table_layout.rs` |

### 변경 파일 (21)

**코드 (5):**
- `src/parser/body_text.rs` (+5/-3) — 마스터 페이지 확장 판별
- `src/renderer/height_measurer.rs` (+16/-6) — Stage 4
- `src/renderer/layout.rs` (+87/-16) — Stage 1/2/3
- `src/renderer/layout/picture_footnote.rs` (+6/-4) — Stage 1
- `src/renderer/layout/table_layout.rs` (+63/-23) — Stage 1/5/6

**골든 (1):**
- `tests/golden_svg/form-002/page-0.svg` (+572/-572)

**문서 (15):**
- 수행/구현/stage1·2·3 보고서 + 최종 보고서 + 6 첨부 PNG (시각 검증 결과) + samples/exam_eng.pdf

## ⚠️ 주요 점검 사항

### Q1. 골든 form-002/page-0.svg 의 ±572 변경 이유

PR description: "외부 API 변경 없음, **골든 SVG 갱신 없음**". 그러나 변경 파일 목록에 `tests/golden_svg/form-002/page-0.svg +572/-572` 명시. 모순 — 실제 변경 본질 확인 필요:

- 의도된 변경 (form-002 영역에 본 PR Stage 1/5/6 의 영향) 인지?
- form-002 는 Task #324 (PR #327) 영역 — 본 PR 의 셀 padding 변경 (Stage 6) 이 form-002 인너 표 페이지 분할 결과에 영향 미치는가?
- Stage 별 골든 갱신을 PR description 에 빠뜨렸는가?

### Q2. 이슈 #345 (exam_eng 9 → 8 페이지 회귀) 해결 여부

본 PR 의 6 단계는 **좌표 / Z-order / 셀 padding 영역** 으로 페이지 분할 알고리즘 변경 없음. 그러나 셀 첫 줄 y 위치 (Stage 5) 또는 셀 padding (Stage 6) 변경이 페이지 분할에 미친 영향 가능.

- 본 PR 적용 후 `samples/exam_eng.hwp` 페이지 수 9 인가?
- 만약 9 라면 #345 도 해결 — 본 PR 머지 시 close 가능
- 만약 8 라면 #345 는 별도 작업 필요

### Q3. v0.7.6 사이클 직후 영향 범위

본 PR 변경이 **이전 외부 기여 PR 의 결과** 와 충돌하는지 검증:

- KTX 목차 (Task #279, PR #282/#343 의 자체 보강): Stage 6 의 셀 padding 변경이 명시적으로 "Task #279 비대칭 케이스 호환" 라고 함 → 호환 의도 ✅
- form-002 인너 표 (Task #324, PR #327): 골든 변경 ±572 — 의도 영역인지 확인
- typeset 정합 (Task #340, PR #341): 본 PR 의 layout.rs 변경과 관계
- aift / biz_plan / 21_언어 / exam_math: 시각 회귀 영향?

### Q4. 첨부 PNG 시각 검증 범위

작성자가 6 첨부 PNG 로 시각 검증:
- p2_after, p2_q18_padding_after
- p4_after, p4_overlay_after, p4_q28_after, p4_q28_padding_after
- p8_after

→ exam_eng 페이지 2/4/8 만 검증. **다른 페이지 (p1/3/5/6/7) + 다른 샘플 (KTX/form-002/21_언어 등)** 시각 검증 결과 명시 안 됨.

## 처리 권고

### 검증 흐름

1. CI 통과 대기
2. **자동 검증** (`cargo test --lib`, svg_snapshot 6/6, clippy, wasm32)
3. **회귀 영역 점검** — 작성자가 제공한 exam_eng 외 6 핵심 샘플 페이지 수 + 골든 영향 범위
4. **이슈 #345 해결 여부 검증** — exam_eng 페이지 수 8 → 9 로 복원되는지
5. **WASM Docker 빌드 + 작업지시자 시각 검증** — exam_eng p2/p4/p8 + form-002 영향 + KTX 호환

### 작성자에게 확인 (선택)

- Q1 (골든 form-002 ±572 의도) 명시 (PR description 정정 또는 답변)
- Q2 (이슈 #345 해결 여부) 명시
- Q3 (다른 샘플 시각 검증 결과) 보강

다만 **작성자가 v0.7.6 사이클 마지막의 시간을 큰 통합 PR 처리에 쓰기보다, 메인테이너 자체 검증 + 작업지시자 시각 검증으로 결정** 하는 것이 빠를 수 있음.

## 판정 (예정)

⚠️ **자동 검증 + 시각 검증 후 결정**

작성자 분석/구현 일관 우수성 (PR #324, #340, #342 같은 레벨) 을 고려하면 머지 권장. 다만 본 PR 은 통합 + 골든 ±572 변경이라 회귀 영향 범위 검증이 필수.

## 참고 링크

- [PR #351](https://github.com/edwardkim/rhwp/pull/351)
- 이슈 [#347](https://github.com/edwardkim/rhwp/issues/347) (작성자 자체 등록)
- 이슈 [#345](https://github.com/edwardkim/rhwp/issues/345) (메인테이너 등록 — 페이지 분할 회귀)
- 트러블슈팅: `paragraph_indent_and_table_x_position.md`, `task_76_image_layout_unification.md`
- 작성자 누적 기여: PR #303, #305, #308, #315, #320, #327, #341, #343, **#351**
