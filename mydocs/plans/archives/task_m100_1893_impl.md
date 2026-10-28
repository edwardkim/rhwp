# 구현 계획서 — Task #1893

**이슈**: #1893 해양경찰청 별지 hwpx 라운드트립 렌더 452~752px 분기
**브랜치**: `local/task1893` (origin/devel 12f7c03a)
**수행계획서**: `task_m100_1893.md` (자동승인)

## 근본원인 (Stage 1 재확정 — 초기 진단 정정)

초기 진단(빈 `<t/>` 직렬화)은 **오귀속**이었다. `hwpx-roundtrip` 산출물(plain
serialize_hwpx)은 렌더 무해(2-파일 render-diff 0.00px, 10회 안정)였고, 752px 는
`render-diff --via hwpx` 가 쓰는 **DocumentCore 경유 직렬화**(`export_hwpx_native`)
에서만 재현된다(5회 안정, `export-hwpx` CLI 로 재현체 확보).

**진범**: `DocumentCore::from_bytes` 의 `clear_initial_field_texts`(초기상태 CLICK_HERE
누름틀 안내문 삭제 — 이 문서의 "소속관서"/"0000.00.00." 등 4곳)가 `para.text` 와
`field_ranges` 만 수정하고 **`char_offsets`/`char_count`/`char_shapes` 를 stale 로
방치**. 이 불일치 IR 을 직렬화하면 재파스가 정준형으로 재계산 → compose 가 갈라져
빈 TextLine 추가·752px 이동. (렌더트리 대조: 재파스본에 y=185.7 빈 TextLine/TextRun
추가 확인.)

**한컴 판정(pyhwpx 한글 PDF)**: 한컴도 안내문을 렌더하지 않음 — 삭제 동작 자체는
정합. 결함은 삭제 수술의 불변성 미완성뿐.

**부수 확인**: 원본의 자식 없는 빈 `<run/>` 33개 = 한컴 자신의 zero-width char run
표현(우리 empty-run 직렬화 rule 5 와 대칭, 무해). #1891 계열 ZIP 의심은 본 건과 무관.

## 단계 (4)

### Stage 1 — 코드 특정·메커니즘 트레이스
- 파서: `<run/>` vs `<run><t/></run>` 파스 결과의 IR 차이 지점(문단 text/char_shapes/
  세그먼트 구성) 특정.
- 직렬화기: run 방출부의 `<t>` 생성 조건 특정 — IR 이 두 형태를 구별하는지 판정.
- 빈 `<t/>` 가 조판(compose/줄바꿈)에 닿는 경로 1건 트레이스(3066571 Cell5).
- 산출: 수정 지점·방식 확정 메모(이 문서에 추기).

### Stage 2 — 직렬화기 수정 (최소 diff)
- 1순위: 텍스트 세그먼트 없는 run → `<t>` 미방출. IR 구별 가능 시 원형 보존
  (빈 `<t></t>` 실존 run 9개 왕복 유지).
- IR 구별 불가 시: 영향면 판단 후 보수 규칙 채택(사유 기록).

### Stage 3 — 검증
- fixture: 3066571 render-diff 752→0px. 해경 계열 11건 전수 재검.
- 게이트: cargo test 전체(hwpx_roundtrip_baseline 포함), samples/hwpx 전수,
  big_hwpx 2,500 render-diff(회귀 0), 20k 표본 hwpx 축 재검(개선 확인).

### Stage 4 — fixture 동봉 + 보고서
- 재현 hwpx 를 `samples/` 동봉 + roundtrip 자기정합 테스트 핀.
- `task_m100_1893_report.md`, PR 생성(pr/devel-1893).
