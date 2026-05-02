# PR #506 처리 보고서

**제목**: Task #460: HWP 3.0 파서 + Square wrap 그림 어울림 렌더링
**작성자**: jangster77 (Taesup Jang)
**처리 결과**: 단일 머지 (51 commits + 작성자 author 보존)

## 처리 본질 — HWP 3.0 포맷 첫 지원

본 PR 은 **HWP 3.0 (한컴 1990년대~2000년대 초반 레거시 포맷) 의 첫 오픈소스 파서 구현**. rhwp 가 이제 HWP 5.0 + HWPX + **HWP 3.0** 3 포맷 모두 지원.

작업지시자 통찰: *"이제 우리도 HWP 3.0 포맷까지 성공했습니다. 컨트리뷰터가 대단합니다."*

## 변경 영역

### HWP 3.0 파서 신규 (10 파일, 본질 영역)

| 파일 | 영역 |
|------|------|
| `src/parser/hwp3/mod.rs` | parse_hwp3() 진입점 + Document IR 변환 |
| `src/parser/hwp3/records.rs` | 레코드 파싱 (+414 lines) |
| `src/parser/hwp3/paragraph.rs` | 문단 파싱 |
| `src/parser/hwp3/encoding.rs` | 인코딩 변환 |
| `src/parser/hwp3/johab.rs` | Johab 조합형 → UTF-8 디코딩 |
| `src/parser/hwp3/johab_map.rs` | Johab 매핑 데이터 |
| `src/parser/hwp3/special_char.rs` | 특수문자 처리 |
| `src/parser/hwp3/drawing.rs` | 그리기 객체 파싱 |
| `src/parser/hwp3/ole.rs` | OLE 컨테이너 파싱 |
| `src/parser/mod.rs` | `FileFormat::Hwp3` → `parse_hwp3()` 라우팅 |

### 렌더러 정정 (Square wrap 어울림)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | +154 lines — `wrap_pic_bottom_y` 계산 + 앵커 Shape 처리 후 y_offset 전진 |
| `src/renderer/typeset.rs` | +66 lines — wrap zone 종료 시 `current_height` 그림 하단까지 보정 |
| `src/renderer/pagination/engine.rs` | +26 lines — 페이지네이션 정합 |
| `src/renderer/layout/paragraph_layout.rs` | +6 lines (작은 변경) |
| `src/renderer/layout/picture_footnote.rs` | +5 lines |
| `src/document_core/commands/document.rs` | HWP3 IR → HWP5 저장 영역 |
| `src/document_core/converters/hwpx_to_hwp.rs` | 변환 영역 |

### 신규 샘플

- `samples/hwp3-sample.hwp` (87 KB) — HWP 3.0 검증 자료
- `samples/hwp3-sample4.hwp`
- `samples/hwp3-sample5.hwp`

## 51 commits 정합

### Task #417 (HWP 3.0 파서 본질, Stage 1~4)

- Stage 1: HWP 3.0 파서 초기 구현 + 코드 정리
- Stage 2: HWP3 그림 겹침 + 캡션 렌더링 + AutoNumber 번호
- Stage 3: HWP3 하이퍼링크 URL 추출
- Stage 4: 최종 결과 보고서

### Task #460 (Square wrap + 렌더러 정정, Stage 1~9 + 보완)

- Stage 1: HWP3 AutoNumber U+FFFC → 공백 (파서 처리, 렌더러 분기 제거)
- Stage 2: HWP3 혼합 단락 LINE_SEG 높이 보정 (그림 겹침 해소)
- Stage 3: 최종 검증 + 결과 보고서
- Stage 4: border_fill_id 1-기반 인덱스 + body clip 하단 제한
- Stage 5: HWP3 LINE_SEG line_spacing=0 (표 텍스트 겹침 해소)
- Stage 6: 페이지 경계 대형 그림 anchor (TypesetEngine + 파서)
- Stage 7: 비-TAC 그림 LAYOUT_OVERFLOW 해소
- Stage 8: HWP3 → HWP5 저장 지원
- Stage 9: HWP3 → HWP5 재열기 그림 위치 복원 (common.attr 비트필드)
- 보완 1~3: 테이블 CTRL_HEADER / 표 TAC 속성 / BinData attr 비트
- 리팩토링: HWP3 수정 mod.rs 집중
- CharShape 수정: 다중 hchar 컨트롤 CharShape 위치 매핑

### Merge upstream/devel

- 작성자가 본 사이클 devel (Task #501 cell.padding + PR #478 7 Task + PR #498 + v0.7.9 릴리즈) 이미 흡수
- 충돌 해결: `.gitignore` + `integration_tests.rs`

## 검증 게이트

| 검증 | 결과 |
|------|------|
| cargo build | ✓ 성공 (rhwp v0.7.9) |
| cargo test --lib | **1105 passed** ✓ (devel 1102 + HWP 3.0 신규 3) |
| cargo test --test svg_snapshot | **6/6** ✓ |
| cargo test --test issue_418 | **1/1** ✓ |
| **cargo test --test issue_501** | **PASS** ✓ (Task #501 cell.padding 회귀 0) |
| cargo clippy --lib -- -D warnings | **0건** ✓ |
| **WASM 빌드** | **4,378,441 bytes** ✓ (이전 v0.7.9 4,202,430 대비 +176 KB = HWP 3.0 파서 + Johab 매핑) |

## 본 사이클 정합 충돌 점검

| 영역 | 본 사이클 | PR #506 | 결과 |
|------|----------|---------|------|
| `table_layout.rs` | Task #501 cell.padding | 변경 0 | 충돌 없음 ✓ |
| `paragraph_layout.rs` | #488/#490/#489/#495 | +6 lines | 충돌 없음 ✓ |
| `layout.rs` | #480/#476 | +154 lines | issue_501 PASS — 충돌 없음 ✓ |
| `typeset.rs` | (없음) | +66 lines | svg_snapshot 6/6 — 회귀 0 ✓ |
| `pagination/engine.rs` | (없음) | +26 lines | 회귀 0 ✓ |

## 머지 정합

- 1차 머지: `local/pr506-test → local/devel` (--no-ff)
- 2차 머지: `local/devel → devel` (--no-ff)
- devel push: **`c7330cf`** 

## 권장 옵션 결정 — B (단일 머지)

옵션 B 선택 사유:
1. 작성자가 51 commits 의 다단계 정정 (Task #417 + Task #460 + 보완 1~3 + 리팩토링) 모두 본인 분기에서 마침
2. 본 사이클 devel (Task #501 + PR #478 + PR #498 + v0.7.9) 이미 흡수 (`Merge upstream/devel`)
3. 자동 검증 게이트 모두 통과 + 본 사이클 회귀 영역 (issue_501) PASS
4. HWP 3.0 본질 = 완성된 사이클 — 분리 cherry-pick 의 단계별 시각 판정 부담 보다 **머지 후 광범위 시각 판정** 정합
5. 메모리 룰 `feedback_small_batch_release_strategy` 영역 외 — HWP 3.0 신규 포맷 자체가 큰 묶음 (분리 어려움)

## 다음 단계

1. **작업지시자 광범위 시각 판정** (Stage 5 게이트):
   - HWP 3.0: `hwp3-sample.hwp` / `hwp3-sample4.hwp` / **`hwp3-sample5.hwp` 페이지 4** (PR 본문 정합)
   - HWP5 회귀 점검: exam_kor / exam_science / exam_eng / 21_언어_기출 / mel-001 (Task #501)
   - 본 사이클 정정 영역: synam-001 / k-water-rfp / aift / kps-ai / hwpspec
2. 시각 판정 통과 후 PR #506 댓글 + close
3. 시각 판정 회귀 발견 시 별도 task 분리 + 정정 (revert 부담 감수)

## 작성자 영역 보존

- 51 commits 모두 author 보존 (jangster77 / Taesup Jang)
- 단계 보고서 (`mydocs/working/task_m100_417_*` + `task_m100_460_*`) 모두 보존
- 작성자가 본인 분기에서 본 사이클 devel 흡수 → 충돌 해결 보존

## 메모리 룰 정합

- `feedback_pr_comment_tone` — 차분한 사실 중심
- `feedback_v076_regression_origin` — 작업지시자 직접 시각 검증 게이트 (Stage 5)
- `feedback_assign_issue_before_work` — 시각 판정 회귀 발견 시 즉시 assignee 지정

## rhwp 의 새로운 가치

본 PR 의 흡수로 rhwp 는 다음 3 포맷 모두 지원:

- **HWP 5.0** (1990년대 후반~ 현재) — 한컴 오피스 메인 포맷 (OLE2 binary)
- **HWPX** (2010년대~) — Open XML 기반 (ZIP + XML)
- **HWP 3.0** (1990년대~ 2000년대 초반) — Johab 조합형 인코딩 + OLE 컨테이너 ★ (본 PR)

→ rhwp = **30년에 걸친 한국 한글 문서 포맷의 오픈소스 변환/렌더링 플랫폼** 으로 한 단계 도약.

## 작업지시자 통찰 인용

> *"이제 우리도 HWP 3.0 포맷까지 성공했습니다. 컨트리뷰터가 대단합니다."*

@jangster77 (Taesup Jang) 의 광범위 작업 (51 commits / 13,018 lines / Task #417 + Task #460 통합) 에 깊이 감사드립니다.
