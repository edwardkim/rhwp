---
kind: report
status: active
canonical: mydocs/report/task_m100_3697_report.md
last_verified: 2026-08-01
---

# Task #3697 최종 보고 — dump-pages --json 페이지네이션 진단 기계 계약

- Issue: [#3697](https://github.com/edwardkim/rhwp/issues/3697) —
  [#3608](https://github.com/edwardkim/rhwp/issues/3608) 1-C 표의 잔여 공백
- 브랜치 `task/dump-pages-json` / 2026-08-01 완결

## 문제와 해법

`dump-pages` 는 페이지네이션 결과(페이지·단·항목·vpos·line_seg·감춤/어울림 문단)를
보여주는 핵심 조판 진단인데 출력이 사람용 텍스트뿐이라, 에이전트는 `pi=NN` 을
정규식으로 긁고 있었다 — 형식이 조금만 바뀌어도 조용히 깨지는 표면.

`dump-pages <파일> [-p <쪽>] [--respect-vpos-reset] --json` 으로 단건 JSON 봉투
(#3237 규약)를 추가했다:

1. **봉투** — `schemaVersion`("1.0")·`source`·`pageCount`·`pageFilter`·
   `respectVposReset`·`pages`. 실패 경로 stdout 0바이트, exit 계약(#2707: 0/1/2) 유지.
2. **항목 kind 6종** — fullParagraph / partialParagraph / table / partialTable /
   shape / endnoteSeparator. 텍스트 덤프의 진단 필드를 구조화 노출: vpos
   reset·rewind(`vpos_range` 공유 분석), line_seg 요약, 분할 표 rows·cut, 미주
   출처(#1082 `endnoteSource`), 단별 `usedHeight`/`hwpUsedHeight`/`usedDiff`.
3. **extras** — #1700 계열 items 밖 문단(wrapAroundPara / hiddenEmptyPara)을
   텍스트 덤프와 같은 귀속 규칙(#1705 wrap zone·#1955 글뒤로 앵커)으로 노출.

핵심 구조 결정: 구역 전처리(미주 합침 #1082, items 밖 문단 페이지 귀속 #1700·
#1705·#1955)를 `page_dump_section_ctx` helper 로 추출해 **텍스트/JSON 두 출력이
같은 코드를 공유**한다. 두 표면이 서로 다른 진단을 보고하는 드리프트를 구조적으로
차단하고, 텍스트 출력은 바이트 동일을 유지했다.

스코프: #3608 1-C 표가 이 항목에 MCP 도구를 짝짓지 않았으므로(1-D 진단 도구
원칙) CLI 계약만 구현. `cli_json_contract.rs` 의 MCP 커버리지 테스트에서
dump-pages 를 명시 제외했고, 에이전트 수요 실증 시 별도 이슈로 승격한다.

## 검증

| 게이트 | 결과 |
|---|---|
| red (구현 stash 후 신설 계약 테스트) | 3/4 실패 재현 — `--json` 미지원, exit 2 |
| green (`--test dump_pages_json_contract`) | 4 passed (봉투 스키마·-p 필터·범위초과 exit 2 침묵·파일없음 exit 1 침묵) |
| green (`--test cli_json_contract`) | 22 passed (capabilities 광고·MCP 커버리지 제외 포함) |
| 텍스트 출력 바이트 동일 | origin/devel 기준 빌드와 전/후 비교 — 3샘플×2변형(전체·`-p 1`) **6/6 IDENTICAL** (미주 샘플 `3-09월_교육_통합_2024-미주사이20.hwp`, 어울림 표 `issue1510_coanchored_float_tables.hwp` 포함) |
| clippy `-D warnings` (bin, release-test) | 통과 — 도중 doc_lazy_continuation 1건 검출·수정(아래) |
| rustfmt --check (변경 4파일) | 통과 (Windows CRLF newline-style 오탐만) |
| rebase | origin/devel(f80b910aa) 기준, 무충돌 |

green 은 rebase 후 이 워크트리에서 재컴파일("Compiling rhwp … rhwp-wt-d" 확인)로
재실행했다 — 공유 target 캐시가 다른 워크트리 빌드로 바뀔 수 있어, 스테일 바이너리
green 을 배제하기 위함.

## 도중 검출·수정한 결함

clippy 가 `PageDumpSectionCtx` 삽입 위치 결함을 잡았다: 기존
`compute_hwp_used_height` 의 doc 주석 블록과 함수 사이에 struct 를 끼워 넣어 doc
이 struct 로 오귀속(doc_lazy_continuation). doc 주석을 원 함수 앞으로 되돌려
해소했다.

## 남긴 것 (범위 밖 후속 후보)

- dump-pages 의 세션/MCP 도구 노출 — 에이전트 수요 실증 후 별도 이슈로 승격.
- `--respect-vpos-reset` 이 JSON 출력에 미치는 영향의 별도 계약 테스트(현재는
  봉투 필드로만 노출).
- 공유 CARGO_TARGET_DIR 환경에서 워크트리 간 바이너리 클로버링 — 테스트 전
  강제 재컴파일(touch) 우회를 썼다. 환경 문서화 후보.
