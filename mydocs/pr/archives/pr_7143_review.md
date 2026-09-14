---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7143 — 분할 rowspan 셀 높이 검토

**최종 판정: 머지 보류.** 목표 문구 복원과 focused 4건 통과는 확인했다. 그러나 같은 원본에서 표 하나를 메모리로 분리한 대조 입력에서 PR 적용본만 본문 하단을 약 4.21px 넘었다. renderer의 높이 증가를 pagination이 예약하지 않는 경로가 남아 있다. GitHub approve·comment·push·merge는 수행하지 않았다.

## 검토 기준

| 항목 | 확인 결과 |
| --- | --- |
| 원 PR | [#7143](https://github.com/edwardkim/rhwp/pull/7143), planet6897, 기존 기여자 |
| 제목 | 수정(layout/table): 이어받는 조각의 걸친 rowspan 셀이 제 높이를 받는다 (#6981) |
| source head | `34e1186f40f9f0b2d4d72596f3d0996b363b4f88` |
| base / 통합 검토 | 최신 `upstream/devel` `38af2aae3571dc9d7f43671c1ca1a2bc8f24c815` 위에 source를 `-x`로 체리픽 |
| 로컬 검토 branch / head | `codex/pr7143-review-20260914` / `4a9546f8c` |
| merge simulation tree | `b4f461a06d26f33badbdbed0aec0dcd6f6cd1bba`; 충돌 없음, 체리픽 tree와 동일 |
| 규모 | 1 commit, 7 files, +437/-1; renderer 3개, 회귀 test 1개, oracle baseline, 보고서·PNG |
| 관련 이슈 | [#6981](https://github.com/edwardkim/rhwp/issues/6981), 본문 `Fixes #6981` |
| 접수·원격 상태 | reviewer jangster77 지정. 최종 조회에서 source head 불변, OPEN / MERGEABLE / CLEAN. 작성 시점 참고값 |

외부 PR 검토 경로에 intake/local validation/visual evidence 절차를 적용했다. 메인터너 production code 보정은 하지 않았다. 전체 renderer 영향과 기준값 변경이 있어 직접 시각 검토 대상이다.

## 발견 1 — [P1] pagination이 예약하지 않은 높이를 renderer가 추가한다

**실행으로 재현된 회귀.** 원 head의 `table_partial.rs:4018–4041`은 모든 per-row continuation에 보정을 적용한다. 반면 `typeset.rs:22964`의 보정은 `rowspan_touched[r] && !rowbreak_rowspan_row_splittable` 분기에서만 적용한다. splittable 행은 일반 `row_total` 경로로 가므로 renderer의 증가분을 예약하지 않는다. 함수 이름과 일부 계산 출처를 공유하지만 최종 행 높이는 공통 결과가 아니다.

원본 `samples/task2287/1342000_edu_curriculum_map.hwp`를 읽고 section 28 / paragraph 2(0-based)의 표 문단만 같은 문서 IR에 남겼다. 글자·셀·여백·용지 정의는 바꾸지 않았다. 이 입력은 메모리에서만 생성되며, 한컴이 저장한 독립 축소 문서/PDF라고 주장하지 않는다. 재현기는 [pr7143_repro_probe.rs](../assets/pr7143_repro_probe.rs), 입력·실행 수치와 진단은 [증적 JSON](../assets/pr7143_review_evidence.json)에 보존했다.

| 같은 입력의 p3 | devel `38af2aae3` | PR 통합 `4a9546f8c` |
| --- | --- | --- |
| 문서 쪽수 | 4 | 4 |
| 0.5px 허용 범위를 넘는 Table/Body bbox | 없음 | Table bottom 722.34666667 / Body bottom 718.13333333 |
| 본문 경계 초과 | 허용 범위 내 | 4.21333334px |
| 엔진의 별도 layout 진단 | 해당 overflow 없음 | `LAYOUT_OVERFLOW ... overflow=3.5px` |

Table bbox와 엔진 소비 높이 진단은 서로 다른 계상값이므로 4.21px와 3.5px를 동일 지표로 섞지 않는다. PR 진단에서 `D6981R r=44 start_row=44 end_row=66 need=20.9 have=17.1`이 나타나지만 해당 r44의 `D6981S` 보정은 없다. 따라서 현재 테스트의 “셀 안에 줄이 들어간다”만으로 조각 전체의 본문 점유가 안전함을 보장하지 못한다.

![같은 메모리 입력의 devel/PR 본문 경계 비교](../assets/pr7143_repro_body_overflow.png)

이 PNG는 한컴 비교가 아닌 devel/PR의 같은 IR 대조다. 원본 p377 SVG의 `@font-face` 별칭만 보충하고 좌표·텍스트·그리기는 바꾸지 않은 뒤, Visual Sweep과 같은 Chrome webfont rasterizer로 캡처했다. 보라색은 Body bbox의 하단이다. 실제로 열어 한글과 라벨을 확인했다.

**해제 조건:** 실제 continuation의 시작/끝 컷·유닛 범위와 적용 가능 여부를 반영한 높이를 pagination과 renderer가 함께 소비하도록 보정한다. helper의 `prior_h`는 renderer의 기존 straddle 경로와 달리 `start_cut` 소비량을 받지 않는 점도 함께 해결해야 한다. 샘플 식별자나 임의 여유값으로 예외를 추가하지 않는다. 위 메모리 입력에서 본문 초과가 사라지고, 기존 focused 4건 및 관련 컷 경계가 유지되어야 한다.

## 발견 2 — [P2] 보정 높이가 예산을 넘으면 원래 높이로 행을 수용한다

**코드 경로 검토로 확인한 별도 경계 결함.** `typeset.rs:22990–23003`에서 `r > cursor_row`이고 `grown`만 예산을 넘으면 원래 `h`로 되돌린다. 이어지는 fit 판정은 이 작은 `h`로 통과할 수 있고, `end_row = r + 1`로 행을 완전히 수용한다. 이후 renderer는 셀 끝이 조각 안에 있으므로 end-cut 제외 조건에 걸리지 않고 높이를 다시 늘린다. “예산 밖이면 컷 소관”이라는 주석과 달리 그 분기에서 실제 컷이나 이월은 만들지 않는다.

예를 들어 spacing=0, consumed=60, h=20, have=80, need=100, available=90이면 grown=40은 거부되지만 원래 h=20은 통과한다. 최종 renderer는 부족한 20을 다시 더해 소비 80과 배치 100이 갈라진다. 이 숫자는 분기 산술을 설명하는 예이며 별도 실물 문서 실행 결과가 아니다.

**해제 조건:** 부족분이 들어가지 않으면 같은 높이로 행을 이월하거나, 실제 컷과 그 소유 범위를 만들어 renderer에 전달한다. `원래 높이는 fit / 보정 높이는 overflow` 경계와, 같은 조각 안에서 여러 rowspan이 끝나는 경우의 누적 높이를 검증한다. 첫 재현 사례의 회귀와 이 정적 경계 결함을 서로 같은 실행 증거라고 쓰지 않는다.

## 실행한 검증과 범위

- source head의 [CI 34839089236](https://github.com/edwardkim/rhwp/actions/runs/34839089236): SUCCESS. 원 로그의 Archive A/B/C/D 합계 9,668 pass, 51 skip이며 신규 4개 테스트의 PASS도 확인했다. Lint·Native Skia·Frontend package, 별도 CodeQL·Render Diff·Adapter·Proptest도 최신 source head에서 완료됐다.
- 최신 devel과 merge simulation, `git diff --check`를 통과했다. local source/test 보정 없이 동일 변경을 적용했다.
- 검토 전용 `target/pr7143-review-20260914`에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후 `node scripts/run-rust-test.mjs issue_6981_split_straddle_row_height -- --cargo-profile release-test --target-dir target/pr7143-review-20260914`를 실행했다. 종료 코드 0, 4 pass. 함께 묶인 다른 201건은 필터로 실행하지 않았다.
- 전체 Rust·Native Skia 회귀는 이미 통과한 exact head CI를 근거로 중복 실행하지 않았다. 비교용 CLI는 base/PR 각각 `cargo build --locked --profile release-test --target-dir target/pr7143-review-20260914 --bin rhwp`로 생성했다.
- 전체 415쪽 PDF 범위에 `fidelity_compare.py --text-only --export-all-svg --layout-ledger`를 실행하고 SVG 414쪽과 text/table/clip 원장을 조사했다. 이는 전체 페이지의 사람 시각 승인과 다르다.
- Visual Sweep을 PR에서 p81–83,151–153,271,273,285,287,376–379의 14쪽, base에서 p81–83,151–153,376–378의 9쪽 실행했다. PR 구조 후보는 0/14였으나 페이지 소유·표 높이 일치나 본문 초과 반례의 안전을 뜻하지 않는다.
- 실제 연 화면은 p82·152 및 아래 3개 내용 대응 비교다. 새 WASM package 실행은 하지 않았고 Native SVG의 실제 Chrome webfont 캡처를 수행했다. WASM 실행 완료로 표현하지 않는다.

## 한컴 PDF와 내용 대응 Visual Sweep

원본 `lastSavedWith.version=9.1.1.3933`, product는 null이다. 확장자로 저장 제품을 추정하지 않았다. 기존 커밋의 Hancom `Hwp 2022 12.0.0.4547` PDF 415쪽을 재사용했으며, 재변환하거나 이름을 바꿔 중복 추가하지 않았다. 이슈의 별도 engine 2024 / 416쪽 기록과 현재 사용한 2022 / 415쪽 기준은 구분한다.

| 내용 대응 페이지 (rhwp / PDF) | 전체 픽셀 일치율 | 내용 픽셀 보조 일치율 | 직접 판정 |
| --- | ---: | ---: | --- |
| 271 / 273 | 85.86897% | 34.13262% | 선언문 작성하기 표시. 표의 페이지 시작·행 위치 차이 잔존 |
| 285 / 287 | 84.42941% | 30.02696% | 공동 선언문 작성하기 표시. 표 행 소유·높이 차이 잔존 |
| 377 / 378 | 85.68662% | 33.61819% | 목표 `선언문 작성` 복원. 한컴과 전체 표 배치는 아직 다름 |

[대표 비교 PNG](../assets/pr7143_hancom_rhwp377_pdf378.png)는 실제 다른 PDF 쪽 번호를 라벨에 표시했다. 원 PDF를 잘라 새 기준 파일을 만들지 않고, Visual Sweep의 overlay·review 함수에 각 원본 raster를 전달했다. 수치는 사람의 정답률이 아니며 글꼴·페이지 시작 위치 영향을 포함한다.

baseline 413 → PR 414쪽으로 415쪽 기준과의 차이가 2 → 1로 줄었다. 다만 p82·152 raster는 devel과 byte-equivalent 픽셀이었고, 이슈 본문이 언급한 그 셀 사례까지 해결됐다고 단정하지 않는다. `Fixes #6981`을 유지하려면 잔여 두 사례도 개별 확인하거나 후속 범위를 명시해야 한다. 목표 한 줄 복원이 전체 이슈 해소·전체 문서 fidelity를 증명하지 않는다.

## 공통 조판 원칙 및 검증 입력 커밋 확인

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 미충족 | 저장 줄·행·패딩과 PDF의 목표 문구가 근거이며 문서 ID 분기는 없음. 다만 적용 분기·컷 계약이 일치하지 않음 |
| 측정·배치 일관성 | 미충족 | r44 renderer 단독 증가와 본문 초과 재현; 예산 실패 시 작은 h 수용 경로 |
| 줄 소속과 점유 높이 | 미충족 | `start_cut` 소비를 새 helper에 전달하지 않고 전체 잔여를 다시 추정; 일반 행 예약과 별도로 renderer가 증가 |
| 사례와 독립 증거 | 충족 | 커밋된 실물 HWP 3개 focused, 한컴 PDF 직접 비교, 별도 메모리 축소 계약 대조를 구분 |
| 기준값 변경 | 충족 | oracle의 한컴 415 유지, rhwp 413→414 실제 관찰. 차이 감소가 다른 회귀를 면제하지 않음 |
| 주장과 검증 범위 | 충족 | exact source/통합 SHA, CI/로컬/시각과 미실행 WASM, 실행 회귀/코드 경로 결함 구분 |
| 검증 입력 커밋 | 충족 | [증적 JSON의 inputs](../assets/pr7143_review_evidence.json)에 HWP 3개·PDF 1개의 기존 경로·SHA-256·source commit 내용 일치 기록 |

## 보정 순서와 merge 전 조건

[메인터너 보정 계획](pr_7143_review_impl.md)에 실제 범위와 검증 순서를 정리했다. 현재는 보정 commit을 만들지 않았다. 새 코드 후보는 focused·위 반례·필요한 lint/CI 및 직접 시각 확인을 다시 완료한 뒤 별도 판정한다. 이 문서는 merge 승인 요청이 아니다.

## Merge 후 contributor PR comment 계획

현재 머지 보류이므로 게시하지 않는다. 보정 뒤 검토 기록을 갱신하고 최신 head가 merge된 경우에만 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), 실제 페이지·후보 수·최종 보조 지표·사람 판정 및 새 증적을 포함한다. 없는 WASM/PDF 결과를 추가하지 않는다.

기존 대표 경로는 `mydocs/pr/assets/pr7143_hancom_rhwp377_pdf378.png`와 `pr7143_repro_body_overflow.png`다. 최종 증적이 devel에 반영된 후 `<merge-commit-sha>` 고정 raw image URL로 표시하고 `--body-file` 게시 뒤 API로 본문을 확인한다. 현재의 실패 PNG를 보정 후 통과 증거로 재사용하지 않는다. 이슈 close 범위와 contributor credit도 실제 해결 결과 기준으로 기록한다.
