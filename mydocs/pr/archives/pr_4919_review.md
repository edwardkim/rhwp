# PR #4919 검토 - 문서 열기·조회의 공통 service 축

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4919](https://github.com/edwardkim/rhwp/pull/4919) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합을 위한 archive review |
| base / head | `devel` / `feat/service-layer` |
| source candidate | `0cdb09c86b8ac8fbf2a1a3ef879bfc5b25a90c14` |
| 통합 commit | `462f8daf7cf4655d45a085c8ea279bf26d38f7d6` |
| 규모 | 7 files, +1,885 / -0 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- `DocumentService`, `OpenedDocument`, 타입 있는 `ServiceError`를 신설해 파일 열기, 형식 판별,
  비밀번호 요구, 정보·검색·텍스트 추출의 공통 답을 제공한다.
- 기존 CLI, MCP, WASM 표면을 이 service로 전면 이관하지 않고 공통 축만 추가하므로, 기존 표면의 동작은
  직접 변경하지 않는다.
- 형식 재판별과 오류 문자열 분기에 흩어진 계약을 후속 표면 이관에서 안전하게 제거할 수 있는 기반으로
  적절하다.

## 검증

- source candidate의 Build & Test와 기본 feature 세 shard·slow shard는 성공했다.
- CodeQL 분석 job은 성공했고 roll-up은 `NEUTRAL`이며, Native Skia와 frontend/WASM job은 변경 영향에 따라
  skipped였다.
- #4931 누적 tree에서 `cargo fmt --check`, clippy, 전체 `cargo test --profile release-test --tests`를
  종료 코드 `0`으로 통과했다.

## 위험과 권고

service API는 새 공통 축이며 기존 public surface의 반환 형식을 즉시 통일하지 않는다. 이관 PR은 각 표면의
exit code·JSON·비밀번호 계약을 별도로 고정해야 한다. #4931을 통한 통합 merge를 권고하며, merge 뒤 원 PR은
통합 PR 링크와 함께 supersede 처리한다.
