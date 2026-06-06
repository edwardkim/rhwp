# PR #1303 완료 보고서 - 미주 다줄 문단 다음 같은 미주 연속 문단 줄간격 과소 수정

## 1. 처리 개요

| 항목 | 내용 |
|---|---|
| PR | #1303 |
| 제목 | fix(#1302): 미주 다줄 문단 다음 같은 미주 연속 문단 줄간격 과소 수정 |
| 작성자 | planet6897 |
| 관련 이슈 | #1302 |
| 통합 브랜치 | `local/pr1303-integration` |

PR의 기능 변경은 수용하되, 기여자 작업 문서(`mydocs/plans`, `mydocs/working`, `mydocs/report`)는 제외했다. 실제 통합 대상은 다음 두 파일이다.

- `src/renderer/height_cursor.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 2. 통합 중 발견한 보강점

PR 원본 diff를 파일 단위로 가져오면 최신 `local/devel`의 `HeightCursor` 필드가 되돌아가 컴파일이 깨진다. 따라서 파일 전체 checkout이 아니라 PR의 최소 diff만 3-way로 적용했다.

또한 최신 `local/devel`에는 #1284 미주 tail/title PDF 정합 핀이 추가되어 있다. PR 원본 조건은 `curr_first_full_advance`를 문항 제목에도 적용해 `issue_1284_2024_between20_page13_question_flow_matches_pdf`를 실패시켰다.

보강:

- #1302 대상인 breakable 비제목 텍스트 연속 문단에는 full-advance gate 적용
- `current_is_endnote_title`인 문항 제목은 기존 title-bottom 보정 모델 유지
- 수식-only tail은 기존 PR 의도대로 atomic frame-fit backtrack 유지

## 3. 검증

통합 브랜치에서 실행:

```text
cargo fmt --all -- --check
통과

cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20_page13_question_flow_matches_pdf -- --nocapture
1 passed

cargo test --test issue_1139_inline_picture_duplicate issue_1302_2022_nov_page18_multiline_endnote_continuation_keeps_line_spacing -- --nocapture
1 passed

cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
68 passed

cargo clippy --lib -- -D warnings
통과

cargo test
통과
```

전체 테스트 주요 결과:

- lib tests: 1587 passed, 0 failed, 6 ignored
- `issue_1139_inline_picture_duplicate`: 68 passed
- doc test 포함 전체 명령 종료 코드 0

## 4. 시각 판정

메인테이너 시각 판정:

```text
2026-06-05 통과
```

검토 산출물:

- `output/poc/pr1303-endnote-spacing/hwp-page18/3-11월_실전_통합_2022_018.svg`
- `output/poc/pr1303-endnote-spacing/hwp-page18-debug/3-11월_실전_통합_2022_018.svg`

WASM 빌드 후 rhwp-studio 시각 판정도 통과했다.

## 5. 결론

PR #1303의 문제 인식과 수정 방향은 타당하다. 최신 `devel` 기준으로는 문항 제목 회귀를 막기 위해 `!current_is_endnote_title` guard를 추가한 형태로 수용하는 것이 안전하다.

권장 후속 절차:

1. 본 완료 보고서 승인
2. 통합 커밋 생성
3. `local/devel` 병합
4. `devel` 병합 및 테스트/푸시
5. PR #1303 및 이슈 #1302 종료 처리
