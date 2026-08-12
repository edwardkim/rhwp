---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4645 review — SVG 폰트 파일 탐색 후보 경계

## 결론

**현재는 Draft 유지 및 최신 GitHub check 확인 전 수용 보류.** 최초 변경은 문서 유래 후보의 경로 성분
검사를 파일 탐색 leaf 안에 두고 private helper만 시험했다. Gestell 재검토에서 그 위치와
검증 범위가 부적절하다는 지적을 받아, 후보 계획을 SVG 렌더러의 해석 단계로 올리고 public
문서 렌더 경로의 회귀 테스트를 추가했다.

수정 뒤 독립 Gestell 재검토는 `PASS`다. `CONTRIBUTING.md`의 로컬 Rust gate도 code/test와
review 기록이 들어간 `4bb2f0a48`에서 끝까지 성공했다. 아직 완료되지 않은 최종 조건은 최신
GitHub required check와 작업지시자의 별도 상태 변경 승인뿐이다. 그 전에는 Ready 전환이나
merge를 권고하지 않는다.

## 검토 경로

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
           visual_fixture_evidence.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_external_pr.md, intake_and_review.md,
                  local_validation.md, visual_fixture_evidence.md,
                  rework_and_exceptions.md, CONTRIBUTING.md
source head before correction: 8b26078163021dcb9ecb0d93c2aa4b00fe100ab6
corrected code head: ca97299c36c9185df75e97c08b6bdfc2140cca7c
local full-gate head: 4bb2f0a48e1042dd0c031d3d33cf543b6e938de1
current upstream/devel checked: 525cf8e8ed9fa030d1db417fda5070668b2df240
merge simulation: clean (`git merge-tree --write-tree upstream/devel HEAD`)
trailing review head: 이 완료 기록을 포함하는 docs-only commit
```

`upstream/devel`은 source branch의 조상이 아니지만 merge tree가 충돌 없이 생성됐다. 이
사실은 current-base의 전체 CI를 생략하는 근거가 아니며, source branch를 임의 rebase 또는
force-push하지 않았다.

## 메타데이터

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4645](https://github.com/edwardkim/rhwp/pull/4645) |
| 관련 이슈 | 별도 공개 이슈 없음 |
| 작성자 | `humdrum00001010` |
| base / head | `devel` / `humdrum00001010:renderer/font-lookup-candidate-boundary-33` |
| source head (보정 전) | `8b26078163021dcb9ecb0d93c2aa4b00fe100ab6` |
| 보정 code head | `ca97299c36c9185df75e97c08b6bdfc2140cca7c` |
| 원격 상태 | Open Draft, `MERGEABLE` / `BLOCKED`; 완료된 check는 `cancel-stale-runs`뿐 |
| maintainer 수정 권한 | `maintainerCanModify: true` |

## 변경 범위와 소유권

문서의 `Font.name`은 parser와 style resolver를 거쳐 SVG의 `font_family`가 된다. `Full`과
`Subset` 임베딩은 그 family에서 별칭과 확장자 후보를 만들고, 설정된 font root에 결합한 뒤
바이트를 읽어 data URI에 넣는다.

따라서 다음 두 책임을 분리했다.

1. `plan_svg_font_file_lookup`은 renderer 정책이다. 문서 유래 family, 별칭, known filename,
   extension, bold 여부와 검색 root를 모아 `FontFileLookupPlan`을 만든다. 여기서만
   `FontFileName::from_document_candidate`가 하나의 `Component::Normal`인 파일명으로
   검증한다.
2. `find_font_file`은 기계적 filesystem loop다. 이미 검증된 `FontFileName`만 받아 root와
   결합해 존재하는 직접 자식 파일을 찾는다. 문서 문자열을 다시 해석하거나 후보 정책을
   선택하지 않는다.

따라서 `nested/Face`나 `../Face` 같은 문서 이름은 계획 단계에서 후보가 되지 않으며, 정상
직접 자식 filename 및 기존 alias 우선순위는 유지한다. parser, document model, 공통
`font_paths` 조달 순서와 다른 renderer backend는 변경하지 않았다.

## 회귀 테스트

새 [`tests/issue_4645_font_lookup_boundary.rs`](../../../tests/issue_4645_font_lookup_boundary.rs)는
실제 HML fixture를 face 이름만 바꿔 `DocumentCore::from_bytes`로 연 뒤 public
`render_page_svg_with_fonts(0, FontEmbedMode::Full, &[root])`를 호출한다.

- root의 `nested/<stem>.ttf` sentinel은 document face가 `nested/<stem>`일 때 SVG data URI에
  나타나지 않아야 한다.
- 같은 root의 `<stem>.ttf` direct-child sentinel은 document face가 `<stem>`일 때 SVG data URI에
  나타나야 한다.

이렇게 parser → style resolver → SVG candidate planning → filesystem read → SVG data URI를 한
테스트에서 확인한다. private helper를 직접 호출하는 단위 테스트는 mechanism 보조 검증으로만
남겼다.

## 완료한 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| public E2E | `cargo test --profile release-test --test issue_4645_font_lookup_boundary -- --nocapture`: 1 passed |
| candidate plan unit | `cargo test --profile release-test --lib renderer::svg::tests::font_file_candidates_are_single_path_components -- --nocapture`: 1 passed |
| planned lookup unit | `cargo test --profile release-test --lib renderer::svg::tests::planned_font_lookup_does_not_descend_below_search_roots -- --nocapture`: 1 passed |
| existing bold control | `cargo test --profile release-test --lib renderer::svg::tests::full_font_embed_uses_real_bold_face_when_document_uses_bold -- --nocapture`: 1 passed |
| formatting | `cargo fmt --all -- --check`: 통과 |
| whitespace | `git diff --check`: 통과 |
| 전체 Rust tests | `cargo test --profile release-test --tests`: 통과 (exit 0) |
| Clippy | `cargo clippy -- -D warnings`: 통과 (exit 0) |
| Gestell | 독립 adversarial 재검토: `PASS` |

## 렌더·시각 영향

레이아웃, geometry, paint, sample, golden은 변경하지 않는다. 보안 경계의 사용자-visible 결과는
`@font-face` data URI의 유무이므로, 위 public SVG 경로 테스트가 이 PR의 결정적 출력 증적이다.
정상 direct-child font의 data URI와 nested sentinel 부재를 함께 확인해 기존 정상 임베딩을
막지 않았음을 고정했다. 별도 한컴 PDF/overlay asset은 이 변경 주장과 직접 관련이 없어 만들지
않았다.

## 최종 조건

1. 최신 GitHub required check가 성공한다.
2. Draft 해제 및 merge는 작업지시자의 별도 승인 뒤에만 진행한다.
