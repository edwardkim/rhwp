# Task M100 #2072 Stage 24 - manual legacy 문서 분류

## 목표

`mydocs/manual` 아래에서 front matter가 없는 26개 문서를 내용 기준으로 감사한다. 반복 수행 절차,
현재 reference, 역사적 결정·기록, memory provenance를 구분하고 canonical 관계를 명시한다.

## 원칙

- 반복 수행 가능한 현재 절차는 `guide/active`로 분류한다.
- API·규약·기준 자료는 `reference/active`로 분류한다.
- 작성 당시 선택·브랜딩·실험 기록은 현재 사실을 재보증하지 않도록 `historical`로 분류한다.
- `manual/memory`의 피드백 원문은 `memory/historical`로 보존하고 현재 PR 권위 문서에 연결한다.
- task 번호가 제목에 있어도 현재 반복 사용되는 baseline 가이드는 investigation으로 이동하지 않는다.

## 검증 계획

- `mydocs/manual` 전체 front matter 누락 재검사
- 기본 링크·메타데이터·Python 구문·`actionlint`·`git diff --check`
- 개인 경로와 종료 브랜치를 active 문서에서 사용하지 않는지 재검색

## 분류 결과

| 분류 | 문서 | 판단 |
| --- | --- | --- |
| `guide/active` | 웹 관리자 통합, AI 샘플 작성, 브라우저 확장 개발·배포, 대시보드, 진단 도구, E2E/CDP, Command/Undo, HWP5/HWPX baseline, HWPX2HWP probe, 단축키, 로컬 웹서버, opengov, SOLID | 현재 코드·테스트·반복 수행 절차와 연결됨 |
| `reference/active` | 소비자 편집 API, native render tree bridge | 현재 공개 API의 사용·계약을 설명함 |
| `decision/active` | WASM options object 규약 | 새 공개 API를 추가할 때 적용하는 현행 설계 결정임 |
| `decision/historical` | 브랜딩 전략, Hyper-Waterfall 방법론 | 작성 당시의 방향과 배경을 보존하며 현재 작업 절차는 canonical workflow를 따름 |
| `snapshot/historical` | 한·영 로고 프롬프트 | 브랜딩 결정 당시 생성 입력을 보존함 |
| `memory/historical` | PR 본문 한국어, PR 전 CI, PR 생성 승인 피드백 | 피드백 원문을 보존하고 현재 권위는 PR workflow로 연결함 |

## 현행성 보완

- `local_web_server.md`의 개인 저장소 경로와 Docker 전용 WASM 명령을 제거하고 공통
  `wasm-pack build --target web --out-dir pkg` 절차로 맞췄다.
- E2E 두 문서의 WASM 사전 조건도 같은 공통 명령과 개발 환경 가이드를 가리키게 했다.
- HWP5/HWPX baseline 문서의 고정 샘플 수를 제거했다. 샘플은 계속 추가되므로 테스트의 재귀 수집과
  현재 `XFAIL`/`EXCLUDED` 목록을 권위로 삼는다.
- manual 지도에 E2E, 저장 회귀, 편집 API, command/단축키, 품질 지표 진입점을 추가했다.

## 검증 결과

- `mydocs/manual` Markdown front matter 누락: **0개**
- `python3 scripts/check_markdown_links.py`: **375개 문서 통과**
- `python3 scripts/check_document_metadata.py`: 기존 검사 범위 **212개 통과**
- `python3 -m py_compile scripts/check_markdown_links.py scripts/check_document_metadata.py`: 통과
- `actionlint .github/workflows/*.yml`: 오류 없음. 기존 workflow의 shellcheck info만 출력됨
- `git diff --check`: 통과
- 제품 소스와 테스트 코드는 변경하지 않았으므로 Cargo 검증은 수행하지 않았다.
