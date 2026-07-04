# #1637 근본원인 조사 — HWPX roundtrip IR-invisible 페이지네이션 변동

- 일자: 2026-06-29 / 브랜치: local/task1637 (devel d3627f55)
- 발견 경위: Task #1636 v3 전수 검증의 T-PI(페이지↔PI roundtrip) — HWPX PASS 표본 3,000 중
  23건(0.77%)이 IR diff=0·lineseg diff=0 인데도 저장 후 PI가 다른 페이지로 이동.

## 1. 결론 (요약)

| 원인 | 비중 | 성격 | 상태 |
|------|-----:|------|------|
| **A. `hideFirstEmptyLine` 직렬화 드롭** | 19/23 (83%) | secPr visibility 1→0 | **규명·검증 완료** |
| **B. 표 `flowWithText` 직렬화 드롭** | 4/23 (17%) | table pos 0→1 | **규명·검증 완료** |

두 원인 모두 **IR-invisible**: `diff_documents`(roundtrip 게이트)가 해당 필드를 비교하지 않는다.
HWP5 는 0건 — HWPX serializer 고유.

> 참고: 이 문서는 원인 A 규명 직후의 기술 조사 기록으로 시작되었으나, PR #1642 최종 변경에서
> 잔여 원인 B도 표 `hp:pos@flowWithText` 드롭으로 규명·수정되었다. 최종 검증 수치는
> `mydocs/report/task_m100_1637_report.md` 를 기준으로 한다.

## 2. 원인 A — `hideFirstEmptyLine` 드롭 (지배원인, 83%)

### 메커니즘
- 원본 secPr 의 `<hp:visibility … hideFirstEmptyLine="1" …/>` (첫 빈 줄 숨김)이
  저장 후 **항상 `hideFirstEmptyLine="0"`** 으로 바뀐다.
- → 문서 선두의 빈 줄이 보이게 되어 **본문 전체가 아래로 밀리고**, 페이지 하단 항목이
  다음 페이지로 넘어간다(페이지 확장/PI 이동).

### 3중 사각지대
1. **파서는 읽는다**: `src/parser/hwpx/section.rs:1274`
   `b"hideFirstEmptyLine" => sec_def.hide_empty_line = attr_str(&attr) == "1"`
   → 값은 IR(`SectionDef.hide_empty_line`, `src/model/document.rs:235`)에 보존됨.
2. **렌더러는 쓴다(레이아웃 영향)**: `src/document_core/queries/rendering.rs:1036`
   `set_bit(flags, 0x00080000, sd.hide_empty_line)` (bit 19) 등 — 레이아웃에 반영.
3. **직렬화기는 무시한다**: secPr 는 템플릿 보존 방식(`src/serializer/hwpx/section.rs:4` 주석
   "secPr/pagePr/grid 등 섹션 정의는 템플릿 보존"). visibility 요소는
   `src/serializer/hwpx/templates/empty_section0.xml` 의 **정적 문자열**
   (`hideFirstEmptyLine="0"`)로 그대로 방출되며, `sec_def.hide_empty_line` 로 치환하는 코드가 없다
   (grep `hideFirstEmptyLine`/`visibility` in serializer → 치환부 0건).
   기본 상수 `src/serializer/hwpx/canonical_defaults.rs:60`
   `VISIBILITY_HIDE_FIRST_EMPTY_LINE: bool = false`.
4. **게이트는 안 본다**: `diff_documents` 가 `hide_empty_line` 미비교 → IR diff=0.

### 검증 (XML 단일필드 revert)
`zip` 미설치 환경 → `output/poc/fidelity3/_repack.py`(python zipfile, mimetype 무압축 선두)로 재포장.

| 파일 | orig hfel | orig/rt page0 items | rt 에서 hfel 0→1 revert |
|------|:---:|:---:|:---:|
| 36374808(농업기술센터 물품검사조서) | 1 | 10 / 8 | **→ 10 (복원)** |
| 36373372(수난구조대 물품검수조서, 한글 EXPAND) | 1 | 11 / 9 | **→ 11 (복원)** |

대조: 무작위 SAME/PASS 60건 전부 `hideFirstEmptyLine="0"`(roundtrip 무변→SAME). shift 23건 중 19건 `="1"`.
다른 secPr 필드(outlineShapeIDRef 0→1·noteLine NONE→SOLID·noteSpacing ±1) revert 는 **무효** = 원인 아님.

### 수정 방향 (PR #1642 반영)
1. **직렬화기**: visibility 의 `hideFirstEmptyLine`(이상적으로 visibility 전 필드: hideFirstHeader/
   Footer/MasterPage/PageNum/showLineNumber 등)를 템플릿 상수 대신 `sec_def` IR 값으로 치환 방출.
   secPr 템플릿 치환부(#1388 page margin·#1388 colPr 동형) 패턴 재사용.
2. **게이트**: `diff_documents` 에 `hide_empty_line`(및 visibility 계열) 비교 추가 → IR-visible 화
   (단위 테스트로 회귀 가드). #1595 ClickHere·#1594 holdAnchorAndSO 와 동형의 "IR-invisible→게이트 편입" 처리.

## 3. 원인 B — 표 `flowWithText` 드롭 (잔여 17%, 규명·수정 완료)

`hideFirstEmptyLine="0"` 인 4건(36390819·36400485·36385069·36384855)은 A 와 무관.
대표 36384855(구로소방서 당직상황근무일지): orig 은 9×4 표(pi11)를 page0 하단에 **1행 partial-split**,
rt 는 split 없이 표 전체를 다음 페이지로. 표 총높이(412.3px)·선행 items used(920.9px) 동일.

- **secPr 원인 아님**: rt body + orig secPr swap → 변화 없음(11 유지).
- 개별 body 필드(lineWrap SQUEEZE→BREAK·empty run `<hp:t/>`→`<hp:t></hp:t>` 32 vs 8·id 재번호) revert 무효.
- 추가 이진탐색 결과 table region 의 `hp:pos@flowWithText` 가 원본 `0`에서 roundtrip `1`로 바뀌는
  지점에 수렴했다.
- **근본원인**: `src/serializer/hwpx/table.rs` 의 table `write_pos`가 `flowWithText="1"`을
  하드코딩해, 원본 treatAsChar 표의 `flow_with_text=false`를 보존하지 못했다.
- **수정**: table 직렬화는 `bool01(c.flow_with_text)`를 사용하고, `diff_documents`에
  `ObjectFlowWithText` 비교를 추가해 표 `flowWithText` 차이를 IR-visible 게이트로 편입했다.
- **검증**: cause B 4건과 A+B 복합 1건이 모두 SAME으로 복원되었고, 최종 검증은
  `mydocs/report/task_m100_1637_report.md`의 수치를 기준으로 한다.

## 4. 산출물·재현 도구

- 검출: `tools/verify_pi_page_roundtrip.py` (#1636)
- 재포장 디버그: `output/poc/fidelity3/_repack.py`
- prevalence 스캔: `output/poc/fidelity3/_scan_hfel.py`
- 재현: `36374808`, `36373372`(원인 A) / `36384855`(원인 B)
