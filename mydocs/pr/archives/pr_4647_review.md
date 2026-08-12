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
current code head: 8ea821936cababd70f92ef5e297ada97157f089f
trailing head: 이 기록과 parser architecture를 갱신하는 docs-only commit
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4647](https://github.com/edwardkim/rhwp/pull/4647) |
| 작성자 | `humdrum00001010` |
| reviewer | `edwardkim` 요청은 작성자 권한 부족으로 실패해 repository 측 지정 대기 |
| base / head | `devel` / `humdrum00001010:fix/bound-open-decompression-m34` |
| 관련 보고 | 비공개 보고 |
| 규모 | 6개 파일, +416/-65 (후속 문서 포함) |
| 상태 | Open, Draft, mergeable; required checks 대기 |

## 변경 범위

HWP5 DocInfo와 본문 섹션, HWP3 압축 본문을 문서 열기 과정에서 해제할 때 단일 스트림 상한을
적용한다. HWP5 strict·lenient 리더와 일반·비밀번호·배포용 섹션은 같은 누적 문서 예산을 소비한다.
상한 초과는 빈 레코드나 빈 섹션으로 대체하지 않고 명시적 오류로 반환한다.

HWP5 비압축 DocInfo·본문도 같은 길이 검사를 받으므로 이 변경은 압축 해제 출력뿐 아니라 핵심
스트림의 일반 크기 정책이기도 하다. 256/512 MiB는 HWP 규격의 유효성 상한이 아니라 rhwp의
결정적 자원 예산이다. `BinData`, preview와 기타 보조 스트림은 이 예산에 포함하지 않는다.

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

## 10k 문서 호환성 실험

2026-08-12 `~/hwpdocs_10k`의 HWP/HWPX 10,000건을 정확한 code head
`8ea821936cababd70f92ef5e297ada97157f089f`로 전건 열었다. 파일명과 문서 내용은 기록하지 않고
포맷·오류·크기만 집계했다.

| 항목 | 결과 |
| --- | ---: |
| 전체 입력 | 10,000 |
| 열기 성공 | 9,948 |
| decompression limit 초과 | 0 |
| 비밀번호 필요 | 5 |
| 지원하지 않는 입력 | 47 (`DRM` 8, empty 24, unknown 15) |
| HWP5 | 6,491 (6,490 성공, 비밀번호 필요 1) |
| HWP3 | 38 (38 성공) |
| HWPX | 3,424 (3,420 성공, 비밀번호 필요 4) |

HWP5 성공 문서의 decompressed/raw 핵심 스트림 분포는 다음과 같다. 파싱된 HWP5 6,490건에서
누락된 `raw_stream`은 0개였다.

| 측정 | bytes | MiB |
| --- | ---: | ---: |
| 최대 `DocInfo` | 190,940 | 0.18 |
| 최대 단일 `BodyText` 섹션 | 21,606,061 | 20.61 |
| 단일 스트림 p99 | 1,175,313 | 1.12 |
| `DocInfo` + 모든 본문 누적 최대 | 21,678,881 | 20.67 |
| 누적 p99 | 1,621,500 | 1.55 |

관측 최대는 단일 256 MiB 상한보다 12.4배, 누적 512 MiB 상한보다 24.8배 작았다. 이 결과는
현재 10k 코퍼스에서 정상 열기를 막지 않는다는 근거다. 다만 비공개 코퍼스의 경험적 결과이므로
모든 정상 문서가 상한 안이라는 규격 근거나 전역 증명으로 일반화하지 않는다.

## 검증

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target cargo test
  --profile release-test --lib open_decompression_`: 3/3 통과
- `cargo fmt --all --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target cargo clippy
  --all-targets -- -D warnings`: 통과
- 정확한 최종 code head `8ea821936cababd70f92ef5e297ada97157f089f`에서
  `cargo test --profile release-test --tests`: 통과
- 10k open/stream-size audit: 10,000건 완료, limit 초과 0, panic 0

최종 code head에서 `fmt`, 전체 `release-test`, Clippy와 diff check를 순차 실행해 통과했다. 후속 문서는
동작을 바꾸지 않는 docs-only 변경이다. 최신 PR head의 GitHub required checks 성공을 merge 조건으로
남긴다.

## 최종 권고

focused 회귀와 정적 검사가 통과했고 포맷별 파서 경계 밖 변경이 없어 merge 후보로 권고한다. repository
권한 보유자의 reviewer 지정, 최신 PR head의 required checks 성공, 작업지시자 승인을 merge 전 조건으로
남긴다.
