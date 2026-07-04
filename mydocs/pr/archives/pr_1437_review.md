# PR #1437 리뷰 기록

## PR 메타

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1437 |
| 제목 | `task 1436: 크기 고정 개체 조작 차단` |
| 작성자 | `jangster77` |
| base | `edwardkim/rhwp:devel` |
| head | `edwardkim/rhwp:task_m100_1436` |
| 상태 | Open, Draft 아님 |
| mergeable | `MERGEABLE` |
| 초기 규모 | 14 files, +627 / -65 |

## 관련 이슈

- Closes https://github.com/edwardkim/rhwp/issues/1436
- 개체 속성의 `크기 고정`이 켜져도 rhwp-studio에서 리사이즈/회전/기울기 조작이 가능한 문제다.
- 한컴오피스처럼 속성 UI와 직접 조작 핸들을 모두 잠금 상태로 표시하고, 조작을 차단해야 한다.

## 변경 범위

- `CommonObjAttr.size_protect`를 Studio JSON `sizeProtect`로 노출했다.
- HWP5/HWPX `size_protect` 라운드트립 회귀 테스트를 추가했다.
- 개체 속성 대화상자의 크기/회전/대칭/기울기 관련 입력을 크기 고정 상태에서 비활성화했다.
- 키보드 리사이즈, 마우스 리사이즈, 회전 핸들, 툴바 회전/대칭 명령을 크기 고정 상태에서 차단했다.
- 선택 렌더러에서 크기 고정 개체는 한컴처럼 금지 표시 핸들로 표시한다.

## 로컬 검증

- `cargo test --test issue_1436_size_protect_properties -- --nocapture`: 통과
- `npm --prefix rhwp-studio run build`: 통과
- `cargo build --release`: 통과
- `cargo test --release --lib`: 통과
- `cargo test --profile release-test --tests`: 통과
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과

## 리스크와 판단

- `비율 유지`는 파일 포맷 속성이 아니라 사용자 설정으로 #1428 범위에 남겼다.
- 모달 외부 클릭 닫힘은 #1436의 크기 고정 조작 차단과 직접 관련된 경로가 아니므로 이번 PR에서는 추가 변경하지 않았다.
- 크기 고정 상태의 UI 비활성화와 직접 조작 차단이 모두 들어갔으므로, GitHub Actions가 통과하면 merge 가능하다.

## 최종 권고

- GitHub Actions 재실행 통과 확인 후 merge.
- merge 후 #1436 자동 close 여부를 확인한다.
