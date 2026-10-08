# PR #7618 대체 메인테이너 조판 보정

## 최종 판정

메인테이너 보정 별도 PR 제출. GitHub CI 및 후속 병합은 최신 head에서 별도로 확인한다.
원 PR을 직접 병합하지 않고 기여자의 구현을 보존한
별도 PR로 처리하라는 2026-10-08 작업지시를 따른다. 메인테이너가 Studio에서 확인한
두 실물 문서는 시각 판정 통과다. 자동 비교의 미달과 남은 줄 이월은 그대로 공개하며,
최신 제출 head의 GitHub CI는 로컬 검증과 별도로 확인한다.

원 PR은 #7618, 관련 이슈는 #7330이다. 이슈 전체 해결을 주장하거나 닫지 않는다.
원 PR의 최종 head는 `3febe51043d7e96af520b9acb99793c6e97bb394`이다.
기여자 planet6897의 `844b13884`, `0fbdebec7`, `dde38d13c`, `03bd0229f`는
원 저자를 보존해 최신 devel 위에 체리픽했다. 메인테이너 보정은 별도 커밋
`37674ddfc`, `f0e5b67b6`, `88df36982`이다. 제출 기준 devel은 `c7f357efbca7c967380c242b1b0e0c94b42043bb`이다.

## 문제와 결과

가림 처리로 재조판된 문단 뒤에서 과거 줄 구성의 전방 저장 vpos 및 자동 쪽 경계가
다시 사용되어 빈 간격과 부당 이월이 발생했다. 저장 셀에서는 같은 높이의 가로
조각을 줄 리셋으로 계산해 중첩 표 원점도 틀어졌다. 제어만 있는 문단의 객체 커서가
글자 offset으로 해석되면 공통 TAC 줄 구성이 기각되어 측정과 실제 배치가 갈렸다.

- 원 PR의 낡은 전방 vpos 허용 예외 제거를 수용한다. 유효한 저장 원문은 유지한다.
- 저장 가로 조각과 실제 리셋을 구별하고, 유효한 셀 프레임을 측정·배치에 공유한다.
- 흐름 그림의 음수 offset과 배경 그림의 비점유 계약을 구별한다.
- 재조판 뒤 저장 빈 줄만 이어지는 구간에서는 옛 자동 경계를 복원하지 않는다.
  저장 본문/개체 owner를 다시 소비한 뒤의 저장 경계는 보존한다. 명시적 쪽/단
  나누기는 유지한다. 단독 기호의 선두 공백을 양쪽 정렬의 낱말 간 슬롯으로 쓰지 않는다.
- 빈 carrier의 개체 커서는 glyph 0에 투영하고 개체 순서를 유지한다. 너비·여백·
  기준선으로 실제 줄을 정하며, 이월하면 새 단의 원점·예산으로 공통 plan을 재계산한다.
- 기본 4px 안전 여유와 기존 baseline/golden/허용치는 변경하지 않는다.

## 조판 원칙과 실제 소비 경로

| 주장 | 실제 경로·반례 | 검증 범위 |
| --- | --- | --- |
| 낡은 저장 간격을 재조판 높이로 대체 | height_cursor의 전방 이동 판단; 비가림 한컴 저장본 대조 | 원 PR 공개 회귀 2개 |
| 셀의 같은 높이 가로 조각은 줄 리셋이 아님 | stored_seg_is_row_fragment → table_layout 저장 extent/문단 배치 | cell-fragments의 그림 뒤 중첩 제목 및 셀 포함 |
| 흐름 원점과 개체 점유를 공유 | cell_wrap_vertical_offset_hu → HeightMeasurer/셀 정렬 → 그림·중첩 표 원점 | 음수/0/양수 및 BehindText 대조 |
| 미해결 재조판→저장 빈 줄 구간의 옛 자동 경계는 무효 | stored_page_boundary_invalidated_by_reflow → section 진입/whole-fit/split-entry → PageItem → 실제 배치 | 원문/재조판/명시적 나누기/여러 줄 기호 대조 |
| TAC 수용과 배치는 동일한 줄 plan 소비 | inline_flow::plan/finish_row → paragraph_flow 수용/새 단 재계산 → commit_inline_flow → PageContent.inline_flow_plans → layout_inline_flow_plan/advance_end | 기존 #7482 줄 배치 회귀 및 실물 Studio 확인; 마지막 보정의 별도 수정 전 FAIL 경계 시험은 미실행 |

첫 TAC 줄이 현 단에 들어가면 기존 분할 owner를 유지한다. 첫 줄도 들어가지 않고
전체 후보가 새 단에 들어갈 때만 이월·재계산한다. 기존 단독 분할 표·캡션·셀 각주
및 미지원 control의 owner를 일반 다중 TAC 경로로 바꾸지 않는다. 최신 보정의
작은 경계 회귀가 모든 분할 조합을 입증한다고 주장하지 않는다.

## 입력과 시각 증거

공개 재현 입력/PDF는 `samples/masked_stale_vpos/`, `pdf/masked_stale_vpos/`와
`tests/fixtures/issue_7618_stored_frames/`에 커밋했다. 한컴 2020에서 직접 저장한
셀 입력과 같은 입력의 독립 Print PDF를 사용한다. 저장 vpos를 손으로 작성하지 않았다.
생성·해시·Print 계약은 fixture README 및 `hancom-evidence.json`에 있다.
비공개 실물 문서·PDF·캡처·식별 목록은 공개 커밋에 추가하지 않는다.

검증 production source는 `93322623d8092f01fe3353cb261f31dffe683bae`이며,
제출본의 변경 source/test/fixture 34개 및 전체 src/crates/tests/cases/scripts·Cargo 입력
2,649개 경로의 바이트 동일성을 확인했다. 공용 빌드 경로와 기존 사용자 작업을 유지한다.
root wrapper가 생성한 fresh WASM과 Studio public 및 브라우저 실제 응답의 SHA-256은
`0b68fda69a28b83d1d719acedd4a139e2f8ea9bdd95c5c2735f155adb5b0459c`로 일치한다.
Native 바이너리 SHA-256은 `8045fa097eeb08db0a7000d5344d7acc601a2a8db5de2168620c824956658bd5`이다.
96dpi 인쇄 프로필, `--embed-fonts=full`로 같은 입력/PDF의 전쪽을 비교했다.
실물 한컴돋움은 실제 한컴 설치 HDOTUM.TTF를 공급했으며 좌표 보정으로 대체하지 않았다.

| 범위 | Native / fresh WASM | 판정·남은 차이 |
| --- | --- | --- |
| 공개 대조 9입력·11쪽 | 최저 92.59797% / 92.59797%, 누락 없음 | 전쪽 90% gate 통과; 대표 셀/기호 이미지 직접 판독 |
| 실물 대조 A·3쪽 | 98.81537 / 92.13646 / 98.55098%, 누락 없음; 최신 Native와 fresh WASM 동일 | 메인테이너 Studio 시각 통과; 최신 제출 production에서도 전쪽 90% 통과 |
| 실물 대조 B·7쪽 | 95.89855 / 99.80717 / 99.15441 / 99.98640 / 95.92454 / 93.11727 / 74.24296%, 두 경로 동일 | 메인테이너 Studio 시각 통과. 7쪽 자동 gate는 re_review_required이며 마지막 별표 줄 이월 차이를 보존 |

추가 회귀 대조: #5755의 3쪽은 전쪽 Native/fresh WASM 최저 99.06252%로 통과했다.
`hwp3-sample16-hwp5.hwpx`는 원래 전체 64쪽 중 영향 경계 55–58쪽만 비교했다.
Native/fresh WASM 점수는 83.83076 / 84.59504 / 49.32169 / 72.64726%이며 자동 gate 미달이다.
수정 전 `79b667eeb`를 실제 재빌드한 출력과 최종 `93322623d`의 네 쪽 Native
raster/review/overlay가 바이트 단위로 동일하다. 따라서 기존 시각 차이이며,
이 문서 전체 64쪽의 시각 일치나 기존 표 원점 차이의 해결을 주장하지 않는다.
이 대조 문서에는 새 fixture/golden/회귀 검사를 추가하지 않았다. 기존 쪽 소유·
겹침·본문 넘침 검사를 통과하도록 원인을 수정했으며 baseline 허용치는 유지했다.

원 PR이 보고한 다른 실물 문서 전부의 시각 일치 완료를 주장하지 않는다.
새 회귀 6개는 해당 공개 9입력 전쪽의 Native/fresh WASM 90% 통과를 먼저 확인한 뒤 추가했다.
실물 B와 추가 64쪽 문서의 낮은 점수를 해당 새 검사의 기대값·golden으로 고정하지 않았다.
마지막 TAC 보정은 실물 B에서 직접 수용된 부분 개선이며, 자동 gate 결과를 passed로 바꾸거나
글꼴 예외로 분류하지 않았다. 이번 별도 PR 제출은 메인테이너의 직접 판정 및 처리 지시에 따른다.

전체 Native 15입력·30개 비교 페이지 TSV 다운로드: [비식별 원본 TSV ZIP](https://github.com/user-attachments/files/33187513/pr7618-maintainer-native-tsv-final.zip).
본문 게시 후 다운로드 동일성을 확인한다. 공개 대조 9입력 외 원 PR의 공개 가림/정상 저장본
2입력도 최신 fresh WASM으로 재출력해 gate를 확인했다.

대표 이미지 안정 경로는 `mydocs/pr/assets/issue_7330_maintainer_20261008/`이다.
PR 본문에 최종 제출 head SHA로 고정한 Native/fresh WASM review·overlay를 실제 이미지로 표시한다.

## 제출 전 실행 기록

검증 worktree와 제출 worktree는 공용 `/home/edward/mygithub/rhwp/target/pr-review`를
재사용한다. prepare로 생성한 suite/manifest는 source PR에 stage하지 않는다.
로그·명령·전체 TSV는 기본 작업공간의 ignored
`output/pr-review/planet6897-20261008/visual-gate-preparation/pr7618-current/submission/`에 보존한다.

- fmt, native/WASM32/workspace all-target Clippy, workspace build,
  manifest 및 source-side unit-tier의 고정 devel base 비교: 통과.
- 최초 전체 release-test nextest: 10,533개 중 10,530 PASS / 3 FAIL. 쪽 전체를
  `any(no-stored-line)`로 무효화한 가정을 제거하고, 실제 PageItem의 소비 순서로
  재조판→저장 빈 줄 구간과 저장 본문/개체 owner의 재개를 구별했다.
  기존 겹침/본문 넘침 원장과 #5755의 독립 쪽 소유 검사, 공개 대조 6개가 보정 후
  focused 9 PASS 뒤 마지막 owner 구분 source에서 전체 10,533 PASS / 0 FAIL / 50 SKIP를 확인했다.
- 최종 production `93322623d` / 제출 production `88df36982`: 필수 lint·전체 nextest 통과.
  Native Skia lib 및 그림 placeholder·직접 PDF export 2종 통과. fresh WASM은 최종 코드로
  다시 빌드했다. Studio public/브라우저 응답 SHA-256 일치 및 공개 문서의 1쪽 실제
  canvas 그리기를 확인했다. 최종 Native/fresh WASM 15입력·30개 비교 페이지는
  누락 없이 완료했고 양 경로 raster가 모두 바이트 동일하다. 미달은 위 표에 그대로 남긴다.
- 수정 전 `79b667eeb`를 실제 재컴파일한 공개 검사: 4 FAIL / 정상 대조 2 PASS.
  실패는 단독 기호의 낱말 간격, 여러 줄 재조판 뒤 부당 쪽 이월, 저장 가로 조각 뒤
  중첩 표 순서, 음수 흐름 offset의 공통 원점이다. 최초 공유 캐시 재사용 결과는
  수정 전 결함 증거로 사용하지 않았다. 최종 수정 후 전체 검사와 연결한다.
- baseline/golden 및 테스트 통과를 위한 기준값 변경: 없음.

## 재현 명령

검증 checkout 루트에서 공용 target을 고정하고 순차 실행했다. 정책 비교 base는
`c7f357efbca7c967380c242b1b0e0c94b42043bb`이다. 실물 입력·로컬 식별 경로는 공개하지 않는다.

```bash
export CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check --base-ref c7f357efbca7c967380c242b1b0e0c94b42043bb
node scripts/rust-unit-test-tiers.mjs --check --base-ref c7f357efbca7c967380c242b1b0e0c94b42043bb
cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast
cargo test --locked --profile release-test --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia
scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

공개 대표 입력의 Native/fresh WASM 비교는 다음 동일 명령에 WASM 경로만 추가한다.
Native는 위 검증 source로 빌드한 바이너리를 사용한다. 각 입력의 전쪽 비교, 실물 A/B의
전쪽 비교와 두 추가 대조 문서의 범위는 위 표·첨부 TSV와 연결한다.

```bash
python scripts/visual_sweep.py --rhwp-bin <검증한-Native-binary> \
  --file-target cell-fragments tests/fixtures/issue_7618_stored_frames/cell-fragments.hwpx \
  tests/fixtures/issue_7618_stored_frames/cell-fragments-2020.pdf \
  --dpi 96 --embed-fonts=full --font-path <실제-Windows-font-directory> \
  --out output/pr-review/pr7618-maintainer/native
# fresh WASM: 위 명령에 --wasm-pkg pkg 추가, --out 경로를 wasm으로 변경
python scripts/visual_sweep.py --silhouette-only \
  --png-pair <최종-rhwp_png-directory> <동일-PDF-png-directory> \
  --key cell-fragments --out output/pr-review/pr7618-maintainer/native-tsv/cell-fragments
```

원 기여자가 이미 공개한 역사적 TSV/PNG는 원 커밋에 보존한다. 이 파일의 최종
검증 source 및 첨부 Native ZIP과 구분하며, 과거 source의 측정값을 최신 결과로 사용하지 않는다.

## 원 PR 종료와 후속 처리

새 PR 번호를 받은 뒤 #7618에 대체 관계·기여자 credit·보정 범위·남은 차이를
한국어 comment로 남기고 close한다. 기여자 fork/branch는 삭제하지 않는다.
새 PR의 최신 CI를 확인하고 병합 승인 범위에 맞춰 진행한다. #7330은 Refs로 유지한다.
병합 후 contributor comment에는 실제 merge SHA 고정 공개 대표 이미지 및
시각 검증 정본 링크를 사용한다. 비공개 실물 증거와 자동 점수를 추정해 추가하지 않는다.
