# Task M100 #3931 — Stage 4 최신 devel 통합·광범위·시각 검증

- 날짜: 2026-08-15 KST
- 기준: `upstream/devel` `f8c784235d7c56fb7ed4c72ca8f70762d278a734` (#4763)
- 후보: `task/3931-declared-rowbreak`
- 상태: 자동 검증 및 작업지시자 시각 판정 통과

## #4763 재기준화 결과

#4763은 같은 편람 HWP/HWPX를 한컴 2020 PDF와 같은 383쪽으로 맞추고, 저장 source frame의
terminal tail을 물리 frame 끝까지 확장하는 계약을 추가했다. #3931의 Stage 1~3 여섯 커밋을 이
최신 `devel` 위로 rebase한 뒤에는 페이지 수 383쪽을 보존하면서 질문 7과 질문 11의 저장 fragment
소유권도 함께 유지해야 한다.

첫 통합 후보는 #4763의 terminal response tail이 질문 7의 plain-text 문단 사이 저장 reset까지
건너가 본문 하단을 넘었다. 같은 행에 저장 reset이 하나라도 있으면 tail 확장을 막는 넓은 가드는
질문 7을 고쳤지만, 질문 14의 control-only 문단과 중첩 도형 표가 쓰는 문단 로컬 `vpos=0`까지 물리
쪽 경계로 오인해 전체를 384쪽으로 늘렸다.

최종 구현은 ordinary cut이 **가시 텍스트가 있는 두 control-free 문단 사이 hard break 바로 앞**에서
끝났는지만 `row_cut_ends_at_plain_text_saved_reset`으로 판정한다. 이 cut-local 조건에서만 terminal
response tail 확장을 억제한다. control-bearing 문단의 로컬 reset은 기존 #4763 source-frame 계약을
유지한다. 단위 테스트 `test_plain_text_saved_reset_is_distinct_from_control_local_reset`이 두 구조를
구분한다.

## 직접 문서 결과

한컴 2020 KoPub 설치 PDF, 최신 후보 HWP와 HWPX는 모두 383쪽이다.

- 질문 7(`sec=10`, `pi=14`): HWP는 물리 p284에서 질문과 첫 답변 조각을 시작하고 p285로
  이어진다. HWPX도 질문과 첫 답변 문단을 같은 물리 쪽에 둔다.
- 질문 11(`sec=10`, `pi=23`): 저장 24.5px pitch와 16줄 전체 높이를 유지한 채 물리 p287의
  12줄과 p288의 4줄로 나뉜다.
- 두 HWP 첫 fragment의 Table bbox 하단은 해당 Body bbox 하단 안에 있다.
- 질문 14의 control-only local reset과 중첩 도형 조판은 유지되어 384쪽 회귀가 없다.

## Rust 회귀·플랫폼 검증

- `issue_3931_declared_rowbreak` — 5/5 통과
- `issue_3930_hwpx_hwp_save_layout` — 3/3 통과
- #3738·#874·#2097·#2105·#2439·#3236·#1156·#1748과 같은 위험축 focused 회귀
  16개 integration binary, 57건 — 전건 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests
  --test-threads 12 --no-fail-fast` — 6,023/6,023 통과, 38 skip
- `cargo clippy --all-targets -- -D warnings` — 통과
- Native Skia lib — 58/58 통과
- Native Skia `issue_2225_missing_picture_placeholder` — 2/2 통과
- Native Skia `render_p37_direct_pdf_export` — 4/4 통과
- `cargo fmt --all`, `git diff --check` — 통과

`cargo-nextest` 설치본은 0.9.137이며 저장소 권장 버전 0.9.140 경고가 있었지만 테스트 실패는 없다.

## 비공개 10k 코퍼스 최신 비교

이전 Stage 4의 10k 수치는 #4763 이전 기준이라 폐기하고, `f8c784235` 기준 바이너리와 최신 후보를
같은 비공개 입력 10,000건에 다시 적용했다. 원본·경로·파일명·개별 결과는 외부 증적에 노출하지
않고 `/tmp`에만 둔다.

| 항목 | 기준 | 후보 | 변화 |
| --- | ---: | ---: | ---: |
| 전체 입력 | 10,000 | 10,000 | 0 |
| 성공 / 오류 | 9,948 / 52 | 9,948 / 52 | 전이 0 |
| 성공 집합 공통 | 9,948 | 9,948 | 누락 0 |
| 쪽수 동일 | - | 9,943 | - |
| 쪽수 변화 | - | 5 | 증가 0, 감소 5 |

변화 5건은 모두 HWP5이다. 네 건은 1쪽 감소, 한 건은 5쪽 감소해 합계는 331쪽에서 322쪽으로
9쪽 줄었다. 이 5건을 기준·후보 바이너리로 각각 전 페이지 SVG export했으며 모두 export가
완료됐다. 양쪽 모두 `overflowCellLines=0`이고 신규 overflow 증가는 없다.

## Docker WASM·Studio

- `docker compose --env-file .env.docker run --rm wasm` — 공식 최적화 빌드 통과
- `pkg/rhwp.js` SHA-256:
  `a226ed9d30ba724addbdd6b3539c407e9909e8ae7d8a195f2632bf56b9419c9b`
- `pkg/rhwp_bg.wasm` SHA-256:
  `88b000c77ef864aba317fe1816c6c55f0fd7b09b9e4e17d4399450d102ccbe01`
- 두 파일을 `rhwp-studio/public/`에 적용했고, 7702 전용 Vite 서버가 제공하는 파일의 해시도
  `pkg/`와 각각 일치한다.
- `npm test` — 923건 중 922 통과, 1 skip, 실패 0

`rhwp-studio/public/rhwp.js`는 이번 renderer 수정과 무관한 생성기 차이가 크므로 시각 판정용 로컬
적용 상태만 유지한다. 최종 #3931 소스 커밋에는 포함하지 않는다. `rhwp_bg.wasm`과 `pkg/`는 Git
제외 산출물이다.

## 시각 근거와 판정 지점

최신 후보와 한컴 2020 PDF의 같은 물리 쪽을 96dpi로 비교했다.

| 물리 쪽 | 확인 대상 | review | 자동 일치율 보조값 |
| ---: | --- | --- | ---: |
| 284 | 질문 7 head | `output/3931/visual/current/issue3931-current/review/review_284.png` | 62.59162% |
| 285 | 질문 7 tail | `output/3931/visual/current/issue3931-current/review/review_285.png` | 61.11530% |
| 287 | 질문 11 head 12줄 | `output/3931/visual/current/issue3931-current/review/review_287.png` | 67.22229% |
| 288 | 질문 11 tail 4줄 | `output/3931/visual/current/issue3931-current/review/review_288.png` | 46.20458% |

각 페이지의 `compare/`, `overlay/`, `review/` 파일과 전체 contact sheet는
`output/3931/visual/current/issue3931-current/`에 있다. 자동 수치는 폰트와 다른 누적 조판 차이를
포함한 후보 축소용 보조값이며 최종 판정이 아니다. 작업지시자는 네 review 이미지와 7702 전용
Studio의 최신 Docker WASM 렌더링을 확인하고 시각 판정 통과를 선언했다.
