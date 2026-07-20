# PR #2511 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2511](https://github.com/edwardkim/rhwp/pull/2511) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +8/-8, 4 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Chrome/Firefox/Safari 확장의 `.hml` 파일 로드 지원 |
| 판단 | collaborator 보정 포함 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 브라우저 확장 파일 선택/URL 로드 경로가 `.hml`을 문서 형식으로 인식하도록 등록한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 변경 `76aa00d5f`, 회귀 `301bb07a0`을 적용했다. Safari만 binary signature gate 때문에 HML XML을 거부하던 별도 결함은 collaborator 보정 `7304b385a`으로 bounded UTF-8/UTF-16 HWPML root 검증을 추가해 해결했다.

## 렌더 영향 판정
- 파일 형식 식별과 보안 gate 변경이며 renderer·layout 변경이 아니다. HML 문서 렌더 fidelity 주장이 없으므로 visual sweep을 merge 조건으로 요구하지 않는다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- shared `file-signature` 회귀 6개, Chrome/Firefox production build, unsigned Safari Xcode build를 통과했다. Safari `build.sh`의 서명 실패는 로컬 인증서·CoreSimulator 환경 제약이며 source build 실패가 아니다.

- headless hml-open.check E2E에서 HML 문서 열기·저장·재열기를 확인했다. 이는 Studio HML 사용자 흐름 근거다.
- 실제 Chrome unpacked extension E2E에서 정적 및 동적 `.hml` 링크에 확장 표식·badge가 붙고, trusted badge click이 viewer에 원본 URL을 전달해 `samples/hml/formatting_table.hml`을 1쪽 canvas로 렌더링함을 확인했다. 상태 바는 `formatting_table.hml — 1페이지`를 표시했다. 이 과정에서 동적으로 추가된 `<a>` 자체를 누락하던 공통 observer 경로는 Chrome/Firefox/Safari content script 보정으로 함께 해결했다.

## 리스크와 권고
- HML 허용은 비어 있지 않은 `HWPML` root와 Version을 확인하는 bounded parser로 제한해 HTML/JSON/손상 입력을 계속 거부한다.
- **권고**: collaborator 보정 포함 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
