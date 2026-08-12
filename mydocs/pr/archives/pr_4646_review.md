---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4646 리뷰 - 썸네일 출력 상한을 소비자 경계에 배치

## 결론

**Draft 유지, 최신 head의 전체 Contributor 게이트 대기.**
[PR #4646](https://github.com/edwardkim/rhwp/pull/4646)는 HWP/HWPX 문서 열기와
브라우저 썸네일 소비자에서 10 MiB 상한을 선택하고, 그 아래 CFB·ZIP·stream
도우미에는 호출자가 정한 상한만 전달하도록 보정했다.

상한을 넘는 `PrvImage`는 선택적 미리보기로 취급한다. 따라서 본문 파싱은 계속
성공하고, HWP 미리보기 이미지는 생략되며 HWPX contract 스트림은 기존 blank
fallback을 사용한다. 10 MiB 이하의 정상 미리보기 바이트는 종전처럼 보존한다.

renderer, layout, paint, fixture는 바꾸지 않았으므로 시각 검증 대상이 아니다.
독립 Gestell 검토는 정책 소유 경계와 정상 문서 호환성에 blocking finding 없음을
확인했다. 다만 이번 코드 head에 대한 `CONTRIBUTING.md` 전체 게이트는 아직
순차 실행 대기 상태이므로, 성공 전에는 수용·ready·merge를 권고하지 않는다.

## 검토 경로

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, rework_and_exceptions.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  rework_and_exceptions.md, local_validation.md
upstream/devel at review: 525cf8e8ed9fa030d1db417fda5070668b2df240
original remote head: 7bed2bed4f173fded30d80279935617b13c7c84e
corrected code candidate: b9cd4953f0bc7f6ccd971193b15d542c19a37754
trailing review head: this docs-only commit
```

`git merge-tree --write-tree upstream/devel b9cd4953`는 clean tree
`d1852472aa9f0730aa9218831e2cd36c921d08e0`를 만들었고,
`git diff --check upstream/devel...b9cd4953`도 통과했다. 이 기록은
collaborator source branch의 정정이며 contributor history를 재작성하지 않는다.

## 변경 판단

### 정책을 선택하는 end-to-end 소비자

- Rust HWP 문서 열기: `parse_hwp_with_cfb`가 `extract_preview`에 10 MiB를 전달한다.
- Rust HWPX 문서 열기: `parse_hwpx`가 `Preview/PrvImage.png` 보존과 HWP contract
  변환에 같은 상한을 전달한다.
- Rust 썸네일 전용 공개 API: `extract_thumbnail_only`가 HWP/HWPX 양쪽에 상한을
  전달한다.
- Chrome·Firefox·Safari: 각 `extractThumbnailFromUrl`이 10 MiB를 선택하여 CFB/ZIP
  추출기에 전달한다. Safari의 content-script 메시지는 이 공개 소비자로만 진입한다.

### 정책을 선택하지 않는 기계적 도우미

- `CfbReader::read_preview_image_limited(max_bytes)`, Rust ZIP thumbnail reader,
  HWPX contract 추출, `HwpxReader::read_file_bytes_limited` 호출은 모두 전달받은
  상한만 검사한다.
- Chrome·Firefox·Safari 공용 `readExactStreamLimited(readable, declaredSize,
  maxBytes)`는 썸네일 숫자를 보유하지 않는다. 선언 크기, 실제 stream 길이, 그리고
  호출자 상한의 일치만 검사한다.

따라서 generic HWPX 바이트 reader의 기존 BinData 기본 정책을 이 PR 범위에서
바꾸지 않으면서도, 모든 `Preview/PrvImage.png` 소비 경로는 명시적인 10 MiB
경로를 사용한다.

## 회귀 근거

| 범위 | 명령 / 확인 | 결과 |
| --- | --- | --- |
| Rust 문서 열기 | `cargo test --lib document_open_` | 4 passed — HWP/HWPX의 정상 미리보기 보존 및 초과 미리보기에서 문서 열기 지속 |
| Rust 썸네일 경로 | `cargo test --lib thumbnail_` | 7 passed — 선언 크기 초과·실제 길이 불일치 거부 포함 |
| 브라우저 공개 소비자 | `node --test rhwp-shared/sw/thumbnail-decompression.test.js` | 6 passed — Chrome `extractThumbnailFromUrl`로 정상 HWPX thumbnail과 초과 선언 크기 확인 |
| 정적/형식 검사 | `node --check` (Chrome, Firefox, Safari, shared), `cargo fmt --all -- --check`, `git diff --check` | 통과 |
| 독립 설계 검토 | terra-max Gestell adversarial review | PASS — 정책 선택은 소비자 경계, 하위 도우미는 명시 상한만 적용 |

전체 `cargo test --profile release-test --tests`와 `cargo clippy -- -D warnings`는
코드 정정 후 최신 head에서 별도로 순차 실행해야 한다. 이전 head의 성공 기록을
이 code candidate의 통과 근거로 재사용하지 않는다.

## 다음 확인

1. 최신 head에서 `CONTRIBUTING.md`의 전체 Rust gate를 순차 실행한다.
2. GitHub가 새 head의 required check를 게시한 뒤 Draft 상태와 mergeability를 다시
   확인한다.
3. 별도 승인 전에는 Draft 해제, reviewer 지정, merge, advisory 상태 변경을 하지
   않는다.
