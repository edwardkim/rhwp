# PR #2521 검토 - kevin9327 G6-G12 기여 PR 누적 통합

| 항목 | 내용 |
|---|---|
| PR | [#2521](https://github.com/edwardkim/rhwp/pull/2521) |
| 작성자 / base | jangster77 / `devel` |
| 대상 | kevin9327의 열린 원 PR 39건 (`#2464`~`#2511`, 결번 제외) |
| 검토자 | @jangster77 |
| 판단 | ClickHere 회귀 보정 후 최신 head CI 성공을 조건으로 수용 |

## 통합 범위

- 원 PR별 검토 기록은 `mydocs/pr/archives/pr_{번호}_review.md` 39개에 보관한다. 각 문서는 PR 본문,
  검토 시점의 코멘트, 직접 적용 또는 상위 통합 흡수 관계, 검증, 렌더 영향과 권고를 구분한다.
- HWP/HWPX/HWP5 속성 보존, HML 표 read-back, undo field range, Safari·Chrome·VS Code·npm·CI 메타데이터를
  하나의 최신 `upstream/devel` 위에서 검증했다.
- 중복된 Actions, Dependabot, npm metadata 제안은 더 넓은 상위 변경만 유지했다. 동일 범위를 두 번 적용해
  정책이나 파일 목록이 되돌아가는 것을 방지한다.

## Collaborator 보정

- Chrome DevTools consumer가 producer와 다른 DOM attribute를 읽던 문제를 기존 contract인
  `data-hwp-extension-version`으로 맞추고 source 회귀를 추가했다.
- Safari의 HML 원격 로드는 bounded UTF-8/UTF-16 HWPML root+Version signature gate로 제한했고, build script가
  실패 상태를 호출자에게 전파하도록 보정했다.
- README 계열의 Rust 테스트 수는 실제 `--list` 결과를 기준으로 `3,400+`로 보수적으로 정정했다.
- E2E runner는 `runTest()` Promise가 종료 전에 유실되지 않도록 저장 포맷 시나리오와 text-flow 시나리오를
  top-level `await`로 완료 대기하게 했다.
- Chrome/Firefox/Safari content script가 동적으로 추가된 `<a>` 노드 자신도 처리하도록 보정했다. 이전에는
  하위 링크만 순회해 동적 HWP/HWPX/HML 링크가 누락될 수 있었다.

## 검증

- 최초 CI의 `issue_258_clickhere_form_mode`는 ClickHere 복사/붙여넣기 3건을 실패했다. `Field`를 일반 이동형 문자 컨트롤로 분류하면서 `split_at()`의 보이는 문자 offset이 한 칸씩 어긋난 것이 원인이다.
- 보정 뒤 `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_258_clickhere_form_mode` 13건과 [#2480](https://github.com/edwardkim/rhwp/pull/2480) 병합 undo·visible-offset 분할 회귀를 통과했다.
- 전체 release-test의 최종 근거는 이 보정을 포함한 최신 PR head CI로 갱신한다. 최초 CI 이전의 전체 회귀 성공 기록만으로는 merge 근거로 사용하지 않는다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg` 통과.
- npm editor `npm test` 18/18, `npm pack --dry-run`, Chrome/Firefox production build, unsigned Safari Xcode build,
  shared HML signature 회귀를 통과했다.
- headless browser E2E에서 HWP→HWPX 및 HWPX→HWP 저장 UI, MIME/파일 매직, 재열기를 확인했고, HML 열기·저장·재열기,
  text-flow의 2쪽 생성과 Backspace 문단 병합도 통과했다.
- 실제 Chrome unpacked extension E2E에서 정적·동적 `.hml` 링크가 badge를 받고, badge click이
  `samples/hml/formatting_table.hml`을 extension viewer의 1쪽 canvas로 렌더링하는 것을 확인했다. shared 및
  Chrome/Firefox 확장 Node 회귀 57개도 통과했다.

## 렌더 영향 판정

- 변경의 중심은 serializer/parser 계약, 편집 상태, 확장 식별과 패키지·CI 메타데이터다. 새 renderer 또는
  layout fidelity 주장을 포함하지 않는다.
- 저장 구조 보존의 browser smoke는 E2E로 확인하되, 각 속성의 정확한 oracle은 원 PR별 focused Rust 회귀로 유지한다.
- 따라서 이 누적 PR에는 포괄 visual sweep을 merge 조건으로 추가하지 않는다. renderer·layout을 직접 바꾸는
  후속 변경은 별도 시각 검증 정책을 적용한다.

## Merge 전 조건과 후속

- [#2521](https://github.com/edwardkim/rhwp/pull/2521) 최신 head의 필수 GitHub Actions가 모두 성공해야 한다. 특히 `issue_258_clickhere_form_mode`를 포함하는 default-feature test shard가 성공해야 한다.
- 작업지시자의 merge 승인을 받은 뒤에만 merge한다.
- merge 뒤 `upstream/devel`을 동기화하고, 각 원 PR에 통합 PR 링크와 해당 개별 검토 문서의 결론을 남긴 뒤
  직접 적용 또는 상위 통합 흡수 여부에 따라 close한다. merge SHA, close 결과, 코멘트 URL은 GitHub 원천 기록으로
  확인하며 이 문서에 미리 단정하지 않는다.
