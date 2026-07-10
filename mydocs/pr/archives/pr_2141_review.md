# PR #2141 검토 - Issue #2136 near-top reset 상한 2500HU

- PR: https://github.com/edwardkim/rhwp/pull/2141
- 작성자: `planet6897`
- base: `devel`
- 원 head: `0734589bf635316ac401d374eff55ba0e517009d`
- 체리픽 커밋: `443642354`
- 포함 README: `samples/task2136/README.md`

## 결론

체리픽 자체는 충돌 없이 적용되고 신규 테스트는 통과했다. `native_near_top_reset`
상한을 2000HU에서 2500HU로 넓히는 변경은 저장 `vpos=sb=2500HU` 문단을 새 쪽
시작으로 보는 목적과 맞다.

다만 MCP로 생성한 HWP 2020 기준 PDF는 합성 fixture를 1쪽으로 출력했다. 따라서
`samples/task2136/neartop_reset_sb2500.hwpx`는 한컴 기준 PDF를 직접 재현하는 fixture가
아니라, PR이 설명한 실문서 계열을 축약한 내부 회귀 fixture로 해석해야 한다.

## README 반영

README는 합성 샘플의 전제를 명확히 적고 있다.

- 출처: `samples/tac-host-spacing.hwpx` 골격 수작업 합성
- 실문서 재현원: hwpdocs `148753276_제3회연구노트확산세미나...hwp`
- 기대: `vpos=2500HU == sb`를 near-top reset으로 인식해 2쪽

README에 있는 “한글 정합” 표현은 이번 MCP PDF 결과와는 직접 일치하지 않는다.

## 검증

- `gh pr edit 2141 --add-reviewer jangster77` 수행
- `git cherry-pick -x 0734589bf635316ac401d374eff55ba0e517009d` 성공
- `target/release-test/rhwp dump-pages samples/task2136/neartop_reset_sb2500.hwpx --page 0`
  - rhwp: 2쪽
- MCP 기준 PDF 생성:
  - `pdf/task2136/neartop_reset_sb2500-2020.pdf`
  - sha256 `d3168e56fc82291c5caaa0a8748e55651d4da5756f8c0f85a8bb6ae8a873477e`
  - `pdfinfo`: 1쪽
- focused test: `issue_2136_neartop_reset_sb2500` pass
- 누적 브랜치 검증:
  - `git diff --check upstream/devel...HEAD` pass
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 합성 fixture의 HWP 2020 PDF가 테스트 기대값과 달라, 이 파일만으로 “한컴 PDF
  기준 2쪽”을 증명할 수 없다.
- 실제 근거 문서 `148753276...hwp`와 기준 PDF가 PR에 포함되어 있지 않다.

## 권고

누적 체리픽 PR에는 포함 가능하다. 단 #2136 close/후속 코멘트에서는 이 합성 fixture를
한컴 PDF 기준 증적으로 쓰지 말고, 실문서 hwpdocs 계열과 survey 결과의 후속 검증 축으로
남겨야 한다.
