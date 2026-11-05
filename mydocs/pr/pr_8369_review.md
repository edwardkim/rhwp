# PR #8369 검토 — fix(renderer): 글자 외곽선 종류를 한컴 인쇄 pen과 반복 구간으로 긋는다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 보정·필수 검증 완료.** [통합 검증](semanticist21_20261105_next10_review.md)을 완료했다. 실제 remote push·Open PR·최신 head CI·self-review·merge·후속 처리는 별도 승인 뒤 수행한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#8369](https://github.com/edwardkim/rhwp/pull/8369) / semanticist21 / devel |
| 원 repository·branch | `semanticist21/rhwp` / `fix/char-outline-line-types` |
| 원 head | `3c5accd7b604594585ebc1b1fe96ea155069821e` |
| 접수 시 devel | `ed057276c1e0e07e743218d95641e015b46b6715` |
| 최종 검증 base | `6e1de406209ba9a58032f6ca6de45b18aa687d1c` |
| reviewer·API 확인 | `edwardkim` / `2026-11-05T01:41:03.191392+00:00` UTC / `requested_reviewers.users` 확인·지정 전후 head 동일 |
| 규모·당시 mergeability | +193 / -19 / 9 files / `CONFLICTING`·`DIRTY` (조회 시점 참고값) |
| 검토 branch | `review/integration-20261105-semanticist21-next10` |

기본 경로는 maintainer 일반이며 접수·로컬 검증·다수 PR·시각 증적·후속 처리를 적용한다. 원 PR head·fork branch를 변경하지 않았다. [전체 범위·CI 근거·충돌 보정안](semanticist21_20261105_next10_review_plan.md)을 따른다.


## 원 기여 commit과 변경 범위

- `3c5accd7b604594585ebc1b1fe96ea155069821e` — fix(renderer): 글자 외곽선 종류를 한컴 인쇄 pen과 반복 구간으로 긋는다

제품 변경 경로: `src/renderer/skia/glyph_replay.rs`, `src/renderer/skia/renderer.rs`, `src/renderer/skia/text_replay.rs`, `src/renderer/stroke_pattern.rs`, `src/renderer/svg.rs`, `src/renderer/web_canvas.rs`.


## 기여의 의미와 메인테이너 보정

원 기여 commit을 수용 후보의 merge 조상으로 그대로 보존했다. PDF의 명시 TextPaint·alpha·width 0·join/miter와 글꼴 shaping을 보존해 충돌을 해결했다. 공통 pen/dash descriptor를 CanvasKit 소비에도 연결했다.

공통 적용 경로: HWP outline type → `renderer/stroke_pattern.rs::char_outline_stroke` → Native/SVG/Canvas 및 `paint/json.rs`의 `outlineStroke` → Studio CanvasKit. 명시 PDF TextPaint는 HWP 외곽선 합성의 비적용 경로다.

실제 CanvasKit 대조 fixture는 textRun 경로다. glyphRun 소비 코드의 존재를 별도 실제 fixture 실행으로 확대하지 않는다.

## 기존 해결 이력과 CI 처리 판단

Archive A는 checkout 단계에서 30분 상한으로 취소됐다. 컴파일·테스트는 시작되지 않았다. 최신 base의 유한 시간 예산을 유지한 메인터너 검증으로 처리한다.

원 CI의 녹색/실패 여부를 새 누적 후보의 검증으로 대체하지 않았다. 기존 실패 3건은 기여자 재작업 댓글보다 최신 base의 메인테이너 통합·실제 재검증으로 처리한다. workflow·timeout·탐지 기준을 변경하지 않았고 원격 재실행도 하지 않았다. 충돌 보정은 승인된 기존 계약 보존 방침을 따랐다.

## 실제 검증과 입력 동일성

현재 시각 producer는 `d53115fabc92bd392044d9b906c599baddb4368d`, base는 `6e1de406209ba9a58032f6ca6de45b18aa687d1c`다. 제품 source `584c9503baa651c62d0403dad9cb77c5cb9ffde6`의 마지막 계산 API 보정과 source 범위·추가 실행은 [전체 기록](assets/semanticist21-20261105-next10/validation.md)에 구분한다.

- 집중 보호 범위: Native/SVG/Canvas의 여섯 획, 실제 CanvasKit textRun 6종·투명 내부·4가지 dash, PDF 명시 TextPaint 7쪽.
- 독립 한컴 Print와 동일 입력의 전 8쪽: Native 최저 **100.00%**, fresh WASM 최저 **100.00%**. 90% 미만·누락·쪽수 불일치·2%p 초과 하락 없음.
- [입력·Print SHA와 전쪽 TSV](assets/semanticist21-20261105-next10/validation/post8622/summary.json), [문서별 글꼴 공급](assets/semanticist21-20261105-next10/font-qualification.md), [직접 확대 판독](assets/semanticist21-20261105-next10/validation/post8622/direct-reading.md), [대표 그림](assets/semanticist21-20261105-next10/images/post8622/outline-review.png).
- 그림의 실제 입력/쪽: **char_outline_types/outline.hwp 1쪽**. [이미지 SHA와 crop](assets/semanticist21-20261105-next10/validation/post8622/image-index.tsv)을 연결한다.
- 최신 devel에서 반영된 39 PR도 [80상태·406쪽 보존](assets/semanticist21-20261105-next10/validation/inherited8622/summary.json)과 실제 CLI/전체 회귀를 별도로 확인한다.

최종 필수 Rust·Native·Studio·fresh WASM 검사는 모두 통과했다. 정확한 계수·명령·source와 무시/건너뛴 기존 검사도 통합 검증 문서에 남겼다. contributor CI 주장·합성 계약 테스트와 독립 실물 시각 증거를 구분했다.

## Merge 후 contributor PR comment 계획

아직 게시하지 않았다. 원 PR별로 이 문서와 [후속 초안](assets/semanticist21-20261105-next10/comments/pr-8369.md)을 확인한 뒤 원 기여의 성과를 먼저 인정하고 메인테이너 보정의 이유·실제 검증 결과를 한국어 존댓말로 적는다.

이미지는 `assets/semanticist21-20261105-next10/images/post8622/outline-review.png`의 **char_outline_types/outline.hwp 1쪽**에 대응하는 그림을 사용한다. merge SHA 확정 후 `https://raw.githubusercontent.com/edwardkim/rhwp/<MERGE_SHA>/mydocs/pr/assets/semanticist21-20261105-next10/images/post8622/outline-review.png`로 고정하고, 실제 devel 포함·HTTP image 형식·해시를 확인한 뒤 게시한다. 임시 output이나 다른 PR의 대표 이미지로 대체하지 않는다. 게시 후 API 본문과 실제 화면을 확인한다. 통합 PR merge·원 PR 완료 반영·댓글·후속 기록·동기화·정리는 별도 원격 승인 뒤 수행한다.
