# PR #1958 리뷰

## 메타

| 항목 | 내용 |
| --- | --- |
| PR | #1958 |
| 작성자 | planet6897 |
| 제목 | Issue #1946: 암호화 HWPX 감지 및 ENCRYPTED_SKIP 분류 |
| base | devel |
| 문서 작성 시점 참고값 | mergeable: MERGEABLE, mergeStateStatus: CLEAN |
| 변경 규모 | 3 files, +101/-2 |

## 변경 범위

- HWPX `META-INF/manifest.xml`의 ODF `encryption-data`를 감지해 암호화 문서를 조기 분류한다.
- 암호문을 UTF-8 손상 XML로 오진하지 않고 `HwpxError::Encrypted`로 명확히 반환한다.
- `hwpx-roundtrip` 배치 진단에서 암호화 문서를 `ENCRYPTED_SKIP`으로 분류한다.

## visual sweep 판정

- 파서/진단 분류 변경이며 렌더 출력 변경이 아니다.
- 목적은 암호화 문서의 명확한 진단과 배치 게이트 분류이므로 parser/diagnostics 테스트와 CI로 판단한다.

## 로컬 검증

누적 검토 브랜치 `review/planet-1940-1960`에서 오래된 순서로 cherry-pick했다.

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib`: 통과, 2123 passed / 0 failed / 6 ignored
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## 검토 메모

- #1942에서 재분류된 암호화 HWPX 축을 별도 PR로 명확히 처리한다.
- 복호화 기능은 범위 밖이고, 본 PR은 감지/분류만 담당한다.

## 결론

merge 후보로 판단한다. #1957 이후 순서로 merge한다.
