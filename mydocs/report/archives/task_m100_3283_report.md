# task_m100_3283 처리결과 보고서 — `search` 주소를 가진 문서 검색

- **이슈**: [#3283](https://github.com/edwardkim/rhwp/issues/3283)
- **브랜치**: `pr/task-search-json` (**upstream/devel 직분기 — 열린 PR 과 공유 커밋 없음**)
- **범위**: `src/document_core/queries/grep.rs`(신규), `queries/mod.rs`(가시성 2행),
  `queries/search_query.rs`(공개 래퍼 1개), `src/main.rs`(명령 1개·디스패치 1행·help),
  `tests/search_json_contract.rs`(신규), `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (읽기 전용 질의)

## 1. 문제

에이전트가 문서에서 무언가를 찾는 유일한 길은 전체 텍스트를 뽑아 외부에서 검색하는 것이었고,
그 순간 **주소가 소멸**했다. 393쪽 편람에서 "이 규정 어디 있나"에 대해 10MB 텍스트를
컨텍스트에 밀어넣거나 "어딘가에 있습니다"라고 답할 수밖에 없다 — **근거 제시가 성립하지 않는다.**

## 2. 분석 — 설계 결정

- **엔진은 이미 있다.** `search_query::search_all` 이 본문·표 셀·글상자를 순회한다.
  같은 매칭 규칙을 쓰도록 `find_matches` 래퍼를 크레이트에 공개했다 — 검색과 치환이 다른
  규칙을 쓰면 "찾았는데 못 바꾸는" 어긋남이 생긴다.
- **페이지 매핑 비용은 0이다.** `DocumentCore::from_bytes` 가 로드 시 `paginate()` 를 끝내므로
  순수 조회다. 다만 `find_pages_for_paragraph` 를 매치마다 부르면 O(N × 페이지 아이템)이라,
  `(구역,문단) → 페이지` **인덱스를 한 번만** 만들어 재사용한다.
- **첫 등장 쪽을 쓴다.** 한 문단이 여러 쪽에 걸치면 인용 기준은 시작 위치다.
- **0건은 성공이다.** #2707 계약에서 1은 런타임 실패 전용이므로 `matchCount:0` + exit 0.
  grep 관례(0건=1)를 따르면 계약이 깨진다.
- `--limit` 은 대형 문서에서 에이전트 컨텍스트를 아끼기 위한 상한이다.

## 3. 변경

- `queries/grep.rs` 신설 — `GrepMatch`/`CellRef`(serde) + `DocumentCore::grep()`
- `search_query` 에 `find_matches` 공개 래퍼, `queries/mod.rs` 가시성 2행
- `search` 명령: `--json` 봉투 / `--ignore-case` / `--limit` / 기본은 사람용 요약
- `cli_commands.md` 신설 항목 (VLM 확인 레시피 포함)

## 4. 검증

- **계약 테스트 9종 red→green**: 봉투·주소 필드 / **페이지가 문서 페이지 범위 안** /
  표 셀 매치 좌표 / **0건은 exit 0** / `--limit` 상한 / `--ignore-case` 반영 /
  기본 출력 비-JSON 가드 / 종료 코드(없는 파일 1·검색어 없음 2)
- 테스트는 **미머지 기능에 의존하지 않는다** — 페이지 수를 `info` 사람용 출력에서 얻고
  검색어를 고정해, devel 단독에서 통과한다.
- `cargo clippy --release --bin rhwp -- -D warnings` 0, `rustfmt` clean, 문서 검사 2종 clean
- **실측**: `2025 행정업무운영 편람(최종).hwp`(393쪽·10MB)에서 "위임전결" 19건을
  **215ms**(파싱+조판 포함)에 찾고, 쪽 번호 35·78·81·104·106·107·291·296… 을 반환.

## 5. 남긴 것

- 정규식은 넣지 않았다 — `regex` 의존성 추가는 별도 판단이 필요하고, 부분 문자열 매칭으로
  에이전트 시나리오 대부분이 커버된다.
- `batch search`(코퍼스 전역 검색)는 batch 축을 건드리는 다른 PR 과 겹치므로 후속.
- 머리말/꼬리말·각주/미주 안의 텍스트는 `search_all` 범위 밖이다(기존 엔진 범위 그대로).
