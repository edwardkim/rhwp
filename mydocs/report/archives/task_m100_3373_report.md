# task_m100_3373 처리결과 보고서 — `edit replace-text` (Stage 3 두 번째 조각, 커버리지 G1)

- **이슈**: [#3373](https://github.com/edwardkim/rhwp/issues/3373)
- **브랜치**: `pr/task-edit-replace-text` (**PR #3345 위 적층** — #3347 과 같은 방식)
- **범위**: `src/main.rs`(edit_replace_text + 디스패치 1행 + help + capabilities/MCP 등재),
  `tests/edit_replace_text_contract.rs`(신규), `mydocs/manual/cli_commands.md`(신설 항목)
- **분류**: 기능 추가 (편집 — 로드맵 #2659 §7.3, 커버리지 체크리스트 #3370 G1)

## 1. 배경

기관명 변경·연도 갱신·용어 정비 같은 **일괄 치환**은 공문 개정의 일상 업무인데 CLI 로
불가했다(체크리스트 8행 ✕). 개인정보 마스킹(13행)도 치환 부재로 막혀 있었다. 코어에는
스튜디오가 쓰는 검증된 치환 스위트(`replace_all_native` — 역순 치환으로 오프셋 안전,
본문+표 셀)가 이미 있고 wasm_api `replace_all` 로 노출돼 있다.

## 2. 설계 결정

- **새 편집 로직 0줄** — `replace_all` 을 그대로 부른다 (#3345 fill-fields 와 동일 원칙).
- **`--dry-run` 은 파일 생성 경로를 타지 않는다** — 읽기 전용 `grep` 으로 치환 예정
  건수만 보고. 계약 테스트가 파일 부재를 단언한다.
- **치환 0건이면 출력 파일을 만들지 않는다** — 무변경 산출물 금지. `output` 필드도
  실제 저장 시에만 실린다.
- **실패 시 원본 불변** — 치환·직렬화·쓰기 실패 시 출력 없이 exit 1.
- **재독 검증이 계약** — 산출물을 `search --json` 으로 다시 읽어 원문 0건·새 문자열
  N건을 대조하는 루프를 테스트로 고정했다 (보고만 믿지 않음).
- `--find` 빈 문자열은 exit 2 (전 문서 폭주 방지), `--replace ""` 는 삭제로 허용.
- 파싱은 #3349 규약(위치 무관·미지 플래그 즉시 exit 2·중복 positional exit 2).

## 3. 검증

- **계약 테스트 5종 red→green**: 치환+재독 대조(search 독립 출처로 기대값 산출) /
  dry-run 파일 부재+건수 일치 / **0건 파일 부재** / 인자 누락·빈 --find exit 2 (4형) /
  없는 파일 exit 1 + 출력 미생성
- **무회귀**: `edit_fill_fields_contract`(7), `cli_json_contract`(22) 전부 green
- `cargo fmt` clean, clippy `--bin rhwp -- -D warnings` 0건 (release-test)
- **실측 루프**: field-01.hwp 에서 "마케팅"→"영업" 1건 치환 → 재독 원문 0건 →
  렌더에서 제목이 "영업 전략 기획서"로 바뀐 것 확인 (전/후 문서 사진
  `assets/task_m100_3373/`, 서식 완전 보존)

## 4. 남긴 것

- `edit set-cell`(G2) — 같은 규약으로 다음 조각. 표 좌표는 export-tables 격자와 동일
  좌표계 사용 예정.
- 개인정보 마스킹 시나리오(체크리스트 13행)는 본 조각으로 △→○ 승격 가능 — 예제집에
  마스킹 예제 추가는 #3370 쪽 후속.
- 머리말/꼬리말·각주 내부 텍스트는 `search_all` 범위(본문+표셀+글상자)를 따른다 —
  fields 와 같은 한계로 문서에 명시됨.