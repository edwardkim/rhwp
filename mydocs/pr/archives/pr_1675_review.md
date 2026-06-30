# PR #1675 검토 - HWPX package graph 직렬화 복원

- 작성일: 2026-06-30
- PR: https://github.com/edwardkim/rhwp/pull/1675
- 제목: `[Ghidra/Frida 근거] HWPX 패키지 그래프 직렬화 복원`
- 컨트리뷰터: [@humdrum00001010](https://github.com/humdrum00001010)
- base/head: `devel` <- `humdrum00001010:codex/pr1573-package-graph`
- 문서 작성 시점 head: `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae`
- 문서 작성 시점 상태: draft 아님, `MERGEABLE` / `CLEAN`, `maintainerCanModify=true`
- 연결 이슈: 없음
- 라벨: `hwpx`, `serialization`, `roundtrip`
- 규모: +429 / -6, 3 files
- 처리 경로: Route A - collaborator-mediated 외부 PR head 문서 포함

## 1. PR 정보

변경 파일:

| 파일 | 내용 |
|---|---|
| `src/serializer/hwpx/mod.rs` | `META-INF/container.rdf`를 섹션 수에 맞게 동적 생성, master page ZIP entry 생성, 신규 회귀 테스트 추가 |
| `src/serializer/hwpx/package_check.rs` | package checker에 `container.rdf` 섹션 커버리지와 master page manifest/idRef 검증 추가 |
| `src/serializer/hwpx/section.rs` | header/footer `id`를 `raw_ctrl_extra`에서 복원해 HWPX XML에 보존 |

커밋:

| SHA | 내용 | 작성자 |
|---|---|---|
| `264e2abdbecab1efde3442024ddc4f571c015e7b` | `fix: restore HWPX package graph for Hancom` | @humdrum00001010 / phi hu |
| `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae` | `Merge branch 'devel' into codex/pr1573-package-graph` | @humdrum00001010 |

## 2. 변경 내용 분석

이번 PR은 HWPX 시각 렌더링 조정이 아니라 serialize 산출물의 package graph 정합성 복원이다.
핵심 축은 세 가지다.

1. `container.rdf`를 정적 `section0.xml` 기준에서 벗어나 실제 `Document.sections` 수에 맞춰
   `Contents/header.xml`과 모든 `Contents/section{N}.xml`을 나열한다.
2. IR의 `section_def.master_pages`를 `Contents/masterpage{N}.xml`로 ZIP에 쓰고, `content.hpf`
   manifest 및 각 section XML의 `hp:masterPage idRef`와 전역 인덱스 규칙을 맞춘다.
3. HWPX header/footer control의 `id` 값을 항상 `0`으로 재작성하지 않고, 파서가 보존한
   `raw_ctrl_extra`의 앞 4바이트에서 복원한다.

`package_check`는 위 변경이 다시 깨지지 않도록 다음을 검사한다.

- `content.hpf` manifest href가 ZIP entry에 존재하는지
- `container.rdf`가 header와 모든 section part를 참조하는지
- master page ZIP entry 수, manifest 등록, section XML의 `masterPageCnt`와 `idRef`가 IR과 일치하는지
- 기존 BinData 수/확장자 검증이 계속 유지되는지

## 3. 검토 의견

### 수용 근거

- 변경 범위가 HWPX serializer/package checker에 한정되어 있고 renderer, layout, golden, sample 변경이 없다.
- `container.rdf` 정적 상수 사용으로 multi-section HWPX에서 package graph가 stale해지는 문제를 직접 제거한다.
- master page는 이미 `src/serializer/hwpx/master_page.rs`와 `content.hpf` writer의 manifest 축이 존재하므로,
  이번 PR의 ZIP entry 생성과 section `idRef` 보강이 기존 설계와 맞는다.
- header/footer id 보존은 파서가 이미 보존한 raw control payload를 쓰는 좁은 변경이다.
- 신규 테스트가 문제 축을 직접 고정한다.
  - `container_rdf_lists_every_section`
  - `master_pages_are_serialized_as_package_parts`
  - `header_footer_ids_are_preserved`
  - package checker의 master page/container.rdf mismatch 검출 테스트

### 리스크 및 확인 사항

- `container.rdf`와 package checker의 XML 검증은 구조화 XML parser가 아니라 자체 writer 산출물에 대한 문자열 검증이다.
  현재 serializer 회귀 가드로는 충분하지만, 외부 임의 XML validator 성격으로 확장할 때는 parser 기반 검증이 필요할 수 있다.
- header/footer id가 `raw_ctrl_extra`에 없으면 기존 동작처럼 `0`으로 fallback한다. 비파싱 IR에서 id를 별도 모델 필드로
  구성하지 않는 현 구조에서는 합리적인 fallback이다.
- 이 PR은 package graph/roundtrip 정합성 PR이며 시각 정합성 자체를 보장하지 않는다.

blocking finding은 없다.

## 4. 로컬 검증

검증 기준 브랜치: `local/pr1675`

| 항목 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo test --release --lib serializer::hwpx::` | 통과, 270 passed / 1 ignored |
| `cargo test --release --test hwpx_roundtrip_baseline` | 통과, 4 passed |

비고: `cargo test --release serializer::hwpx::`는 integration test 바이너리 전체 컴파일까지 확장되어
검증 범위가 과도하므로 중단했고, 최종 검증은 `--lib`로 serializer 단위 테스트 범위를 명확히 제한했다.

## 5. 시각 검증

별도 시각 검증 산출물은 만들지 않았다. 이번 변경은 HWPX ZIP package graph, manifest/idRef, header/footer id
보존에 관한 직렬화 변경이며 renderer 출력이나 golden SVG/PDF를 변경하지 않는다. 따라서 로컬 검증은 HWPX
parse -> serialize -> reparse, package check, 2-round 안정성 게이트로 수행했다.

## 6. PR head 문서 push 계획

작업지시자 승인 후 이 문서와 `mydocs/pr/archives/pr_1675_report.md`만 별도 docs commit으로 묶어
원 contributor PR head에 push한다.

예정 커밋:

```text
docs: PR #1675 검토 기록
```

예정 push 대상:

```text
https://github.com/humdrum00001010/rhwp.git HEAD:codex/pr1573-package-graph
```

push 전 확인:

- 로컬 branch가 최신 PR head `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae` 기반인지 재확인
- staged file이 `mydocs/pr/archives/pr_1675_review.md`와 `mydocs/pr/archives/pr_1675_report.md`만인지 확인
- contributor 원 코드 commit rewrite 없음

push 후 확인:

- PR head SHA가 docs commit으로만 변경됐는지 확인
- PR diff에 두 archive 문서가 포함됐는지 확인
- 다른 로컬/원격 PR branch에 문서 commit이 들어가지 않았는지 확인

## 7. 권고

**수용 / merge 준비 권고.**

merge 전 최종 조건:

- 문서 push 후 최신 PR head 기준 GitHub Actions 재실행 결과 확인
- 최신 `mergeable` / `mergeStateStatus` 재확인
- review 문서가 PR diff에 포함됐는지 확인
- GitHub review approval 또는 동등한 검토 의견 등록
- 작업지시자 merge 승인
