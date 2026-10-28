# task_m100_4403 Stage 1 — 암묵적 기본 탭이 HWPX 왕복에서 TabDef를 무시하던 결함

- **이슈**: [#4403](https://github.com/edwardkim/rhwp/issues/4403)
- **브랜치**: `fix/issue-4403-hwpx-tab-marker`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 전체 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 재현 — 픽셀로 쟀다

`rhwp export-hwpx samples/SO-SUEOP.hwp` → `ir-diff` 로 이슈 그대로 재현했다. 문단 0.19
"I.소설의 이해\t3" 에서 `tab_ext count: A=0 vs B=1`.

`render-diff --via hwpx` 실측: **46쪽 중 9쪽 임계 초과, 최대 변위 470px**(1쪽 목차).
`export-render-tree` 로 좌표를 직접 대조해 목차 쪽번호 "3" 의 x 가 원본 674.3px →
왕복 후 670.9px 로 이동함을 확인했다.

## 2. 한컴 기본 탭 4000 HWPUNIT 은 맞았다

이슈가 상수를 의심했는데 **상수는 정확했다.** OWPML 스키마(`secPr@tabStopVal` = "기본 탭 간격"),
코드 내 [Finding 14] 의 실제 한컴 생성 HWPX 픽스처 실측, HWP5 스펙 정오표 세 곳에서 확인했다.

문제는 상수가 아니라 **탭 정지를 계산하지 않는 것**이었다.

## 3. 진짜 원인 — tab_extended 하나가 TabDef 전체를 무력화한다

문단의 실제 `TabDef`(`tab_def_id=3`, `tabs=[pos=85032(300mm) type=1(RIGHT) fill=3]`)가 무시되고,
`render_hp_t_content` 가 방출한 `width="4000"` 이 재적재 시 렌더러(`text_measurement.rs`)에
**"실제 계산된 탭 폭"으로 신뢰되어** `total + width` 로 문단 정렬과 무관하게 고정 거리만 전진한다.

`TabDef` 를 파서가 채우는지도 확인했다 — **채운다.** HWP5(`doc_info.rs::parse_tab_def`)·
HWPX(`header.rs`) 모두 `doc_info.tab_defs` 를 채우고 `ParaShape.tab_def_id` 가 정확히 참조한다.
선행 문제는 없었다.

## 4. 구현 — HWP5가 이미 푼 방식을 이식

직렬화기가 진짜 픽셀 위치를 계산하는 근본 수정은 문단 레이아웃(폰트 메트릭·커서 위치) 접근이
필요해 범위가 크다는 이슈의 판단이 맞았다.

대신 HWP5 바이너리가 **동일 문제(#1892)를 이미 푼 방식**(데이터-없음 널 마커)을 HWPX 에 이식했다:

- 직렬화기 폴백을 `width="4000"` → `width="0"`(`TAB_NO_DATA_WIDTH_MARKER`). 폭 0 인 탭은 실제로
  나올 수 없어 시각 효과가 없는 안전한 마커다.
- 파서 두 경로(`read_text_content_with_tabs`, `parse_paragraph`)에 `is_tab_no_data_marker`
  (width=0 **및** leader=0·type=1 정확 일치) 추가 — 마커면 `tab_extended` 에 싣지 않아 렌더러가
  원본과 동일하게 `find_next_tab_stop`/TabDef 경로로 돌아간다.

## 5. 실측 검증

- `ir-diff`: `tab_ext count: A=0 vs B=1` → **차이 0건**
- `render-diff --via hwpx`: 9쪽 임계 초과(최대 470px) → **4쪽**(26,43,44,45 — 수정 전에도 이미
  초과였던 탭 무관 기존 결함, 새로 생기지 않음), 최대 271px. **목차 페이지(1,8,19,22,34) 완전 clean**
- 목차 쪽번호 x: 670.9px → **674.3px(원본과 정확히 일치)**
- `samples/hwp3-sample11.hwp`, `hwp3-sample19.hwp`: tab_ext 차이 0건

## 6. 범위 밖

- 직렬화기의 정밀 픽셀 위치 계산 — width=0 센티널만으로 실측 회귀(470px)가 해소돼 필요성이 줄었다.
- `fill_lines` 가 `tab_stops` 를 못 받는 결함 — PR #4380(#4324)에서 이미 별개로 기록했다.
- **부수 발견**: `secPr` 의 `tabStop`(모델 `default_tab_spacing`, 실측 8000) vs `tabStopVal`
  (=4000, 스키마상 "기본 탭 간격") 불일치. 파서가 `tabStop` 을 읽는데 스키마 문서상 기본 탭 간격은
  `tabStopVal` 쪽으로 보인다. 이번 범위 밖이라 후속 이슈 후보로 남긴다.

## 7. 검증 (완료)

- 회귀 테스트 2건 신설, 기존 2건 값 갱신. **수정 전 실패 확인**했다.
- `cargo test --profile release-test --tests` 통과.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
