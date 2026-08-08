# 에이전트 태스크 벤치마크 스위트 1차(r0) — 태스크 정의·기계 오라클·양방향 실증

- **Issue**: [#4220](https://github.com/edwardkim/rhwp/issues/4220) T2 / refs [#3907](https://github.com/edwardkim/rhwp/issues/3907)
- **브랜치**: `task/t2-agent-bench-suite`
- **기준**: `upstream/devel` `dc7d7adcc`
- **작성 시각**: 2026-08-08 KST
- **환경**: Windows 11, `cargo build --release`(기본 feature), Python 3 stdlib

## 1. 무엇을 만들었나 — 1차의 정직한 절단

T2 의 최종 목표는 "표준 HWP 작업 N종을 에이전트가 rhwp 표면만으로 몇 % 완주하는가"의
실측이다. 1차는 그 전제 조건인 **판정 기반**만 착지한다:

- **태스크 정의** — [`tools/agent_bench/tasks.json`](../../tools/agent_bench/tasks.json):
  태스크 10종 각각에 (입력 fixture, 한글 목표 서술, 풀이 인터페이스, 오라클 명령열,
  타임아웃)을 선언한다. fixture 는 전부 `samples/` 실물이다.
- **채점 하니스** — [`tools/agent_bench/run_bench.py`](../../tools/agent_bench/run_bench.py):
  풀이 디렉터리(`<task_id>.py` 모음)를 받아 태스크별로 실행하고, 오라클 명령열을 돌려
  pass/fail 을 기계 판정한 뒤 성공률 표를 낸다. `RHWP_BIN` 으로 바이너리를 주입한다.
- **레퍼런스 풀이** — [`reference_solutions/`](../../tools/agent_bench/reference_solutions/):
  태스크당 1파일, 문서 조작은 rhwp CLI 만 사용. 오라클 판정 가능성의 **양성 증거**.
- **오풀이(음성 대조)** — [`null_solutions/`](../../tools/agent_bench/null_solutions/):
  "입력을 그대로 복사" · "검사 없이 전부 깨끗하다 보고" 류의 그럴듯한 게으른 오답.
  오라클이 이를 전부 걸러내는 것이 **음성 증거**다.

**살아있는 LLM 호출은 하지 않았다.** 이 문서의 수치는 오라클 검증(판정기가 옳은
풀이와 그른 풀이를 가르는가)이지 에이전트 성공률이 아니다 — 성공률 실측은 후속이다.

판정을 LLM 심판이 아니라 기계 오라클로 고정한 것은 T2 착수 게이트 ①(자기 저작 검증
함정 회피)의 이행이다. 태스크 원천은
[`agent_task_playbook.md`](../manual/agent_task_playbook.md)의 35 시나리오와 공무원
업무 체크리스트 21행에서 축이 겹치지 않게 10종을 골랐다.

## 2. 태스크 10종

| id | 축 | fixture (samples/) | 오라클 판정(요지) |
|---|---|---|---|
| t01_form_fill | 서식 채우기 | field-01.hwp | `fields --json` 재독 — 4개 필드 값 일치 |
| t02_table_roundtrip | 표 CSV 왕복 | 2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | `table-to-csv` 재독 — 목표 칸만 변경, 나머지 35칸 보존 |
| t03_redact | 레닥션 | field-01.hwp + CLI 로 합성한 가공 PII | `edit redact --dry-run` 잔여 0 + 원문 문자열 부재 + 내용·쪽수 보존 |
| t04_convert_hwpx | 형식 변환 | 20250130-hongbo.hwp | `info` format/쪽수 + `export-text` 전문 유사도 ≥ 0.999 |
| t05_replace_preserve_pages | 쪽수 보존 치환 | 20250130-hongbo.hwp | `search` 이전 0건·신규 6건 + `info` 쪽수 보존 |
| t06_visual_safe_edit | 시각 무회귀 편집 | 기부·답례품 양식.hwpx | 편집 반영 + 비편집 쪽(page 0) SVG **바이트 동일** |
| t07_text_extract | 텍스트 추출 | 2022년 국립국어원 업무계획.hwp | 자기일관 유사도 ≥ 0.99 + 외부 앵커 3종 포함 |
| t08_structure_report | 구조 파악 | 2025 행정업무운영 편람(최종).hwpx | `export-structure` 대조 — mode·nodeCount·최상위 수 |
| t09_bulk_aggregate | 대량 처리 집계 | 혼합 5건(hwp5 4·hwp3 1) | `info` 대조 — 파일별 쪽수 + 합계 정합 |
| t10_security_sweep | 보안 스윕 판정 | 더러운 2건 + 깨끗한 2건 | `inspect` 3축 × 4건 = 12 판정 전수 대조 |

- 오라클 명령은 전부 devel 에 실존하는 표면만 쓴다: `fields`·`table-to-csv`·
  `edit redact --dry-run`·`info`·`export-text`·`search`·`export-svg`·
  `export-structure`·`inspect hidden-text|unicode|injection`. 미머지 PR 의
  `verify --expect`(#4186)·`scan`(#4217)은 쓰지 않았다 — merge 후 오라클을 verify 로
  치환/보강할 수 있다(§5).
- t10 의 "더러운" fixture 는 합성이 아니라 **samples 실물**이다 —
  `issue1892_hwp3_tab_roundtrip.hwp`(흰 글씨 은닉 텍스트)와
  `정책연구용역사업 중간진도보고서(…).hwp`(U+200B 제로폭)는
  `tests/security_corpus_regression.rs` 가 개별 근거와 함께 "탐지가 옳다"고 선언한
  문서들이다.
- t03 의 가공 PII(주민번호·전화·이메일)는 `tests/redact_sanitize_contract.rs` 와 같은
  값(검증숫자 통과·실재 무관)을 하니스 setup 단계가 `edit fill-fields` 로 심는다 —
  fixture 합성까지 CLI 로만 한다.

## 3. 양방향 실증 — 실측 수치

같은 하니스·같은 바이너리(`devel dc7d7adcc` release)·같은 머신에서 두 번 실행했다.

### 3-1. 레퍼런스 풀이 → **10/10 PASS** (exit 0)

```
t01_form_fill               PASS            0.8s
t02_table_roundtrip         PASS            1.9s
t03_redact                  PASS           29.4s
t04_convert_hwpx            PASS            0.9s
t05_replace_preserve_pages  PASS            0.9s
t06_visual_safe_edit        PASS            0.8s
t07_text_extract            PASS            0.9s
t08_structure_report        PASS            1.4s
t09_bulk_aggregate          PASS            1.2s
t10_security_sweep          PASS            9.8s
성공률: 10/10   (전체 소요 약 48s)
```

### 3-2. 오풀이 → **0/10** (exit 1), 전부 의미 있는 검사에서 절단

```
t01_form_fill               [회사명] value = '' (기대 '한국수자원공사')
t02_table_roundtrip         [목표 칸 값] (1,1) = '' (기대 '1,234')
t03_redact                  [잔여 탐지 0] findingCount = 3 (기대 0)
t04_convert_hwpx            [형식] format = 'hwp5' (기대 'hwpx')   ← 확장자 사칭을 내용 스니핑이 잡음
t05_replace_preserve_pages  [이전 문자열 잔존 0] matchCount = 6 (기대 0)
t06_visual_safe_edit        [편집 반영] (2,1) = '' (기대 '7,777')
t07_text_extract            [전문 유사도] 0.0000 (기대 ≥ 0.99)
t08_structure_report        [mode] 'outline' (기대 'clause')
t09_bulk_aggregate          [hongbo 쪽수] 1 (기대 4)
t10_security_sweep          [issue1892 hidden] 전부-깨끗 보고가 실제 판정(clean=false)과 불일치
성공률: 0/10
```

### 3-3. 종료 코드 규약 실측

| 상황 | exit |
|---|---|
| 전 태스크 pass (3-1) | 0 |
| 하나라도 fail — 오풀이(3-2)·풀이 파일 없음(NO_SOLUTION) | 1 |
| 구성 오류 — `RHWP_BIN` 경로 부재 | 2 |

## 4. 오라클 설계 노트 (재현에 필요한 실측 사실)

- **SVG 렌더는 결정적이다** — 같은 바이너리로 같은 입력의 page 0 을 두 번 내보내면
  바이트 동일(실측 23,726B). t06 의 "비편집 쪽 바이트 동일" 게이트가 이 위에 선다.
  표 12 편집의 `changedPages` 는 `[6,7]` 로 page 0 과 겹치지 않는다.
- **`export-text` 의 전문은 `--json` 봉투로 받아야 한다** — 무봉투 stdout 은 전문
  계약이 아니다(35쪽 문서 실측: 봉투 33,719자 vs 무봉투 1,341자). 하니스는
  `pages[].text` 이어붙이기를 스텝 옵션(`extractPagesTextTo`)으로 내장했다.
- **t07 의 외부 ground truth 는 부분 앵커다** — 동봉 `samples/2022년 국립국어원
  업무계획.txt` 는 1,214자 발췌본이고 표 평탄화 방식이 달라 전문 대조가 안 된다
  (정규화 후 29줄 중 17줄만 포함, 미포함은 전부 표 행). 그래서 판정은 도구 자기일관
  유사도 + 발췌본에서 확인된 앵커 3종 포함으로 구성했다.
- **판정이 봉투 안에만 있는 명령**(playbook §원칙)을 그대로 계승했다 — redact 는
  `findingCount`, inspect 는 `clean` 이 게이트고 exit 는 0 이다. 오라클 검사가 그
  자리를 짚는다.

## 5. 한계와 후속

1. **에이전트 성공률이 아니다.** 본 실증은 "오라클이 판정 가능하다"까지다. 실제
   LLM 에이전트를 태스크 서술만 주고 N 회 돌려 성공률을 재는 것(실행 주체·비용
   판단 포함)은 T2 착수 게이트 ②로, 메인테이너 판단 사항이다.
2. **오라클은 rhwp 자신의 읽기 표면을 신뢰한다** — 재독 대조 원칙의 의도적 채택.
   읽기 표면 자체의 회귀는 이 하니스가 아니라 기존 계약 테스트·시각 게이트 몫이다.
   독립 오라클(한/글) 교차 검증은 범위 밖.
3. **t07 자기일관 판정은 추출기 회귀를 못 잡는다** — §4 의 앵커 3종만이 외부 근거다.
4. **verify 게이트(#4186) merge 후** — t04·t02 류의 쪽수·표 기대는 `verify --expect`
   1회 호출로 치환 가능하다. 오라클 명령열만 바꾸면 되도록 tasks.json 에 선언으로
   분리해 두었다.
5. **단일 머신·단일 실행 실측**이다. 시간 수치는 상대 참고용.
