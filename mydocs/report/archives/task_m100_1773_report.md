# Task #1773 최종 보고 — render-diff TextRun ±1 WARN 등급 분리

## 개요

#1773 판별(레거시 무-컨트롤문자 구역정의의 직렬화 정규화 → 재파스 조성 차이 → TextRun ±1)의
후속 처리로, 판별된 두 방향 중 저위험 안인 **② render-diff 게이트 WARN 등급 분리**를 구현했다.
① 원 인코딩 보존(파서 attachment-mode 플래그 + 직렬화 분기)은 별도 설계 과제로 이슈 open 유지.

## 구현

`src/diagnostics/render_geom_diff.rs`:

- `walk()` 의 LCS 삽입/삭제 이벤트에 노드 타입을 기록 (`PageAccum.struct_inserts/struct_deletes`)
- `is_textrun_pm1()`: 이벤트 전부 TextRun + 삽입·삭제 각 ≤1 → `PageGeomDiff.struct_textrun_pm1`
- status 판정 재구성:
  - `hard_struct_pages`(TextRun ±1 로 설명 안 되는 구조 불일치) > 0 → STRUCT_MISMATCH (종전 유지)
  - 변위 임계 초과 → OVER (조성 노이즈 있어도 하드 유지)
  - TextRun ±1 뿐 + 변위 임계 이내 → **WARN_TEXTRUN** (신설, 하드 실패 아님)
- `status_is_hard_failure`: PASS | WARN_TEXTRUN 통과
- 배치 요약에 WARN_TEXTRUN 카운트, 단일 보고에 `[STRUCT:TextRun±1]` 마커, 종료코드는
  `status_is_hard_failure` 공통 사용

### 보수성 설계

- 삽입/삭제 서브트리는 **최상위 노드 타입만** 기록 — TextLine 삽입(내부에 TextRun 포함)은
  TextRun 이벤트가 아니므로 종전대로 하드 실패
- 같은 방향(삽입 또는 삭제) 2개 이상이면 하드 유지 — 페이지당 ±1 만 완화
- 페이지 수 불일치(PAGE_MISMATCH)는 완화 대상 아님

## 검증

- 단위 테스트 6종 추가 (predicate 경계, WARN 강등, 비-TextRun 하드 유지, OVER 우선,
  혼재 페이지 하드 우선, 게이트 판정) — `diagnostics::render_geom_diff` 13/13 통과
- 케이스 문서 2912837(sample_hwp/admrul_072.hwp) 재검: `[STRUCT:TextRun±1]` 마커 정확 검출,
  변위 168px 로 **OVER 유지** (조성 노이즈여도 대변위는 하드 실패 — 설계 의도)
- 코퍼스 스팟 (TextRun-only struct_delta 전수):
  - big_hwp admrul_0644 (104px) / admrul_0645 (465px): `[STRUCT:TextRun±1]` 검출 + OVER 유지
  - big_hwpx seoul_0620 (23px): TextRun:-1 — OVER 대상
  - 일반 구조 붕괴(admrul_0020 Cell:-9;Line:-6): STRUCT_MISMATCH 유지 — 부당 강등 없음

**관찰**: 현 코퍼스의 TextRun ±1 케이스는 전부 대변위 동반이라 WARN_TEXTRUN 실사례는 0건 —
본 수정의 즉효는 STRUCT(구조 붕괴) 분류에서 조성 노이즈를 분리(`[STRUCT:TextRun±1]` 마커 +
STRUCT→OVER 재분류)하는 triage 정밀화이며, WARN 강등은 향후 소변위 케이스에 대비한 안전망이다.
근본 해소는 ① 원 인코딩 보존 설계(별도 태스크)로 남는다.

## 잔여

- ① 원 인코딩 보존 설계 (#1773 open 유지 사유): 파서가 "레코드 전용 구역 컨트롤"을
  플래그로 보존하고 직렬화기가 컨트롤 문자 합성을 생략하는 왕복 계약 — 위험 평가 후 별도 태스크
