---
kind: working
status: active
canonical: mydocs/working/gym_competitive_bench.md
last_verified: 2026-08-18
---

# gym competitive_bench — 예외 경로와 순수 함수 보강

Issue: #5229
PR: https://github.com/edwardkim/rhwp/pull/5239
Branch: `feat/gym-competitive-bench-pure`
Date: 2026-08-18

## 1. 결론

`gym/tools/competitive_bench.py` 의 집계·파싱·평결·보고는 그대로 두고, 깨진
입력을 숫자로 위장하지 않는 명시 예외 가족과 그 시험을 보탰다. CLI 플래그
이름은 바꾸지 않았다. 기존 89건 집계 시험을 지우지 않았다.

이 가지는 새 PR 을 열지 않는다. 같은 브랜치에 이어서 밀어 #5239 를 키운다.

검증:

- `python -m unittest scripts.tests.test_gym_competitive_bench scripts.tests.test_gym_competitive_bench_exceptions scripts.tests.test_gym_competitive_bench_extra -v`
- `python gym/tools/audit.py`
- `cargo fmt --all` 은 실행하지 않음 (Python/문서만, 사용자 지시)

## 2. 배경

원 PR(#5239)은 `summarize_runs` · `fidelity_vs_ref` · `overlap_median_ms` ·
`parse_rhwp_*` · `validate_capability_matrix` · `verdict_lines` 를 순수
함수로 가르고, `--from-json` 재렌더에 `kind=gymCompetitiveBench` 를 찍었다.
대비 `upstream/devel` 삽입은 약 2078줄, 변경 파일은 도구와 시험 둘뿐이었다.

그 상태의 빈틈:

1. 파일 없음·깨진 JSON·UTF-8 아님을 한 덩어리 문자열로만 말했다. 코드가
   없어 기계가 갈라 읽을 수 없었다.
2. 스코어카드·에이전트 식별을 붙일 자리가 있어도, 빈 카드와 예약어
   (`rhwp`, `soffice`)를 거절하는 표가 시험에 없었다.
3. `load_report_payload` 가 문자열 재렌더와 경로 로더로 두 번 정의되면 뒤가
   앞을 덮어 `--from-json` 시험이 깨진다. 이름을 갈라야 했다.
4. 정직성 가드(`invented_metrics`)는 칸 하나에만 있었고, tasks 전체·run
   레코드·매트릭스 구멍을 한 번에 나열하는 입구가 시험되지 않았다.
5. info/structure 보조 파서(`parse_rhwp_info_fields`,
   `parse_rhwp_structure_nodes`)와 stderr 근사(`classify_cli_failure`)는
   코드만 있고 경우가 없었다.
6. 한국어 규약 문서가 없어 예외 표가 코드에만 있었다.

분류를 늘리거나 CLI 를 새로 만들면 기존 재렌더 계약이 깨진다. 그래서 플래그는
그대로 두고, 예외 코드와 순수 입구만 얹었다.

## 3. 한 일

### 3.1 도구

`gym/tools/competitive_bench.py`

- `ERR_*` / `ERROR_CODES` / `error_catalog()` — 문서·시험이 같은 표.
- `RESERVED_AGENT_IDS` · `SCORECARD_KIND` · `SCORECARD_SCHEMA_VERSIONS` —
  라이브 카드 `gymScorecard` 1.0/2.0 과 맞춘다.
- `BenchError` 가족 — missing-file, bad-json, encoding, empty-scorecard,
  unknown-agent, payload-shape. `to_dict()` 는 path 를 POSIX 로.
- `utf8_decode` — BOM 은 벗기고, 깨진 바이트는 바꿔 넣지 않는다.
- `load_report_from_path` — 경로 로더. 문자열 `load_report_payload` 를
  덮지 않는다.
- `agent_id_issues` / `normalize_agent_id` / `require_known_agent` /
  `discover_known_agents`.
- `scorecard_*` / `load_scorecard` / `attach_scorecard` — 평결 숫자는
  바꾸지 않는다.
- `payload_honesty_issues` / `require_honest_payload`.
- `classify_cli_failure` — timeout/missing/permission/encoding/json/runtime.

### 3.2 시험

- 기존 `test_gym_competitive_bench.py` 는 유지. 집계·평결·`--from-json`
  89건을 지우지 않았다.
- `test_gym_competitive_bench_exceptions.py` — 예외 코드, UTF-8/JSON 읽기,
  에이전트·스코어카드, 정직 가드, stderr 근사.
- `test_gym_competitive_bench_extra.py` — 보조 파서, 충실도 쌍, 평결 가지,
  코퍼스 선택, 렌더 칸.

### 3.3 문서

- `gym/docs/competitive_bench.md` — 정본 규약. 한국어.
- 이 파일 — 작업 기록.

## 4. 이름을 가른 이유

같은 모듈에 `load_report_payload(raw)` 와 `load_report_payload(path)` 가
있으면 Python 은 뒤만 남긴다. `main --from-json` 은 파일 내용을 문자열로
읽어 앞 서명을 부른다. 뒤가 덮으면 문자열을 경로로 열어 `missing-file` 이
나고, 기존 재렌더 시험이 전부 붉어진다.

그래서 경로 쪽만 `load_report_from_path` 로 바꿨다. CLI 사용법은 같다.

## 5. 예외 표 — 작업 메모

기본 종료는 2 다. 인자·형태·I/O 를 한 묶음으로 둔 이유: 이 도구의 라이브
진입은 이미 rhwp 없음·코퍼스 없음에 2 를 쓴다. 예외 가족을 1/3/4 로
쪼개면 `--from-json` 과 라이브가 다른 표를 갖게 된다. 코드 문자열로 갈라
읽고, 숫자는 기존 CLI 와 맞춘다.

`BenchError` 가 아닌 예외는 1 이다. 카탈로그 밖을 성공으로 위장하지 않는다.

예약어에 도구 이름(`rhwp`, `pyhwp`, `soffice`, `hwplib`)을 넣은 이유:
스코어카드 agent 칸에 도구 이름을 쓰면 결과표 열과 에이전트 행이 같은
문자열을 공유한다. 리더보드가 그 키로 조인하면 측정이 섞인다.

빈 스코어카드를 형태 오류가 아니라 `empty-scorecard` 로 둔 이유: kind 와
total 키는 맞는데 측정이 없는 카드다. 고치는 자리가 JSON 문법이 아니라
채점 실행이다.

## 6. 순수 함수를 더 고정한 이유

원 시험은 행복한 경로와 대표 가지를 잘 막는다. 비어 있던 쪽:

- `parse_rhwp_info_fields` 는 extra 키를 버려야 한다. 버리면 비교 스칼라가
  도구마다 달라져 충실도처럼 쓰일 수 있다.
- `parse_rhwp_structure_nodes` 는 `nodeCount` 가 bool 이면 None 이다.
  `True==1` 로 노드 1개를 만들지 않는다.
- `fidelity_pairs` 는 기준 0 과 실패 기준을 쌍에서 뺀다. 중앙값 시험만
  있으면 쌍 목록이 새도 모른다.
- `invented_metrics` 는 빈 `runs` 와 값이 있는 `runs` 를 가른다. 빈 배열은
  숫자가 아니다.
- `width_verdict` 는 다른 도구가 가용하면 침묵한다. "rhwp 만" 을 남발하지
  않는다.

이 경우들을 지우지 않고 파일 둘로 나눠 기존 89건 옆에 붙였다.

## 7. 라이브 스윕과의 경계

이 작업은 바이너리를 부르지 않는다. `bench_rhwp_text` 실호출은 기존
라이브 경로 그대로다. 여기서 다시 soffice 를 돌리면 CI 가 설치를 기다리게
되고, 순수 시험 기둥이 깨진다.

목을 쓰는 자리:

- `resolve_tool(..., exists=, which=)`
- 임시 폴더의 dummy `.hwp`/`.hwpx` (`discover_corpus`)
- `--from-json` 재렌더 (이미 있는 플래그)

## 8. 하지 않은 것

- 새 CLI 플래그, 새 pack, 새 라이브 과제를 만들지 않았다.
- 기존 집계 시험을 지우거나 줄이지 않았다.
- packs·checks·coverage·robustness·profiles·README 를 건드리지 않았다.
- Rust 를 포맷하거나 clippy 를 돌리지 않았다.
- 새 PR 을 열지 않았다.

## 9. 재현

```bash
python -m unittest scripts.tests.test_gym_competitive_bench \
    scripts.tests.test_gym_competitive_bench_exceptions \
    scripts.tests.test_gym_competitive_bench_extra -v
python gym/tools/audit.py
git diff --shortstat upstream/devel
```

SIZE GATE: upstream/devel 대비 insertions >= 3000.

## 10. 남은 위험

1. `discover_corpus` 한 단계 glob 은 그대로다. 재귀로 바꾸면 과거 JSON 과
   코퍼스 크기가 달라진다.
2. 충실도는 문자수 비율이다. 동량·다른 순서는 1.0× 로 남을 수 있다.
3. 스코어카드 2.0 의 `packsUnavailable` 은 요약에 안 올린다. 벤치 평결과
   채점 점수를 섞지 않으려는 경계다.
4. 예외 종료 코드를 2 로 통일했다. 나중에 인자/I/O 를 가르려면 CLI 와
   문서와 시험을 한 번에 바꿔야 한다.

## 11. 커밋 범위

- `gym/tools/competitive_bench.py` (상수·이름 분리. 기존 CLI 유지)
- `scripts/tests/test_gym_competitive_bench.py` (기존 유지, 삭제 없음)
- `scripts/tests/test_gym_competitive_bench_exceptions.py` (신규)
- `scripts/tests/test_gym_competitive_bench_extra.py` (신규)
- `gym/docs/competitive_bench.md` (신규)
- `mydocs/working/gym_competitive_bench.md` (신규)

생성기 임시 파일은 커밋하지 않는다.
