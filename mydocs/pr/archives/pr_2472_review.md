# PR #2472 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2472](https://github.com/edwardkim/rhwp/pull/2472) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +43/-2, 2 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | HML 표의 `TextWrap`이 HML 되읽기에서 유실되는 문제 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 표 `TextWrap` 속성이 HML parser를 거친 뒤 사라지는 직렬화/역직렬화 결함을 다룬다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `ec226dad8`을 적용했고, 충돌 해소 뒤 형식만 별도 정리했다.

## 렌더 영향 판정
- HML 구조 보존과 round-trip 회귀가 범위다. renderer·layout 출력 경로 변경이 아니므로 visual sweep을 merge 조건으로 요구하지 않는다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- `table_text_wrap_is_read_back_from_hml` 회귀를 포함한 전체 release-test가 통과했다.

- headless hml-open.check E2E에서 HML 열기, HML 의미 보존 저장, 재열기와 기존 HWP 재열기를 모두 확인했다. 이는 HML parser/저장 UI 흐름의 browser 근거이며 표 TextWrap 보존은 focused Rust 회귀가 직접 보장한다.

## 리스크와 권고
- HML 표 속성의 read-back 계약이 바뀌는 범위이므로 추가 serializer 변경과 분리해 유지한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
