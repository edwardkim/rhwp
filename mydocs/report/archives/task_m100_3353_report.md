# task_m100_3353 처리결과 보고서 — `search --limit` 절단 가시성

- **이슈**: [#3353](https://github.com/edwardkim/rhwp/issues/3353)
- **브랜치**: `pr/fix-issue-3353-search-limit-truncation` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/main.rs`(`search_document` 결과 집계부 + capabilities recordFields),
  `tests/issue_3353_search_limit_truncation.rs`(신규), `mydocs/manual/cli_commands.md`(2줄)
- **분류**: 버그 수정 (CLI JSON 계약 — 절단 은폐)

## 1. 배경

`search --limit N` 은 도움말이 "컨텍스트 절약용"으로 권하는 옵션인데, 이걸 쓰면
`matchCount` 가 절단된 수를 보고하고 절단 표시가 없어 **문서 전체 매치 수를 알 수 없다.**
v0.8.0 실측: 316건 문서에서 `--limit 3` → `{"matchCount":3}` — "정확히 3건"과
"3건만 표시(실제 316건)"를 에이전트가 구별할 방법이 없다. 컨텍스트를 아끼려는
에이전트일수록 `--limit` 을 쓰므로, 신중한 소비자가 더 크게 속는 구조다.

## 2. 설계 결정

- **`matchCount` 의미 보존** — 종전대로 반환된 매치 수(= `matches.len()`)다. 스키마
  정책(필드 추가만 허용, 변경·삭제 금지)과 기존 소비자(`matchCount > 0` 게이트,
  `search_limit_caps_result_count` 계약 테스트)를 깨지 않는다.
- **추가-전용 2필드** — `totalMatchCount`(문서 전체 매치 수), `truncated`(절단 여부).
  에이전트 재질의 판단 근거가 봉투 안에 생긴다.
- **전수 grep 후 표시만 절단** — 총량 보고에는 전수 스캔이 불가피하다. `--limit` 의
  목적은 스캔 시간이 아니라 **출력 컨텍스트** 절약이므로 목적을 해치지 않는다
  (스캔 비용은 limit 유무와 무관하게 문서 크기에 비례. 코어 `grep` 시그니처 무변경).
- **비-JSON 출력도 동일 원칙** — 절단 시 `— 316건 중 3건 표시 (--limit)`.
- **`capabilities` recordFields 동기화** — 자기서술만 보고 소비하는 에이전트가 새 필드를
  발견할 수 있게 등재.

## 3. 변경

- `search_document()` — `grep(…, limit)` → `grep(…, None)` 전수 스캔 후 `take(n)` 절단,
  `totalMatchCount`·`truncated` 봉투 추가, 절단 시 사람용 출력 문구 변경.
- capabilities 의 search `recordFields` 에 `totalMatchCount`·`truncated` 등재.
- `cli_commands.md` — search 봉투·`--limit` 항목 갱신.

## 4. 검증

- **회귀 테스트 4종** (`tests/issue_3353_search_limit_truncation.rs`, red→green):
  절단 시 totalMatchCount=전체·truncated=true / 미절단 시 totalMatchCount==matchCount·
  truncated=false / limit ≥ 전체면 truncated=false / 비-JSON 절단 안내 문구
- **무회귀**: `search_json_contract`(기존 계약 — `matchCount ≤ limit` 포함),
  `cli_json_contract` 전부 green (release-test 프로필)
- `cargo fmt --all -- --check` clean, clippy `--bin rhwp -- -D warnings` 0건
- **실측 전/후**: 전 = v0.8.0 릴리스 바이너리, 후 = 본 브랜치 빌드 — PR 본문 수록

## 5. 남긴 것

- #3347 의 `batch search` 는 파일당 1,000건 상한을 두므로 같은 가시성이 필요하다.
  #3347 이 봉투 빌더(`search_json_value`)를 추출하므로, 머지 순서에 따라 이 필드들을
  빌더로 옮기는 리베이스(또는 빌더 위 얹기)로 정리한다 — #3353 본문에 기록됨.
