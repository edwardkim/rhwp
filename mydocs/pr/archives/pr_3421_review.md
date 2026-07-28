# PR #3421 검토 기록 — XML 1.0 비허용 문자 방어 확대

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3421](https://github.com/edwardkim/rhwp/pull/3421) — `fix(xml): XML 1.0 비허용 문자 방어를 전 방출 경로로 확대 — SVG·PDF·HWPX 저장 (#3382)` |
| 작성자·검토자 | `@chrisryugj` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `539920af94c369896b849752b05f7e26058f891e` (`chrisryugj/rhwp_fork`) |
| 원 변경 규모 | 10 files, +166 / -10; 기능 commit `23d94ac6fef1c1263f3503df17ea71aa853168b1` (source의 devel merge commit은 미적용) |
| 통합 검토 | `review/chrisryugj-20260727`, 기준 `upstream/devel` `2d7303c5bea13eaf072e782cd7f7b4a6db59b35e`; `23d94ac6…` → `e656e0381` |
| 메인터너 보정 | `ad7ce8ca6` — 공용 HWPX event writer의 text·attribute 경로까지 XML 1.0 필터 적용 |
| 관련 이슈 | [#3382](https://github.com/edwardkim/rhwp/issues/3382) |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`; CI·CodeQL·Render Diff·Native Skia·8개 default-feature shard 모두 성공 |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`visual_fixture_evidence.md`, `multi_pr_update_branch.md`.

## 원 변경과 메인터너 보정

원 PR은 XML 1.0 `Char` 집합(`#x9 | #xA | #xD | #x20..#xD7FF | #xE000..#xFFFD |
#x10000..#x10FFFF`)만 통과시키도록 DocLang, EMF/WMF·OLE/OOXML chart SVG, 수식 SVG, PDF XMP,
HWPX section 재조립과 field attribute 경로를 보완한다. 허용 범위·기존 `& < >` 이스케이프 순서는
각 방출기에서 일관되고, 탭·개행·CR·한글·non-BMP의 보존 시험도 포함한다.

정적 검토 중 HWPX의 공용 `text()` 및 `start_tag_attrs()`/`empty_tag()`가 `quick-xml`의 마크업
이스케이프만 이용하고 XML 1.0 scalar 범위를 검사하지 않는 경로를 발견했다. 이 경로는 폼 텍스트·도형
설명·hyperlink URL과 bookmark·폼 속성에서 실제 사용된다. 따라서 원 PR의 `xml_escape()`만으로는
저장한 패키지에 제어문자가 남을 수 있었다.

`ad7ce8ca6`은 공용 `filter_xml_1_0_chars()`를 추가해 event writer의 텍스트와 속성 전체에 적용하고,
수동 `field_begin_open_tag()`도 같은 helper를 재사용하게 했다. 새 회귀 시험은 `U+0003`이 text와
attribute 모두에서 제거되고 `&` 이스케이프가 유지됨을 실제 writer 출력으로 고정한다. 이 보정은
원 contributor commit의 XML 방어 의도를 확장할 뿐, 정상 XML의 출력 형식·레이아웃은 바꾸지 않는다.

## Fixture·시각 증적 판단

#3382에는 재현에 사용한 HWPX, 한글 기준 PDF, 페이지 번호 또는 첨부 파일이 남아 있지 않다. 또한 이
변경은 불법 scalar를 제거해 XML parser/브라우저의 중단을 막는 저장·방출 유효성 변경으로, 정상 문서의
기하·글리프·페이지 배치를 바꾸지 않는다. 따라서 기준 PDF 없는 임의 PNG를 시각 증적으로 만들면 이
결함의 수용 근거가 되지 않는다.

시각 sweep은 **원본 fixture 부재 및 비시각 XML 유효성 변경**으로 적용하지 않았다. 대신 위 공용 writer
출력 회귀 시험과 HWPX serializer 전체 회귀로 방출 바이트에 금지 문자가 남지 않는 경로를 검증했다.
새 HWP/HWPX fixture를 추가하지 않았으므로 IR field-sweep baseline TSV 등록 trigger도 없다.

## 검증

- #3382 XML filter focused tests와 보정의 `event_writers_drop_xml_invalid_chars_from_text_and_attributes`: 성공.
- 통합 후보 `cargo test --profile release-test --tests`: **2,962 passed / 0 failed**, IR field sweep 포함.
- Native Skia 공식 3종: **57/0**, **2/0**, **4/0**.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy -- -D warnings`,
  `cargo check --target wasm32-unknown-unknown --lib`: 성공.

## 최종 권고

**메인터너 보정 후 기술적으로 수용 가능**. 통합 PR의 최신 head CI·mergeable과 작업지시자 승인을
최종 merge 조건으로 둔다. #3382의 실제 HWPX와 한글 기준 PDF가 후속 제공되면 별도 재현·시각 대조를
추가할 수 있지만, 현재 수용을 막을 근거는 아니다.
