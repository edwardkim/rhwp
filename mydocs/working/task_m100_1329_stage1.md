# Task M100-1329 Stage 1 진단 보고서 — 글머리표 빈 줄 caret 위치

## 범위

- 대상 이슈: #1329 `rhwp-studio: 글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨`
- 브랜치: `issue-1329-bullet-caret`
- 수행 범위: 소스 수정 없이 Enter 처리 경로, 문단 분할 모델, 렌더 트리, cursor rect 계산 경로를 조사했다.
- 한컴 직접 편집기 검증은 로컬에 `Hancom Office HWP Viewer.app`만 있어 수행하지 못했다. 한컴 도움말 기준으로 Enter 후 글머리표가 이어지는 동작과 marker 영역/본문 영역 분리 정책만 참고했다.

## 사용자 관찰 요약

글머리표 문단 끝에서 Enter를 누르면 새 문단에 같은 글머리표가 유지된다. 이때 입력 전 caret은 글머리표 앞쪽에 보이지만, 실제 타이핑을 시작하면 글자는 글머리표 뒤 본문 시작 위치에 들어간다.

따라서 현재 현상은 새 문단 생성이나 실제 삽입 위치의 결함이 아니라, 빈 글머리표 문단의 caret 표시 좌표가 실제 본문 삽입 좌표와 다른 결함으로 판단한다.

## 코드 경로 확인

Enter 처리:

- `rhwp-studio/src/engine/input-handler-keyboard.ts:1050`에서 plain Enter를 처리한다.
- 셀 밖에서는 `SplitParagraphCommand`, 셀 안에서는 `SplitParagraphInCellCommand`를 실행한다.

본문 문단 분할:

- `rhwp-studio/src/engine/command.ts:293`의 `SplitParagraphCommand.execute()`는 문단 분할 성공 후 새 문단으로 이동한다.
- 반환 cursor 위치는 `paragraphIndex: result.paraIdx`, `charOffset: 0`이다.
- 즉 새 문단의 문서 좌표상 삽입 위치는 문단 시작점이다.

문단 모델:

- `src/model/paragraph.rs:689`의 `Paragraph` 생성부는 `para_shape_id: self.para_shape_id`를 복사한다.
- 따라서 분할된 새 문단은 기존 글머리표 ParaShape를 유지한다.

렌더링:

- `src/renderer/layout/paragraph_layout.rs:2575`에서 번호/글머리표 marker는 별도 `TextRun`으로 렌더링된다.
- 해당 marker `TextRun`은 `char_start: None`이다.
- marker 폭을 더한 뒤 `x += num_width`로 본문 run 배치가 시작된다.

본문 cursor rect:

- `src/document_core/queries/cursor_rect.rs:593`의 본문 cursor 탐색은 `char_start: None`인 번호/글머리표 `TextRun`을 건너뛴다.
- `src/document_core/queries/cursor_rect.rs:800` 이후 fallback은 TextRun에서 못 찾은 경우 첫 `TextRun` 또는 `TextLine`을 찾아 x/y/height를 반환한다.
- 텍스트가 없는 빈 list 문단에서는 본문 `TextRun`이 없으므로 `TextLine.x`가 사용될 수 있고, 이 값은 marker 뒤 본문 시작점이 아니라 marker 시작점이다.

## 렌더 트리 기준선

`rhwp-studio/public/samples/number-bullet.hwp`를 기준으로 렌더 트리를 추출했다.

사용 명령:

```bash
cargo run --quiet --bin rhwp -- diag rhwp-studio/public/samples/number-bullet.hwp
cargo run --quiet --bin rhwp -- dump rhwp-studio/public/samples/number-bullet.hwp --section 0
cargo run --quiet --bin rhwp -- export-render-tree rhwp-studio/public/samples/number-bullet.hwp -o /private/tmp/rhwp1329_tree -p 0
```

문서 진단 결과:

- `number-bullet.hwp`는 Bullet 정의 4개를 가진다.
- ParaShape 분포는 `None: 12개, Outline: 5개, Number: 0개, Bullet: 4개`로 확인됐다.
- `dump` 결과 list 문단 본문 텍스트에는 marker 문자가 포함되지 않고, marker는 ParaShape metadata로 유지된다.

렌더 트리 x 좌표:

| 문단 | TextLine x | marker | marker x | marker w | body x |
|------|------------|--------|----------|----------|--------|
| 1 | 70.0 | `☑ ` | 70.0 | 28.0 | 98.0 |
| 2 | 83.3 | `• ` | 83.3 | 13.0 | 96.3 |
| 6 | 110.0 | `* ` | 110.0 | 13.0 | 123.0 |

세 문단 모두 `body x = TextLine x + marker w` 관계가 성립한다. 빈 글머리표 문단에서 caret이 `TextLine x`를 쓰면 marker 앞/marker 위치에 보이고, 실제 입력은 `body x`부터 시작하는 현상이 발생한다.

번호 문단 샘플도 확인했다.

사용 명령:

```bash
cargo run --quiet --bin rhwp -- diag rhwp-studio/public/samples/para-head-num-2.hwp
```

`para-head-num-2.hwp`는 Numbering 4개와 Number head 문단을 가진다. 글머리표와 같은 `numbering_text` 렌더링 경로를 쓰므로 번호 문단도 동일 원칙으로 검증 대상에 포함해야 한다.

## 셀/중첩 표 영향

셀 문단 렌더링도 번호/글머리표 적용 경로를 탄다.

- `src/renderer/layout/table_cell_content.rs:681`에서 셀 문단에 `apply_paragraph_numbering()`을 적용한다.
- `src/renderer/layout/table_layout.rs:2064`에서도 표 레이아웃 중 `apply_paragraph_numbering()`을 적용한다.

다만 셀 cursor rect 경로는 본문과 다르게 marker 예외 처리가 충분하지 않다.

- `src/document_core/queries/cursor_rect.rs:1996`의 `get_cursor_rect_in_cell_native()`는 `text_run.char_start.unwrap_or(0)`을 사용한다.
- `char_start: None`인 marker TextRun도 offset 0으로 매칭될 수 있다.
- `src/document_core/queries/cursor_rect.rs:2060` 이후 빈 셀 fallback은 해당 셀의 아무 `TextRun` x를 반환하므로 marker TextRun을 반환할 수 있다.
- `src/document_core/queries/cursor_rect.rs:2367`의 경로 기반 중첩 표 cursor rect도 `tr.char_start.unwrap_or(0)`을 사용한다.

즉 셀/중첩 표 list 문단에도 같은 계열 결함 가능성이 있다. 다만 사용자 재현은 rhwp-studio 본문 편집 화면이므로 Stage 2에서 본문만 우선 보정할지, 셀/경로 기반 cursor rect까지 같은 원칙으로 포함할지 범위를 확정해야 한다.

## 결론

Stage 1 결론은 다음과 같다.

1. Enter 처리와 문단 분할 모델은 정상이다. 새 문단은 `charOffset: 0`으로 이동하고 기존 `para_shape_id`를 복사해 글머리표 속성을 유지한다.
2. 번호/글머리표 marker가 문서 문자 좌표에 포함되지 않는 것도 정상 설계다. marker는 `char_start: None`인 별도 `TextRun`이다.
3. 결함 후보는 cursor rect 계산이다. 특히 빈 list 문단에서 본문 anchor가 없을 때 fallback이 marker 시작 x인 `TextLine.x`를 반환할 수 있다.
4. 실제 입력 위치와 표시 caret 위치가 다른 이유는 `TextLine.x`와 본문 시작 x 사이에 marker 폭만큼 차이가 있기 때문이다.
5. 번호 문단도 같은 렌더링 구조이므로 구현 계획에서 함께 검증해야 한다.
6. 셀/중첩 표 cursor rect는 본문보다 marker 예외 처리가 약하므로 영향 조사 또는 후속 분리가 필요하다.

## 구현 방향 판정

후보 A, 즉 cursor rect 보정을 우선 구현 방향으로 잡는 것이 적절하다.

- 빈 번호/글머리표 문단에서 `charOffset: 0`의 caret x는 marker 시작점이 아니라 marker 뒤 본문 시작점이어야 한다.
- 렌더 트리에는 marker `TextRun`의 bbox가 이미 존재하므로, 가능하면 렌더 트리 기준으로 marker 폭을 반영하는 방식이 렌더링 결과와 가장 잘 맞는다.
- marker 자체를 문서 문자 좌표에 포함시키거나 렌더 트리 구조를 크게 바꾸는 후보 B는 현재 결함 범위에 비해 영향도가 크다.

Stage 2 구현 계획서에서는 다음을 확정한다.

- 본문 빈 list 문단 cursor rect 보정 방식
- 번호 문단 포함 여부와 테스트 케이스
- 셀/중첩 표 cursor rect를 이번 PR에 포함할지, 후속 이슈로 분리할지
- 자동 회귀 테스트 작성 방식

## 검증 기준 제안

자동 검증은 다음 기준을 권장한다.

1. 기존 `number-bullet.hwp` 또는 작은 테스트 문서를 사용해 글머리표 문단의 marker x, marker width, body x를 확인한다.
2. 빈 글머리표 문단의 `charOffset: 0` cursor rect가 marker x가 아니라 body x에 가까운지 검증한다.
3. 번호 문단 샘플 `para-head-num-2.hwp`에 대해 같은 원칙을 검증한다.
4. 일반 빈 문단의 cursor rect가 기존 line start 정책을 유지하는지 확인한다.
5. 셀/중첩 표를 포함할 경우 `char_start: None` marker TextRun이 offset 0으로 매칭되지 않는지 별도 테스트한다.

## 다음 단계

Stage 2에서 `mydocs/plans/task_m100_1329_impl.md` 구현 계획서를 작성한다. 구현 계획서 승인 전에는 소스 수정을 진행하지 않는다.
