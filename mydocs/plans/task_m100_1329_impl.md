# Task M100-1329 구현 계획서 — 글머리표 빈 줄 caret 위치 보정

## 이슈

- GitHub Issue: #1329 `rhwp-studio: 글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨`
- 브랜치: `issue-1329-bullet-caret`
- 선행 문서:
  - 수행계획서: `mydocs/plans/task_m100_1329.md`
  - Stage 1 진단 보고서: `mydocs/working/task_m100_1329_stage1.md`

## 구현 결론

이번 구현은 본문 문단의 `get_cursor_rect_native()` fallback을 보정한다.

Enter 처리, 문단 분할, ParaShape 복사는 정상이다. 문제는 빈 번호/글머리표 문단에서 본문 `TextRun`이 없을 때 cursor rect fallback이 `TextLine.x`를 반환할 수 있다는 점이다. list 문단에서 `TextLine.x`는 marker 시작점이고, 실제 본문 시작점은 marker `TextRun`의 오른쪽 끝이다.

따라서 빈 list 문단의 `charOffset: 0` caret x는 다음 원칙으로 계산한다.

```text
일반 빈 문단: 기존 TextLine.x 유지
번호/글머리표/개요 빈 문단: marker TextRun bbox.x + bbox.width 사용
```

## 구현 범위

포함:

- 본문 `DocumentCore::get_cursor_rect_native()`의 빈 문단 fallback 보정
- `HeadType::Outline`, `HeadType::Number`, `HeadType::Bullet` 문단 대상 처리
- 글머리표 샘플과 번호 샘플 기반 자동 회귀 테스트
- 일반 빈 문단 cursor rect 회귀 확인

제외:

- Enter command 동작 변경
- `Paragraph::split_at()` 모델 변경
- 번호/글머리표 marker를 문서 문자 좌표에 포함시키는 구조 변경
- 글머리표/번호 UI 개편
- 한컴의 list continuation, 자동 번호 해제, level 증감 규칙 구현

조건부 제외:

- 셀/중첩 표 list 문단은 이번 PR의 필수 범위에서 제외한다.
- Stage 1에서 `get_cursor_rect_in_cell_native()`와 `get_cursor_rect_by_path_native()`에도 같은 계열 결함 가능성이 확인됐다. 다만 별도 cursor API, 별도 cell path fixture, hit-test 연계 영향이 있어 본문 결함 수정과 분리하는 편이 안전하다.
- 구현 중 동일 helper를 매우 좁게 재사용할 수 있고 회귀 테스트를 단순히 추가할 수 있으면 포함한다. 그렇지 않으면 최종 보고서에 후속 이슈 후보로 명시한다.

## 수정 대상 파일

### `src/document_core/queries/cursor_rect.rs`

주 수정 파일이다.

1. 본문 문단이 list 문단인지 판별하는 보조 로직을 추가한다.
   - 대상 문단의 `para_shape_id`를 읽는다.
   - `self.styles.para_styles[para_shape_id].head_type`을 확인한다.
   - `HeadType::Outline | HeadType::Number | HeadType::Bullet`이면 list 문단으로 본다.

2. 빈 문단 fallback의 `find_para_line()`을 보강한다.
   - 기존 반환값 `(x, y, h)`만으로는 marker와 body 시작점을 구분할 수 없다.
   - 반환 구조를 내부 struct로 바꿔 다음 정보를 담는다.
     - `line_x`
     - `line_y`
     - `line_h`
     - `first_body_x`: `char_start.is_some()`인 첫 본문 `TextRun` x
     - `marker_end_x`: list 문단이고 `char_start.is_none()`인 marker `TextRun`의 `bbox.x + bbox.width`
   - 우선순위:
     1. `first_body_x`가 있으면 기존처럼 본문 `TextRun` x 사용
     2. `first_body_x`가 없고 list 문단이며 `marker_end_x`가 있으면 `marker_end_x` 사용
     3. 그 외에는 기존처럼 `line_x` 사용

3. `char_start: None`을 무조건 marker로 취급하지 않는다.
   - 안내문 등 다른 `char_start: None` TextRun이 있으므로 반드시 문단의 `HeadType`이 list 계열인 경우에만 marker fallback 후보로 사용한다.
   - 본문 문단만 대상으로 하므로 `text_run.cell_context.is_none()` 조건은 유지한다.

4. 기존 인라인 도형 fallback 보정은 유지한다.
   - `char_offset > 0`일 때 inline bbox를 이용해 x를 조정하는 기존 로직은 보존한다.
   - list 보정은 `charOffset: 0` 빈 문단 표시 위치 보정이 목적이다.

### `tests/issue_1329_bullet_caret.rs`

새 integration test 파일을 추가한다.

테스트 1: 글머리표 문단 split 후 빈 list 문단 cursor x

- `rhwp-studio/public/samples/number-bullet.hwp` 로드
- 문단 1 `마스크 착용 의무`의 길이를 `get_paragraph_length_native(0, 1)`로 구한다.
- `split_paragraph_native(0, 1, len)`으로 문단 끝 Enter를 재현한다.
- 새 문단은 2번 문단이 되며 텍스트가 없는 글머리표 문단이어야 한다.
- 기존 문단 1의 `get_cursor_rect_native(0, 1, 0)` x는 marker 뒤 body 시작점이다.
- 새 빈 문단 2의 `get_cursor_rect_native(0, 2, 0)` x도 같은 list level의 body 시작점 범위에 있어야 한다.
- 실패 기준은 새 문단 x가 marker 시작점 쪽으로 되감기는 경우다.

테스트 2: 번호 문단 split 후 빈 list 문단 cursor x

- `rhwp-studio/public/samples/para-head-num-2.hwp` 로드
- Number head 문단 하나를 끝에서 split한다.
- 새 빈 번호 문단의 `charOffset: 0` cursor x가 marker 뒤 본문 시작점에 위치하는지 확인한다.

테스트 3: 일반 빈 문단 회귀

- `saved/blank2010.hwp` 로드 후 `convert_to_editable_native()`
- 일반 텍스트 문단 끝에서 split해 빈 일반 문단을 만든다.
- `get_cursor_rect_native()`가 성공하고, x가 비정상적으로 오른쪽으로 밀리지 않는지 확인한다.

보조 함수:

- `load_doc(rel: &str) -> HwpDocument`
- `cursor_x(doc, para, offset) -> f64`
- `assert_close_or_after_body_start(actual, expected_body_x, context)`

## 테스트 판정 기준

글머리표/번호 테스트는 절대 좌표에 과도하게 의존하지 않는다.

- split 전 같은 list 문단의 `offset 0` cursor x를 body 시작 기준값으로 사용한다.
- split 후 빈 list 문단의 `offset 0` cursor x가 기준값과 매우 가까운지 확인한다.
- 페이지/줄 높이 차이 때문에 y는 보조 확인만 한다.
- x 비교 허용 오차는 렌더 트리 좌표가 0.1 단위 JSON으로 반환되는 점을 고려해 `<= 1.0px` 수준으로 둔다.

## 수동 검증

구현 후 rhwp-studio에서 다음을 확인한다.

1. `rhwp-studio/public/samples/number-bullet.hwp` 열기
2. 글머리표 문단 끝에 커서 배치
3. Enter 입력
4. 새 빈 글머리표 문단의 caret이 marker 뒤 본문 시작점에 보이는지 확인
5. 글자를 입력했을 때 입력 전 caret 위치와 첫 글자 위치가 일치하는지 확인
6. 번호 문단 샘플 `para-head-num-2.hwp`에서도 같은 절차 확인
7. 일반 빈 문단 Enter 반복이 기존처럼 동작하는지 확인

rhwp-studio 명령:

```bash
cd rhwp-studio
npm run build
npm run dev
```

브라우저 검증은 dev server 실행 후 수행한다. 의존성이 없거나 브라우저 자동화가 막히면 수동 검증 절차와 실패 사유를 Stage 3 보고서에 남긴다.

## 검증 명령

구현 후 최소 실행:

```bash
cargo fmt --all -- --check
cargo test --test issue_1329_bullet_caret
cargo test --test issue_1308_forced_break_hanging_indent
```

가능하면 추가 실행:

```bash
cargo test --lib
cd rhwp-studio && npm run build
```

## 리스크와 대응

1. `char_start: None` TextRun 오인식
   - 대응: ParaShape `HeadType`이 list 계열일 때만 marker fallback으로 사용한다.

2. 번호 문단 counter 상태와 split 후 marker 폭 변화
   - 대응: 번호 테스트는 절대 x가 아니라 split 전 같은 문단의 body 시작 x와 비교한다.

3. 일반 빈 문단 x 회귀
   - 대응: 일반 빈 문단 테스트를 추가하고, `HeadType::None`이면 기존 `TextLine.x` fallback을 유지한다.

4. 셀/중첩 표 미해결
   - 대응: 이번 PR은 본문 결함을 우선 해결한다. 셀/경로 기반 cursor rect는 후속으로 분리하거나, 구현 중 안전하게 포함 가능할 때만 함께 처리한다.

5. `split_paragraph_native()` 반환 JSON의 `paraIdx` 의미 혼선
   - 대응: 이번 테스트는 반환 JSON에 의존하지 않고, split 대상 문단의 다음 인덱스를 새 문단으로 직접 사용한다.

## 산출물

Stage 3 구현 후 산출물:

- 소스 수정
  - `src/document_core/queries/cursor_rect.rs`
- 테스트 추가
  - `tests/issue_1329_bullet_caret.rs`
- Stage 3 완료 보고서
  - `mydocs/working/task_m100_1329_stage3.md`

## 승인 요청

본 구현 계획서를 승인하면 Stage 3 구현을 진행한다. Stage 3에서는 위 범위에 따라 소스와 테스트를 수정한다.
