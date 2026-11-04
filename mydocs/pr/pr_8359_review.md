# PR #8359 검토 — fix(layout): 기본 탭 자리를 구역 정의의 기본 탭 간격으로 정한다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8359, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `2587dd58dac9a8bf312d7cd2f59b37e803d1a387`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:00.198412+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: CONFLICT (content): Merge conflict in src/renderer/style_resolver.rs.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 다중 구역 반례와 메인터너 보정

- 위반 계약: `SectionDef.default_tab_spacing`은 현재 구역의 기본 탭 간격이며 공유 `ParaShape`의 문서 전역 속성이 아니다. HWPX `tabStop=8000`/`tabStopVal=4000`과 한컴 Mac 12.30의 16000 대조군(80pt)을 근거로 한다.
- 고정 base Native에서 두 번째 구역(8000)의 탭 간격은 53.3px였다. 원 기여의 첫 구역 전역 덮어쓰기를 통합한 진단 후보 `fd220910efe3e77aa311d8dad90af971c8b23d0f`에서는 첫 구역 16000과 두 번째 구역 8000 모두 106.7px가 되어 두 번째 구역에 회귀가 생겼다. 공개 `DocumentCore::set_section_def_native`·`get_cursor_rect_native`를 실제 호출한 결과이며 합성 한컴 Print 결과로 기록하지 않는다.
- 보정은 원본 구역값과 공유 문단 모양을 보존하고, 구역 composition → pagination 측정 → 페이지 배치에 동일한 scoped 폭을 전달한다. 단일 문단 편집·삽입도 현재 구역 범위로 재조판한다. TextStyle과 줄 채움은 같은 해소값을 소비한다.
- 구역 기본 폭이 달라지면 LayoutEngine의 셀·중첩 표 캐시를 비우고, 측정/배치의 single-line overflow 캐시도 새 폭의 namespace로 전환한다. 같은 폭의 연속 페이지에서는 캐시를 보존한다. 범위 종료·오류 반환 시 이전 폭을 복원한다. `ResolvedStyleSet` 전체를 구역마다 복제하지 않는다.
- 보정 후 반례·저장 재열기·Native/fresh WASM 검증을 완료했다.

- 1차 보정의 Native·최적화 WASM API 반례는 각각 통과했다: 구역 순서를 바꾼 cold/warm 조회, 단일 문단 편집, 다른 구역의 간격 변경, HWPX 저장 후 재열기를 독립 1구역 대조군과 비교했다. 원본 실물 1쪽은 Native/fresh WASM raw 모두 100.00%였다.
- scoped 구역 값은 공유 문단·글꼴 스타일의 변경을 요구하지 않는다. 원 PR이 setter에 추가한 전체 스타일 재해소를 제거해 기존 base의 스타일 소유·비용 계약을 유지한다. 이 마지막 보정 후 필수 gate와 실제 API 반례도 모두 통과했다.
- 전체 target Clippy가 새 context 필드가 없는 기존 테스트 생성자 6곳을 찾아냈다. 해당 테스트 파일의 10개 스타일 생성자를 기본 context로 일관되게 초기화했고 동일 all-target 검사는 통과했다. 기대값·검사 범위·source unit 수(4099)는 바꾸지 않았다.

## 구역 값의 공통 생산·소비 코드 위치

| 단계 | 실제 경로 |
| --- | --- |
| 현재 구역 원본값 → scoped 폭 | [resolved.rs](../../src/renderer/style_resolver/resolved.rs#L611); 같은 파일의 Drop에서 이전 context 복구 |
| 구역 composition 진입 | [composition.inc.rs](../../src/renderer/composer/composition.inc.rs#L31) |
| 실제 줄 채움 소비 | [line_breaking.rs](../../src/renderer/composer/line_breaking.rs#L4408) |
| 본문·도형 TextStyle 소비 | [paragraph_layout.rs](../../src/renderer/layout/paragraph_layout.rs#L5714), [shape_layout.rs](../../src/renderer/layout/shape_layout.rs#L340) |
| pagination 측정과 페이지 배치 진입 | [rendering.rs](../../src/document_core/queries/rendering.rs#L5551), [같은 파일의 페이지 트리](../../src/document_core/queries/rendering.rs#L8356) |
| 단일 문단 편집·삽입 진입 | [rendering.rs](../../src/document_core/queries/rendering.rs#L4677) |
| 폭 변경 시 셀/중첩 표 캐시 전환 | [layout.rs](../../src/renderer/layout.rs#L4155) |

최종 runtime의 실제 WASM 반례는 [final-wasm-section-tabs.json](assets/semanticist21-20261105/final-wasm-section-tabs.json)에 있다. 서로 다른 문서의 독립 1구역 대조군과 두 구역을 교대 조회하고, 편집·다른 구역 보존·저장 후 재열기를 확인했다. 문서 ID에 따른 분기나 구역별 전체 스타일 복제는 없다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·Open 통합 PR·해당 head CI·self-review·merge·후속 댓글·정리를 작업지시자가 승인했다. 실제 실행과 성공을 확인한 뒤 원격 결과를 기록한다.

## Merge 후 contributor PR comment 계획

기본 탭 16000의 저장값을 사용하고, 다른 구역의 8000 간격을 오염시키지 않는다. 실제 WASM 교대 조회·편집·저장 재열기를 확인했다.

실제 통합 PR·merge SHA·최신 head CI URL을 확정한 뒤 아래 원 PR별 증적과 함께 한국어 존댓말로 감사 댓글을 게시한다. 그림은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 아래 경로를 붙인다. merge SHA의 HTTP Content-Type과 SHA-256을 검증하고 게시 후 API 본문과 GitHub 화면 표시를 확인한다.

| 입력·쪽 | 경로 | raw 실루엣(2px) | pixel match | 내용 픽셀 보조값 | review / overlay |
| --- | --- | --- | --- | --- | --- |
| `section_default_tab_spacing` 1쪽 | native | 100.00000% | 99.82863% | 37.06755% | [review](assets/semanticist21-20261105/comment-evidence/pr8359-section_default_tab_spacing/native/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8359-section_default_tab_spacing/native/overlay/overlay_001.png) |
| `section_default_tab_spacing` 1쪽 | wasm | 100.00000% | 99.89121% | 59.70087% | [review](assets/semanticist21-20261105/comment-evidence/pr8359-section_default_tab_spacing/wasm/review/review_001.png) / [overlay](assets/semanticist21-20261105/comment-evidence/pr8359-section_default_tab_spacing/wasm/overlay/overlay_001.png) |

자동 geometry 후보 추출은 수행하지 않았으므로 후보 수를 0으로 쓰지 않는다. TSV-only 전쪽 raw 측정과 직접 판독을 구분한다. 해당 기능 검증 문서의 Native/fresh WASM 전쪽 raw 90% 미만은 0쪽이다. 내용 픽셀 보조값은 사람의 판정 정확도가 아니며 승인 gate를 대신하지 않는다. [Visual Sweep 댓글 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 [전쪽 TSV manifest](assets/semanticist21-20261105/visual-manifest.json)를 연결한다.
