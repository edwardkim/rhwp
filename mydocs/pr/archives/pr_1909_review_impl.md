# PR #1909 Review Impl — task 667: ingest 스키마 확장

## Stage 1. 코드 변경 검토

완료.

- ingest serde 모델과 JSON schema가 같은 필드 집합을 갖는지 확인했다.
- 새 필드는 모두 optional/default 처리되어 기존 v1 JSON 호환을 유지한다.
- shared passage는 `passages[].id`와 `Question.passage_ref`로 연결된다.
- `boxed` block은 `StemBlock::Boxed`로 표현된다.

## Stage 2. builder 매핑 검토

완료.

- 같은 passage는 첫 참조 위치에서 한 번만 출력된다.
- 존재하지 않는 passage 참조는 placeholder 문단으로 남겨 schema 오류 위치를 추적할 수 있다.
- `boxed` block은 `border_fill_id=2` paragraph shape로 출력된다.
- `header_text`, `footer_text`, `form_label`은 Header/Footer control로 매핑된다.

## Stage 3. 문서와 샘플 검토

완료.

- `rhwp-exam-ingest` skill 문서가 `passages`/`passage_ref`/`boxed` 사용법을 안내한다.
- `cli_commands.md`가 새 필드와 `sample_structured.json`을 안내한다.
- `sample_structured.json`은 공유 지문, 보기 박스, 머리말/꼬리말 예제를 포함한다.

## Stage 4. 검증

완료.

- JSON 유효성, focused unit test, focused clippy, CLI smoke를 수행했다.
- 검증 세부 명령과 결과는 `pr_1909_review.md`에 기록했다.

## Stage 5. merge 전 확인

대기.

- PR head 최신 커밋 기준 GitHub Actions 통과 확인
- 작업지시자 merge 승인 확인
- merge 후 #667 auto-close 여부 확인 및 후속 코멘트 수행
- merge 후 브랜치 정리

