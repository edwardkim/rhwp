# Task M100-1329 최종 보고서 — 글머리표 Enter 직후 빈 줄 caret 위치

## 개요

- GitHub Issue: #1329 `rhwp-studio: 글머리표 Enter 직후 빈 줄 caret이 글머리표 앞에 표시됨`
- 브랜치: `issue-1329-bullet-caret`
- 후속 이슈: #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`

## 문제

rhwp-studio에서 글머리표 문단 끝에서 Enter를 누르면 새 문단에 같은 글머리표가 유지된다. 이때 실제 텍스트 입력은 글머리표 뒤 본문 위치에 정상 삽입되지만, 입력 전 caret은 글머리표 앞쪽에 표시됐다.

문서 모델의 삽입 위치와 화면에 표시되는 caret 위치가 달라 사용자에게 입력 위치가 잘못 보이는 문제였다.

## 원인

Enter 처리와 문단 분할 모델은 정상이었다.

- `SplitParagraphCommand`는 새 문단 `charOffset: 0`으로 이동한다.
- `Paragraph::split_at()`은 기존 `para_shape_id`를 새 문단에 복사하므로 글머리표 ParaShape가 유지된다.
- 번호/글머리표 marker는 문서 문자 좌표에 포함되지 않는 `char_start: None` TextRun으로 렌더링된다.

문제는 빈 list 문단의 cursor rect 계산이었다. 빈 list 문단에는 본문 TextRun이 없으므로 fallback 경로가 marker 시작점 또는 marker 앞쪽의 빈 body anchor를 반환할 수 있었다.

## 구현 내용

`src/document_core/queries/cursor_rect.rs`를 수정했다.

1. 본문 문단이 list 문단인지 ParaShape `HeadType`으로 판별한다.
   - `Outline`
   - `Number`
   - `Bullet`

2. list 문단의 빈 body anchor를 직접 hit으로 반환하지 않는다.
   - 빈 anchor는 marker 앞쪽 x에 놓일 수 있으므로 fallback으로 넘긴다.

3. fallback에서 첫 `TextLine` 하위 TextRun을 수집한다.
   - 본문 TextRun x
   - marker TextRun 오른쪽 끝 x

4. 빈 list 문단의 `charOffset: 0` caret x는 marker 뒤 본문 시작점으로 보정한다.

5. marker 폭은 실제 입력 스타일과 일치하도록 새 문단의 활성 char shape 기준으로 재측정한다.

## 테스트

`tests/issue_1329_bullet_caret.rs`를 추가했다.

검증 항목:

- 글머리표 문단 끝 Enter 후 새 빈 글머리표 문단의 caret x
- 새 빈 글머리표 문단에 실제 글자 입력 후 첫 글자 시작 x와 입력 전 caret x 일치
- 번호 문단에 같은 검증 적용
- 일반 빈 문단 cursor x 회귀 방지

## 검증 결과

통과한 명령:

```bash
cargo fmt --all -- --check
cargo test --test issue_1329_bullet_caret
cargo test --test issue_1308_forced_break_hanging_indent
cargo test --lib
wasm-pack build --target web
cd rhwp-studio && npm run build
```

주요 결과:

```text
cargo test --test issue_1329_bullet_caret
test result: ok. 3 passed; 0 failed
```

```text
cargo test --test issue_1308_forced_break_hanging_indent
test result: ok. 8 passed; 0 failed
```

```text
cargo test --lib
test result: ok. 1613 passed; 0 failed; 6 ignored
```

```text
rhwp-studio npm run build
✓ built
```

로컬 서버도 실행해 확인했다.

```bash
cd rhwp-studio
npm run dev -- --host 127.0.0.1 --port 7700
```

서버 응답:

```text
HTTP/1.1 200 OK
```

작업지시자가 `http://127.0.0.1:7700/`에서 직접 수동 검증했고, #1329의 의도한 동작은 정상으로 확인했다.

## 후속 이슈

수동 검증 중 새 현상이 확인됐다.

- Enter로 빈 글머리표 줄을 만든 뒤 텍스트를 입력하면 marker와 caret/입력 글자 크기가 커져 보인다.

이는 #1329의 caret x 좌표 문제와 별개로, 빈 list 문단의 marker/caret 스타일 기준이 입력 전후로 달라지는 문제다. 범위를 분리해 후속 이슈로 등록했다.

- #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`

## 변경 파일

- `src/document_core/queries/cursor_rect.rs`
- `tests/issue_1329_bullet_caret.rs`
- `mydocs/orders/20260608.md`
- `mydocs/plans/task_m100_1329.md`
- `mydocs/plans/task_m100_1329_impl.md`
- `mydocs/working/task_m100_1329_stage1.md`
- `mydocs/working/task_m100_1329_stage3.md`
- `mydocs/working/task_m100_1329_stage4.md`
- `mydocs/report/task_m100_1329_report.md`

## 결론

#1329는 구현과 검증을 완료했다. caret은 Enter 직후 빈 글머리표/번호 문단에서도 marker 뒤 본문 시작점에 표시되고, 실제 입력 후 첫 글자 시작 위치와 일치한다.

다음 단계는 작업지시자 승인 후 커밋, push, PR 작성이다.
