# PR #4775 검토 - HWP3 HWPX 내보내기 IR 보존

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4775](https://github.com/edwardkim/rhwp/pull/4775) |
| 관련 이슈 | `Closes #3739` |
| 작성자·검토 방식 | `jangster77` · 작성자 self-review (외부 reviewer 미지정) |
| base / head | `devel` / `task_m100_3739` |
| code candidate | `d7585993dd1b67e5bfc2e7db9165dae2a2cd79b6` (CI lint 보정 포함) |
| 규모 | 코드 5 commits + 이 review·오늘할일 trailing commit |
| 작성 시점 상태 | code head Full CI 성공, `MERGEABLE/CLEAN`; 이 trailing 문서 head CI 대기 |
| 라우팅 | collaborator self-merge; intake·local validation·visual fixture evidence |

`mergeable`, `mergeStateStatus`, head SHA 및 CI 결과는 작성 시점의 참고값이다. 이 review·오늘할일
trailing commit을 push한 뒤에는 최신 head의 GitHub Actions, `MERGEABLE`, `CLEAN`을 merge 직전에
다시 확인한다.

## 변경 범위와 판단

- 동일 `char_shape_id`라도 start position이 다른 HWPX run 경계를 serializer·parser에서 보존한다.
- Windows PowerShell/.NET pipe의 선두 UTF-8 BOM을 암호 본문에서 제거해 암호 HWP3·HWP5·HWPX의
  `--password-stdin` 내보내기를 보정한다.
- 암호 HWP3가 개체 위치를 `U+FFFC` 한 글자로 보이되 실제 offset에서는 8 UTF-16 단위 슬롯으로
  세는 계약을 HWPX control 슬롯으로 치환해, 이후 char shape가 밀리지 않게 한다.
- HWP3 하이퍼텍스트는 HWPX `HYPERLINK` field와 Command parameter로 보존한다. URL이 비어 있으면
  표시 문자열을 fallback으로 유지한다.
- HWP3의 빈 그림 `imgRect`와 HWPX의 canonical 실측 사각형, HWP3 Hyperlink와 재파싱 HWPX Field의
  차이는 원본·재파싱 control을 확인한 정확한 경우에만 HWP3→HWPX `--verify`에서 정규화한다.

renderer, layout, WASM, Studio, 신규 sample·golden·기준 PDF는 변경하지 않았다. Windows PowerShell의
한글 PR 본문 전송 절차와 review·오늘할일도 함께 정리했다. 기존 HWP/HWPX fixture를 쓰는
parser·serializer 구조 보존 PR이므로 별도 visual sweep을 merge 근거로 사용하지 않았다. `--verify-pages`의
24쪽 결과는 페이지 수 구조 검증이지 한컴 PDF와의 시각 일치 주장도 아니다.

## 완료된 검증

- 최신 `upstream/devel@86b966ac5` 위로 rebase한 뒤
  `cargo build --profile release-test --target-dir target\pr-review --bin rhwp`를 통과했다.
- 실제 암호 HWP3 표본을 `--password 123456 export-hwpx --verify --verify-pages`로 변환해
  IR 무차이와 24쪽 재열기, exit 0을 확인했다.
- `cargo test --profile release-test --target-dir target\pr-review --test issue_3739_hwpx_same_char_shape_boundary -- --nocapture`를
  실행해 4 passed를 확인했다. BOM 포함 stdin의 암호 HWP3 IR·페이지 검증도 이 통합 테스트에 포함된다.
- #3739 serializer/parser/정규화 focused 단위 테스트 4건과 HWP3 URL 누락 fallback field 단위 테스트를
  code candidate에서 통과했다.
- CI에서 실패한 `password_stdin_ignores_only_a_leading_utf8_bom` 바이너리 단위 테스트를 보정 뒤 다시
  실행해 1 passed를 확인했고, `cargo build --workspace --target-dir target\pr-review`도 통과했다.
- 변경한 `src/main.rs`의 `rustfmt --check`와 `git diff --check`를 통과했다. 전체 `cargo fmt --check`는
  이 Windows 호스트에서 긴 파일 경로(`os error 206`)로 시작 단계에 실패했다.

전체 `cargo nextest run --tests`, WASM은 로컬에서 실행하지 않았다. `cargo clippy --workspace --all-targets
-- -D warnings`는 이 호스트에서 Cargo 내부 대기 상태로 완료 결과를 내지 못해 중단했다. 그러나 code head
`d7585993`의 GitHub Actions는 Lint, Build & Test, Native Skia, Canvas visual diff, Rust CodeQL을 모두
성공으로 판정했다. 이 trailing 문서 head는 별도로 CI 성공을 확인한다.

## 위험과 후속 범위

- 중첩 문단의 HWP3 Hyperlink는 원본 control 경로를 확인하는 정규화가 아직 없으므로 보수적으로 diff를
  남긴다. 이번 표본의 본문 hyperlink만 추정 없이 정규화했다.
- HWP3 추가정보가 없는 hyperlink의 표시 문자열은 실제 URL이 아닐 수 있다. 빈 값으로 버리지 않고
  Command에 보존하지만, URL 추출 정밀화는 별도 범위다.
- 다른 HWP5·HWPX field나 실제 picture geometry 차이는 정규화하지 않는다.

## 최종 권고

조건부 merge를 권고한다. 이 review·오늘할일 trailing head의 CI가 성공하고, merge 직전에 최신 head SHA,
`MERGEABLE`, `CLEAN`을 다시 확인한 뒤 작업지시자 승인으로 self-merge한다.
