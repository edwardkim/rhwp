---
kind: report
status: active
last_verified: 2026-08-12
---

# PR #4647 검토 — 문서 열기 압축 해제 상한

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md
current head: 23505a2fbba27a4e01b5b7af289c667805dd5d6a (접수 시점 참고)
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4647](https://github.com/edwardkim/rhwp/pull/4647) |
| 작성자 | `humdrum00001010` |
| reviewer | `edwardkim` 요청은 작성자 권한 부족으로 실패해 repository 측 지정 대기 |
| base / head | `devel` / `humdrum00001010:fix/bound-open-decompression-m34` |
| 관련 보고 | 비공개 보고 |
| 규모 | 4개 파일, +272/-66, 1 commit |
| 상태 | Open, non-draft, `MERGEABLE/BLOCKED` |

## 변경 범위

HWP5 DocInfo와 본문 섹션, HWP3 압축 본문을 문서 열기 과정에서 해제할 때 단일 스트림 상한을
적용한다. HWP5 strict·lenient 리더와 일반·비밀번호·배포용 섹션은 같은 누적 문서 예산을 소비한다.
상한 초과는 빈 레코드나 빈 섹션으로 대체하지 않고 명시적 오류로 반환한다.

변경은 `src/parser/`의 포맷 소유 경계에 한정한다. renderer, layout, WASM API와 rhwp-studio 출력은
바꾸지 않으므로 시각·fixture 증적 경로는 적용하지 않는다.

## Review finding

blocking code finding은 없다.

- 단일 스트림은 HWPX XML 엔트리와 HWP3 레코드의 기존 계약에 맞춘 256 MiB로 제한한다.
- HWP5 문서 전체의 DocInfo·본문 결과는 512 MiB 누적 예산을 사용해 섹션 반복으로 상한을 우회하지
  못하게 한다.
- 기존 `decompress_stream_limited`를 HWP5 strict·lenient 리더가 함께 사용하고, HWP3 전용 raw-deflate
  해석은 HWP3 파서 안에 유지한다.
- 회귀 테스트는 큰 fixture를 저장하지 않고 작은 결정적 입력과 축소한 테스트 상한으로 초과·경계 성공을
  함께 확인한다.

## 검증

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target cargo test
  --profile release-test --lib open_decompression_`: 3/3 통과
- `cargo fmt --all --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target cargo clippy
  --all-targets -- -D warnings`: 통과

공유 검증 host의 저장 공간 제약 때문에 local `release-test` 전체 회귀는 실행하지 않았다. 최신 PR head의
GitHub required checks 성공을 merge 조건으로 남긴다.

## 최종 권고

focused 회귀와 정적 검사가 통과했고 포맷별 파서 경계 밖 변경이 없어 merge 후보로 권고한다. repository
권한 보유자의 reviewer 지정, 최신 PR head의 required checks 성공, 작업지시자 승인을 merge 전 조건으로
남긴다.
