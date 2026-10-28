# Task M100 #1434 — 2단계 완료 보고서 (저장→재파싱 정합 + 전수/CI)

- 브랜치: `local/task1434`
- 작성일: 2026-06-19
- 추가 파일: `tests/issue_1434_clickhere_guide_hancom_command.rs`

## 1. round-trip 통합 테스트

`tests/issue_1434_clickhere_guide_hancom_command.rs` 신규 — 누름틀 삽입 → HWP 저장 →
재파싱 정합 검증.

- `clickhere_command_is_hancom_format_without_name_key`: 삽입 직후 command 가 한컴
  정답지 동형(`Clickhere:set:48:...HelpState:wstring:0:  `)이고 Name 키 부재.
- `clickhere_guide_memo_name_survive_hwp_roundtrip`: `export_hwp_native` → `from_bytes`
  재파싱 후 guide_text(여기에 입력)·memo_text(도움말)·field_name(회사명=CTRL_DATA) 보존,
  재파싱 command 에도 Name 키 부재.
  - `RHWP_ISSUE1434_OUT` env 지정 시 한컴 판정용 HWP 산출(3단계용).

## 2. 검증

- `cargo test --test issue_1434_clickhere_guide_hancom_command`: **2/2 passed**.
- CI급 `cargo test --profile release-test --tests`: 전체 그린(FAILED 0).
- `cargo fmt --check`: 정렬 1건 적용 후 CLEAN.
- `cargo clippy --all-targets`: **0 warnings/errors**.

## 3. 한컴 판정 샘플 산출 (3단계 입력)

`output/poc/task1434/clickhere-guide-여기에입력.hwp` 생성. 누름틀 command:
```
Clickhere:set:51:Direction:wstring:6:여기에 입력 HelpState:wstring:3:도움말  
```
Name 키 부재 + HelpState 뒤 공백 2개 + set=inner−1 — 한컴 정답 포맷 정합.
(memo "도움말" 3글자 포함이라 set:51, 빈 memo 면 set:48.)

## 4. 다음 단계

- 3단계: **작업지시자 Windows 한컴 2020/2022 편집기**에서 위 샘플 열어 안내문 바인딩
  판정(자기 검증 ≠ 한컴 호환, 필수 게이트) + 트러블슈팅·최종 보고서.
