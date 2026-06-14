# PR #1406 검토 — 그림/도형/묶음 캡션(hp:caption) 파싱·직렬화 (#1403)

- PR: https://github.com/edwardkim/rhwp/pull/1406
- 제목: fix(hwpx): 그림/도형/묶음 캡션(hp:caption) 파싱·직렬화 누락 — roundtrip 캡션 소실 (#1403)
- 작성일: 2026-06-12 / 검토일: 2026-06-13
- 작성자: `oksure` (누적 merged 2건 — #1398, #1327. 첫 PR 아님. 현재 열린 PR 5건 운영 중)
- 관련 이슈: #1403 (#1387 1단계 발견 분리)
- base: `devel` (`f19c6a06` — #1387 머지 직후 분기) / head: `oksure:contrib/fix-1403-pic-shape-caption` (`3baf8724`, 단일 커밋)
- 로컬 검토 브랜치: `local/pr1406-review` / 검증 머지: `local/pr1406-merged`

## 1. 요약 판단

**merge 권고** (보완 1건은 후속 처리 가능 수준).

#1403의 3중 누락(파서/serializer/게이트)을 정확히 짚었고, #1387(`parse_table_caption`·
`write_caption` 공유)과 #1387/#1388의 게이트 동승 패턴(`ObjectCaption` ↔ `TableCaption`
대칭)을 그대로 따른 모범적 구현이다. 이슈 본문과의 분류 차이(pic 8+line 3 vs
pic 9+container 2)도 본문에서 스스로 규명했다 — 코퍼스에 `hp:line` 요소 0건이므로
컨트리뷰터 분류가 더 정확하다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open, mergeable **MERGEABLE** (#1382 머지 후 최신 devel 기준 재계산 확인) |
| 변경량 | 7 files, +520 / -37 |
| GitHub CI | Build & Test pass (13m36s) + Analyze 3종 + CodeQL 전부 pass |
| 신규 테스트 | 10건 (게이트 4 + 방출 5 + 통합 1) |

## 3. 검토 내용

### 3.1 파서 (`parser/hwpx/section.rs`) — 적합

- `parse_picture`/`parse_shape_object`(그리기 6종 공통)/`parse_container`에
  `b"caption"` arm 추가, `parse_table_caption`(#1387) 재사용 — 중복 없음.
- `parse_shape_object`의 `..Default::default()` 제거는 전 필드 명시로 대체 (컴파일 보증).

### 3.2 serializer — 적합

- `write_caption`을 `pub(super)` 공유, 방출 위치는 한컴 실물(aift)·OWPML 순서 근거 명기.
- legacy 문자열 경로(ellipse/arc/polygon/curve/chart/ole)까지 방출 — **HWP5→HWPX
  변환의 기존 소실 결함도 함께 해소** (HWP5 파서는 전 도형 캡션 적재 중).
- `write_picture` 등 `ctx` `&mut` 전환 사유(캡션 문단의 para id 발급·참조 수집) 명기 — 타당.

### 3.3 게이트 (`roundtrip.rs`) — 적합 + 보완 1건

- `ObjectCaption` variant + `diff_table_caption` 공유 + `shape_caption` 접근자
  (그리기 6종 `drawing.caption` + Group/Chart/Ole/Picture 전용 필드) — #1387 패턴 정합.
- char_shapes·controls 재귀: Picture arm 신설 + `diff_shape_char_shapes` caption 방문 ✓
- **보완(Minor)**: `diff_paragraph_linesegs`/`diff_shape_linesegs`에는 객체 캡션
  재귀가 없다 — #1387은 표 캡션을 char_shapes·linesegs 양쪽에 동승시켰으므로 대칭
  결여. 보존 자체는 공유 `write_caption` 경로가 lineseg를 방출하므로 정상이고,
  **검출 범위만 좁은 게이트 사각** — merge 차단 사유 아님. 후속 보완 요청.

### 3.4 #1382(오늘 머지)와의 의미 호환 — 확인 완료

PR은 #1382 이전 분기이나, 캡션 내 autoNum 보유 샘플이 ta-pic(표 캡션 — #1382에서
처리)뿐이라 상호작용 없음. 아래 로컬 검증이 최신 devel과의 의미 호환을 입증.

## 4. 로컬 검증 (`local/pr1406-merged` = 최신 devel + PR)

| 명령 | 결과 |
|---|---|
| `git merge local/pr1406-review` (devel 기준) | 충돌 없음 |
| `cargo test --profile release-test --tests` | **2262 passed, 0 failed** |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets` | 경고 0 |

## 5. 권장 조치

1. **merge** (devel)
2. merge 코멘트로 후속 보완 1건 요청 또는 메인테이너 측 후속 처리:
   객체 캡션의 lineseg 재귀 동승 (`diff_paragraph_linesegs` Picture arm +
   `diff_shape_linesegs` caption 방문 — #1387 대칭 복원)
3. 이슈 #1403 close (merge 시 closes 연동 확인)
4. 본 검토 문서 `pr/archives/` 이동 (처리 완료 후)
