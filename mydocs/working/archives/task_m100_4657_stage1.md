# Issue #4657 Stage 1 — 배분 정렬 문단의 오른쪽 끝 정렬 수정

## 목표

HWPX `DISTRIBUTE`(배분 정렬) 문단을 SVG 로 렌더링할 때, 글자 수가 다른 문단들의
마지막 글자(콜론) 오른쪽 끝이 한컴처럼 문단 폭에 정확히 닿게 한다.

## 원인

`compute_line_extra_spacing` 의 배분 분기(`src/renderer/layout/paragraph_layout.rs`)가
남는 폭을 글자 수 N 으로 나눠 `extra_char_spacing` 으로 분배했다. `extra_char_spacing`
은 각 글자의 advance 뒤에 붙으므로 마지막 글자의 잉크 오른쪽 끝은 `W + (N-1)·extra`
지점이다. `extra = slack/N` 이면 마지막 글자가 오른쪽 여백에서 `slack/N` 만큼 안쪽으로
밀리고, N 이 줄마다 달라 문단 간 오른쪽 끝이 어긋난다. 짧은 문구일수록 slack/N 이 커서
차이가 두드러진다 — 이슈 보고 증상과 일치.

## 수정

1. 배분 슬랙을 글자 **사이** 간격(N-1곳) 기준으로 분배: `raw = slack / (N-1)`.
   마지막 글자 오른쪽 끝이 줄 길이와 무관하게 문단 폭에 닿는다.
2. 말미 공백은 보이는 글자가 아니므로 분배 대상(글자 수)과 기준 폭에서 제외
   (needs_justify 분기의 후행 공백 제외와 동일 규칙).
3. 보이는 글자가 1자 이하면 분배하지 않음 (0-division 가드).

## 재현 픽스처

이슈 첨부 파일이 없어 이슈 본문의 임의 예시 문구 5개("문서관리번호 :" 등)로 최소
재현 HWPX 를 합성했다: `samples/issue4657/distribute-alignment-sample.hwpx`
(ref_text.hwpx 기반, paraPr `DISTRIBUTE` + 오른쪽 여백 30000 HWPU, 문단 5개).

## 검증 실측

`rhwp export-svg` 좌표 실측 (같은 픽스처, 수정 전후 동일 명령):

| 문단 (글자 수) | 수정 전 콜론 x | 수정 후 콜론 x |
| --- | --- | --- |
| 문서관리번호 : (8) | 440.85 | 475.59 |
| 처리기관번호 : (8) | 440.85 | 475.59 |
| 기관명 : (5) | 412.40 | 475.79 |
| 소재지 : (5) | 412.40 | 475.79 |
| 적용분류 : (6) | 425.06 | 475.72 |

- 시작 x 는 전후 모두 113.39 로 동일.
- 수정 전: 콜론 x 편차 최대 28.4px. 수정 후: 0.2px 이내.
- 전후 SVG: `mydocs/pr/assets/issue4657_distribute_before.svg` / `_after.svg`.

## 테스트

- 단위 (`paragraph_layout.rs` `issue_4657_distribute_alignment_tests`):
  N-1 분배로 오른쪽 끝이 문단 폭에 닿음(글자 수 8/5 두 케이스), 말미 공백 제외,
  1자 가드. 3건.
- 통합 (`tests/issue_4657_distribute_alignment.rs`): 픽스처 SVG 의 줄별 min/max x 를
  문단 간 상대 비교(절대좌표 단언 금지 — #3458). 수정 전 코드에서 Δ28.4px 로 실패,
  수정 후 통과함을 stash 교차 실행으로 확인.

## 검증 게이트 결과

| 검증 | 결과 |
| --- | --- |
| 변경 파일 rustfmt --check (LF 사본) | 통과. 이 장비 checkout 은 `core.autocrlf=true` 로 CRLF 라 저장소 전체 `cargo fmt --check` 는 newline_style 로 전면 실패 — 환경 요인, CI fmt gate 로 대체 |
| `cargo clippy --all-targets` | 경고 없이 통과 |
| `cargo test --lib alignment` | 17개 통과 (신규 3건 포함) |
| `cargo test --test issue_4657_distribute_alignment` | 1개 통과 (수정 전 코드에서는 실패 확인) |
| `cargo test --profile release-test --tests` | 537개 test binary 모두 `test result: ok`, FAILED 0, exit 0 |
| `cargo test --profile release-test --features native-skia skia --lib` | 58개 통과 |
| `cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder` | 2개 통과 |
| `cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export` | 4개 통과 |
| `wasm-pack build --target web` | 통과. `rhwp_bg.wasm` 8,039,251바이트 생성 |
| `git diff --check` | 통과 |

## 다음 단계

- 없음 — 전 게이트 통과, commit·PR 완료 단계.
