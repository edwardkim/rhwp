---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4607 검토 - 인라인 개체 흐름 높이 단일화

## 판정

로컬 수용. `ShapeObject::flow_height_hu()`를 조판, 줄바꿈, 배치가 공통으로 사용하도록 하여
`common.height`와 `shape_attr.current_height`가 다른 글자처럼 취급 도형의 예약 높이 불일치를
제거한다. 원격 PR의 최종 CI와 작업지시자 승인 전에는 병합하지 않는다.

## 검토 기준

- 원격 head: `e9f2dad5fb593b4567d19e624b2900800358c113`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 기준 base: `upstream/devel@9b9cbf3c80b62f504f347d0713d302e50e1d9243`
- 적용 순서: #4607의 8개 commit을 첫 번째로 적용했다.

## 확인

- `cargo test --profile release-test --test issue_1116`: 13 passed.
- 통합 전체 Rust: `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: 5,906 passed, 37 skipped.
- Native Skia와 direct-PDF fixture는 각각 58/58, 2/2, 4/4 통과했다.
- 한컴 2022 기준 PDF `pdf/hwp3-sample16-hwp5-2022.pdf`와 `samples/hwp3-sample16-hwp5-2022.hwp`의 p2/p3를 대조했다. render tree와 기준 PDF는 모두 64쪽이고, owner 이동·본문/각주·표 구조 후보는 없었다.

## 시각 검토

픽셀 diff는 p2 12.77%, p3 22.34%다. p3의 PDF 텍스트층 U+F000 6개와 SVG의 U+25A1 6개 차이 및 기존 글꼴·조판 차이가 남아 있어 절대 일치 판정으로 사용하지 않았다. 인라인 도형 높이 변경이 새 페이지 이동이나 구조 누락을 만들었다는 근거는 없으며, 잔여 fidelity는 [#3820](https://github.com/edwardkim/rhwp/issues/3820)에서 계속 추적한다. 산출물은 `output/pr-review/humdrum-4607-sample16-2022-p2/`, `output/pr-review/humdrum-4607-sample16-2022-p3/`에 있다.
