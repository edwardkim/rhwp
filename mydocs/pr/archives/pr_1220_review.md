# PR #1220 검토 — HWP5 wrap=Square 호스트 본문 커서 전진 (답안↔문제 겹침)

- **작성일**: 2026-06-01
- **PR**: #1220 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1202/#1203/#1208 머지)
- **연결 이슈**: #1218 (OPEN)
- **base/head**: `devel` ← `fix/1218-wrap-square-host-height` (cross-repo)
- **규모**: 8 파일, +295/−0 (소스 `src/renderer/layout.rs` 단일 hunk +19 + 작업문서 7)
- **mergeable**: MERGEABLE / BEHIND
- **CI**: **전부 PASS** (Build&Test/Canvas visual diff/Analyze×3/CodeQL)
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (계측 확정)

`3-09월_교육_통합_2023.hwp` 4쪽 문26: 답안 ①(0.7262)가 문제 끝줄과 세로 겹침. `wrap=Square`
인라인 표가 커서를 표 높이만큼만 전진시키고 호스트 본문(`layout_wrap_around_paras`, void 렌더)은
커서를 전진 안 시켜, 본문(90px) > 표(73.6px)일 때 다음 단락(①)이 표 하단에서 시작 → 겹침.

## 2. 수정 내용 검토 (layout.rs:4401, +19 단일 hunk)

`layout_wrap_around_paras` 직후 호스트 본문 단락(`composed[para_index]`)의 텍스트 높이로
본문 하단을 계산해 커서 전진:
```rust
let host_text_bottom = table_y_before + (text_h - last_ls).max(0.0);
if host_text_bottom > y_offset { y_offset = host_text_bottom; }
```
- 마지막 줄 trailing line_spacing 제외(height_for_fit 정합).
- **가드**: 본문 ≤ 표 인 기존 다수 케이스는 `host_text_bottom ≤ y_offset` → 동작 불변.

## 3. ⚠️ 핵심 위험 — double advance 트러블슈팅 정합 (검증 완료)

트러블슈팅 `square_wrap_pic_bottom_double_advance.md`: Square wrap 에서 커서를 그림/표 하단으로
advance 하면 **wrap-around paragraph 누적 height 와 결합한 double advance** → exam_science.hwp
페이지네이션 회귀(4→6쪽, p2 37→2 items). 옵션 C(col 경계 검사) 효과 없음, 옵션 3(HWP5 case-guard) 권장.

**본 PR 의 회귀 여부를 직접 검증** (트러블슈팅 지표):

| 지표 | 보정 전(devel) | 보정 후(PR) | 판정 |
|------|---------------|-------------|------|
| exam_science 페이지 수 | 4 | **4** | ✅ 무회귀 |
| p2 단 0 items | 37 | **37** | ✅ |
| p2 단 1 items | 45 | **45** | ✅ |

→ **함정 무재발.** PR 보정은 **호스트 본문 단락 자기 텍스트 높이(`composed[para_index]`)** 기준이라
wrap-around 누적과 결합하지 않음(트러블슈팅 함정은 그림+wrap-around 누적 결합). 표(Table) 호스트
한정 + `> y_offset` 가드로 안전.

## 4. 검증 결과 (로컬 머지 시뮬 `pr1220-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (충돌 0) |
| fmt | ✅ clean |
| 전체 테스트 `cargo test --tests` | ✅ **1924 passed, 0 failed** (svg_snapshot / issue_546 wrap_around 회귀 포함) |
| **exam_science 회귀(double advance)** | ✅ 4쪽/37items 보정 전후 동일 |
| **문26 겹침 해소 (정량)** | ✅ 최하단 ① y 보정 전 906.3(문제끝줄 904.7과 겹침) → 후 **922.8**(18px 아래 분리) |

산출물: `output/poc/pr1220/{before,after}/3-09월_교육_통합_2023_004.svg`.

## 5. 판단 — 머지 권고 (시각 판정 게이트)

- 진단 정확(계측), 가드로 기존 케이스 불변, **트러블슈팅 함정(double advance) 무재발 직접 확인**,
  전체 테스트 1924 passed, 겹침 해소 정량 확인, CI green.
- 레이아웃 핵심 경로 + 시각 영향 → **작업지시자 시각 판정** 게이트(4쪽 문26 ①~⑤ 각 줄 분리 ↔
  한글 2022 PDF). 산출물 before/after SVG.
- 승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push + WASM 빌드(렌더 경로 변경).

## 6. 비고

- z-표 행 압축(셀 내부 세로정렬/줄높이)은 별도 서브시스템 — PR 범위 외(별도 이슈, PR 본문 명시).
- 트러블슈팅 옵션 3(HWP5 case-guard)은 그림+wrap-around 결합 함정 대비책이었으나, 본 PR 은 호스트
  본문 자기 높이 기준이라 해당 함정과 무관함이 실측으로 확인됨.
