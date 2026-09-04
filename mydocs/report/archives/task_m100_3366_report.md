# task_m100_3366 처리결과 보고서 — thumbnail 종료 코드·파싱 계약 정합

- **이슈**: [#3366](https://github.com/edwardkim/rhwp/issues/3366)
- **브랜치**: `pr/fix-issue-3366-thumbnail-contract` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/main.rs`(`extract_thumbnail` 파싱부·종료 경로 + 디스패치 1행),
  `tests/issue_3366_thumbnail_contract.rs`(신규)
- **분류**: 버그 수정 (CLI 계약 — #2707/#3349 정합)

## 1. 배경

658건 실문서 스윕 중 `thumbnail` 이 형제 명령들과 다른 계약 밖 동작 4건을 보였다
(v0.8.0 실측):

| 호출 | 실측 | 계약 |
|---|---|---|
| `thumbnail FILE --no-such-option -o t.png` | **exit 0 + 산출물 생성** | 즉시 exit 2, 산출물 없음 |
| `thumbnail` (인자 없음) | exit 1 | exit 2 (#2707 사용법 오류) |
| `thumbnail FILE -o` (값 누락) | exit 0, 조용히 무시 | exit 2 |
| `thumbnail --base64 FILE` | exit 1 (--base64 가 파일이 됨) | 동작 (#3349 위치 무관) |

원인: `extract_thumbnail()` 이 자체 `std::process::exit` 경로에 기본 암 `_ => {}`(무시),
`args[0]` 파일 강제를 쓰고 있었다 — CLI 초기의 잔재.

참고: 스윕의 thumbnail '실패' 113건은 전부 Preview 스트림이 실제로 없는 파일의 계약대로의
exit 1 이었고(한컴 원본 HWPX 는 Preview/PrvImage.png 를 정상 추출 — 755×1024 실측),
이번 수정 범위가 아니다.

## 2. 설계 결정

- **파싱을 #3349 규약으로 통일** — 위치 무관, 미지 플래그 즉시 exit 2, 중복 positional
  exit 2, `-o` 값 누락 exit 2. 옵션 처리 암(`--base64`/`--data-uri`/`-o`)은 무변경.
- **반환형 i32 + `exit_with` 디스패치** — 형제 명령들과 같은 종료 경로. 인자 없음 =
  EXIT_USAGE(2), 파일 읽기 실패·썸네일 부재·저장 실패 = EXIT_RUNTIME(1) 유지.
- **성공 경로 무변경** — 파일/base64/data-uri 3모드 출력·기본 파일명 규칙 그대로.

## 3. 검증

- **회귀 테스트 7종** (red→green): 미지 옵션 exit 2 + **산출물 미생성**(종전 최악 사례) /
  인자 없음 exit 2 / `-o` 값 누락 exit 2 / 옵션 선행 동작(base64) / 중복 positional exit 2 /
  정상 추출 무회귀 / 썸네일 부재 exit 1 유지(HWP3)
- 무회귀: `cli_exit_codes` green, `cargo fmt` clean, clippy `-D warnings` 0건
- **문서 증거**: `assets/task_m100_3366/extracted-preview.png` — 수정 빌드로 한컴 원본
  HWPX 에서 추출한 실제 문서 미리보기 (`rhwp thumbnail 편람.hwpx -o …`)

## 4. 남긴 것

- 라운드트립 스윕에서 발견한 별개 결함 2건은 이슈로 분리: #3367(secd/cold 순서 스왑),
  #3368(ParaShape ml 반올림 손실).
