# PR #2166 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2166`
- 제목: `fix: 표 셀 반복 Enter 캐럿 위치 정정`
- branch: `task_m100_2164`
- base: `devel`
- 관련 이슈: #2164

## 커밋 구성

```text
569ae5e1a task 2164: 표 셀 Enter 문단 좌표 재계산
7b895dcbe task 2164: 반복 셀 Enter 캐럿 위치 정정
0f92d2e3a docs: task 2164 사전 검증 증적 추가
```

## 처리 단계

1. 실제 제보 HWP의 대상 셀을 구조로 탐색해 문단 분할 후 vpos 겹침을 재현했다.
2. 셀 문단 재계산과 빈 문단 높이 처리를 보정하고 첫 Enter/병합 후 재Enter 회귀를 추가했다.
3. 반복 Enter에서 새 문단 임시 vpos 원점이 RowBreak로 오인되는 경로와, 프론트
   `cellPath`가 이전 문단을 가리키는 경로를 각각 정정했다.
4. HWP 2020 기준 PDF와 visual sweep 증적을 생성하고 macOS 전체 사전 검증을 수행했다.
5. source PR #2166 merge 뒤 본 review 문서와 증적을 docs-only PR로 분리한다.

## 주요 검증 명령

```bash
CARGO_INCREMENTAL=0 cargo build --release
CARGO_INCREMENTAL=0 cargo test --release --lib
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
cargo fmt --check
git diff --check
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
CARGO_INCREMENTAL=0 cargo test --doc
cd rhwp-studio && npx tsc --noEmit && npm test && npm run build
cd rhwp-chrome && npm run build
wasm-pack build --target web --out-dir pkg
```

## 시각 검증 명령

```bash
hwp2020-mcp-convert --input samples/issue2164/의견제출서(양식).hwp --target pdf
python3 scripts/task1274_visual_sweep.py \
  --key issue2164 \
  --hwp samples/issue2164/의견제출서(양식).hwp \
  --pdf pdf/issue2164/의견제출서(양식)-2020.pdf \
  --out mydocs/report/assets/task_m100_2164_visual_sweep
```

실제 실행은 로컬 인증 환경의 CLI package와 `.env.local`을 사용했다. URL/IP와 인증 토큰은
공개 기록에 포함하지 않는다.

## 후속

1. PR #2166은 CI/CodeQL/Render Diff 통과 뒤 merge commit `f3712542e980b1eab625624f54eb49e789ddfc5d`로 반영됐다.
2. #2164는 GitHub closing keyword로 CLOSED 상태를 확인했다.
3. 본 문서, 기준 PDF, visual sweep asset, 오늘할일을 docs-only fast-pass PR로 반영한다.
4. 후속 PR merge 뒤 source/docs 브랜치와 원격 추적 브랜치를 정리한다.
