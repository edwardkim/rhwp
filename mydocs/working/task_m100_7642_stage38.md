# 이슈 #7642 — 분할 가능한 미주의 전체 선행 이월 보정

## 독립 근거와 원인

Stage37 생산 소스 `94ebd9c0a`의 HWPX/2024 Print 전쪽 대조는 19쪽 62.18357%,
20쪽 61.77429%로 실패했다. 원본은 `samples/3-11월_실전_통합_2022.hwpx`,
기준은 `pdf/issue7642-failed-regression-print/3-11월_실전_통합_2022-hwpx-print-2024.pdf`다.
실제 flow 로그에서 본문 문단 379의 미주가 문단별 fit 전에 `prepare.rs`의 전체
미주 선행 이월 분기로 넘어갔다. 한컴 Print는 제목과 첫 본문을 왼쪽 단 하단에
남긴다. 원본 제목·첫 본문의 `keepWithNext`, `keepLines` 및 강제 나눔은 모두 0이다.

앞 미주의 저장 좌표 되감기는 다음 미주 전체가 분할 불가라는 근거가 아니다.
전체 선행 이월은 다음 문단 묶음이 실제로 분할 불가일 때만 허용한다. 문단 사이
`keep_with_next` 연결이 있고 각 문단이 `keep_lines` 또는 실제 한 줄이며,
내부 강제 쪽·단 나눔이 없는 경우로 판단한다. 문서 ID, 문항 번호, 고정 px로
분기를 좁히지 않는다. 여러 줄이고 묶음 속성이 없는 문단은 기존 문단별 fit·분할을
소비하도록 한다.

## 소비 경로와 검증

- 문단 속성·실제 format 줄 수 → `prepare_endnote_emit`의 전체 이월 수용 조건 →
  `advance_column_or_new_page` 호출 여부 → 기존 미주 문단 fit·emit의 실제 단 소속.
- 변경 전 frozen Native와 fresh WASM은 Stage37에 별도로 보존한다. 실행 중인
  교차 검증은 source를 고정한 산출물을 사용하며 새 코드의 검증으로 재분류하지 않는다.
- 공용 target의 Native 빌드·Clippy를 순차 실행하고 새 바이너리로 19·20쪽과
  기존 개선 쪽 13·14·15·17, HWP/2020 대조, 간격 없는 HWPX를 확인한다.
- 전체 이월이 필요한 명시적 묶음과 내부 강제 나눔은 별도로 조사한다. 경계나
  출력 증거가 부족하면 미검증으로 남긴다. 전체 21쪽 및 새 fresh WASM으로 재검증한다.
- 실패 회귀는 현재 브랜치에서 보정한다. 원본의 관련 모든 페이지 90% 이상과
  확대 판독을 충족한 뒤 소유·순서·보존·포함·겹침 관계 검사로 변경한다. 이번
  구현 단계에서 px 기대값·baseline·허용치를 갱신하거나 검사를 제거하지 않는다.

명령·입력 해시·전후 관측값·TSV·대표 PNG는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`에 기록한다.

## 구현과 실행 결과

`prepare.rs`의 전체 선행 이월 수용 조건에 `endnote_is_indivisible`을 적용했다.
다른 문항 번호·고정 px 조건을 추가하지 않았고, 기존 문단별 fit 및 emit 경로를
사용한다. Native 빌드는 exit 0, 618.74초이며 바이너리 SHA-256은
`33b105e7cdb5eb501454f5458d181f718ff8e0739e82434f5987a3ea7176debd`다.
Native Clippy는 exit 0, 98.84초, fmt check는 exit 0, 29.53초다.

실제 `dump-pages --compat 2024 --json` 결과에서 본문 문단 379의 미주 내부 문단
0·1이 왼쪽 단에 있고 문단 2부터 오른쪽 단에 있다. 변경 전에는 모두 오른쪽 단에
있었다. 전체 쪽수는 전후 21쪽이다. 이월 호출도 전체 선행 이월이 아니라 미주
문단 2의 문단별 fit으로 바뀌었다. 원본 속성, 전후 소유 및 호출 로그는 각각
`p19-original-note-break-contract.json`, `p19-note26-owner-before-after.json`,
`nov2022-explicit-keep-p19-fit.log`에 보존한다.

요청 목록의 HWPX 21개, 미주 598개를 확인했으나 여러 문단 전체를 연결한 명시적
keep chain은 없었다. 해당 양성 경로는 이 목록만으로 검증했다고 주장하지 않는다.
변경 소스의 focused Native 대조는 실행 중이다. 독립 PDF의 일치율·확대 판독,
전쪽 Native 및 새 fresh WASM이 미완료이므로 시각 완료·회귀 전환 완료로 판정하지
않는다. 이 단계에서 테스트·baseline·기대값은 변경하지 않았다.
