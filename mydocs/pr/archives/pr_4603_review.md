---
kind: pr-review
status: pending-review-only-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4603 검토 - 차트 숫자 데이터 편집과 CSV 왕복

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, review_only_fast_pass.md
```

| 항목 | 기록 |
| --- | --- |
| PR | [#4603](https://github.com/edwardkim/rhwp/pull/4603) |
| 관련 이슈 | [#4100](https://github.com/edwardkim/rhwp/issues/4100) |
| 작성자 / source | @johndoekim / `task4100` |
| 원 보정 전 source | `1cc5bdfea20e2dbd51f5c712ca6dc6297f57e97b` |
| 가시성 검토 브랜치 | `review/johndoekim-4603-20260812` |
| 최신 `devel` 병합 | `f464fd7dfe613d9b9beb07088e71ae25d93c46b7` (`upstream/devel@525cf8e8e`) |
| 최종 code candidate | `60f3989ae3dacb5876a36d96ab75070974545c20` |
| source 수정 권한 | `maintainerCanModify=true` |
| code candidate CI | Full CI·CodeQL·Render Diff 성공 |
| mergeable | 문서 작성 시점 `MERGEABLE` / `CLEAN`; docs-only head에서 재확인 필요 |

원 PR은 기존 HWP/HWPX 차트의 숫자 값을 최소 바이트 패치로 바꾸고, HWPX의 zip 차트 파트와
중첩 CFB 사본을 함께 갱신하는 B1 엔진·CLI 구현이다. 점·계열 신설이나 삭제, WASM/UI 표면은
범위 밖으로 유지한다.

## contributor 보정 확인

메인터너의 공식 요청 세 건은 현재 source에 모두 반영됐다.

1. `276649891`은 실제 차트 편집 뒤 `invalidate_page_tree_cache()`를 호출한다. `bin_data_epoch`만
   증가하면 RawSvg 차트가 이미 만들어진 page/layer cache에 남아 재렌더가 옛 SVG를 반환할 수 있었다.
   T8은 HWPX·HWP5에서 선행 렌더, `91.7` 편집, 재렌더 순서를 실행하고 새 layer SVG와 냉간 렌더의
   일치를 고정한다.
2. `943253e60`은 희소·역순·중복 `c:pt idx`를 `nonSequentialPointIndex`로 거부한다. 기존 CSV와
   편집 주소는 벡터 출현 순서를 썼으므로, 희소 `idx`를 허용하면 한 행의 수정이 다른 XML 점을
   조용히 바꿀 수 있었다. 읽기·쓰기·CSV가 같은 스캐너를 지나며, 거부 시 두 슬롯은 바뀌지 않는다.
3. `6948e8723`은 Windows CRLF checkout에서 누락됐던 rustfmt 검사를 LF 기준으로 다시 적용했다.
   이후 `cargo fmt --all -- --check`와 CI format check가 통과했다.

## 메인터너 안전 보정

최신 source를 로컬 검토하면서, 원 요청만으로는 남는 CSV 편집 경계를 확인해
`60f3989ae`에서 보정했다. contributor 원 변경과 이 보정은 별도 commit으로 유지했다.

- **두 OOXML 표현을 독립 패치한다.** HWPX에는 `Chart/chartN.xml`(①)과 중첩 CFB
  `OOXMLChartContents`(②)가 있다. 기존 경로는 ①에서 만든 전체 patched XML을 ②에도 복사했다.
  확장 속성·미래 요소처럼 논리 차트 데이터 밖의 바이트가 서로 다르면 ②의 정보를 ①로 덮어써
  잃게 된다. 이제 각 원본 XML에서 각자 scan한 byte span만 패치한다.
- **두 표현의 논리 데이터가 다르면 정본을 추정하지 않는다.** 계열명·축·라벨·점 `idx`·값이
  서로 다른 ①/②는 `representationMismatch`로 읽기와 쓰기 모두 거부한다. 한쪽을 기준으로 다른
  사본을 고치는 것은 데이터 손실이므로, 거부 경로에서 어느 slot도 기록하지 않는다.
- **CSV 한 열이 표현할 수 없는 라벨은 차단한다.** 계열별 카테고리 라벨, 또는 분산형의 X 값이
  다르면 첫 CSV 열 하나가 서로 다른 점 주소를 뜻한다. `chart-to-csv`는 파일을 만들지 않고,
  `csv-to-chart`와 native labels 입력은 각각 `sharedCategoryRequired` 또는 `sharedXRequired`로
  무기록 거부한다.
- **빈 값과 무제목 계열의 무편집 왕복을 보존한다.** `<c:v/>`는 구조를 새로 만들지 않는 한 직접
  편집할 수 없어 `valueNotPatchable`로 거부한다. 다만 같은 빈 칸을 그대로 둔 채 다른 숫자를
  고칠 때 전체 CSV를 `notANumber`로 거부하지 않는다. `<c:tx>` 부재와 CSV의 빈 머리 칸도
  계열명을 편집하지 않는 B1 범위에서는 같은 무편집 상태로 비교한다.

CLI 계약 문서도 `representationMismatch`, `sharedCategoryRequired`, 각 표현의 독립 패치와
fail-closed 조건을 명시하도록 갱신했다.

## 완료한 검증

로컬 검증은 고정 `target/pr-review`를 재사용했고 `CARGO_INCREMENTAL=0`은 지정하지 않았다.

- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test issue_4100_chart_data_edit --test chart_csv_contract --test issue_2724_passthrough_invalidation_guard --test provenance_contract --test-threads 12 --no-fail-fast`
  : 67 passed, 1 skipped.
- 위 실행에는 ①/② 독립 패치, 논리 불일치 무기록 거부, 빈 값의 이웃 값 편집, 비공유 카테고리
  거부, 무제목 계열 CSV 왕복과 기존 T8·비순차 `idx` 회귀가 포함된다.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings`,
  `git diff --check`: 모두 통과.
- code candidate의 [CI](https://github.com/edwardkim/rhwp/actions/runs/31583743027)는 Lint,
  Frontend package gates, Native Skia, archive build 3개, default-feature shard 4개와 최종
  Build & Test까지 성공했다. [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31583742677)의
  JavaScript/TypeScript·Python·Rust 분석과 [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31583742647)도 성공했다.

## 범위와 최종 판단

이번 보정은 안전하지 않은 CSV 행렬을 거부하고 두 기존 OOXML 사본의 보존을 강화한다. 차트의
희소 `idx`/`ptCount` 완전 지원, 점·계열·라벨 구조 편집, `c:f` 동기화, WASM과 Studio UI는 B1의
후속 범위로 남긴다.

**수용 권고.** 현재 code candidate는 로컬 집중 회귀와 Full CI를 모두 통과했다. 이 review와
오늘할일만 포함하는 trailing commit의 review-only fast-pass, 최신 `MERGEABLE` 상태를 확인한 뒤
merge한다. PR 본문의 closing keyword에 따라 merge 뒤 [#4100](https://github.com/edwardkim/rhwp/issues/4100)의
자동 close 여부를 확인하고, 실제 보정 이유와 검증 결과를 포함한 maintainer 후속 코멘트를 남긴다.
