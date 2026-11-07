# #8671 / #8337 실제 원 사례의 숫자 대체 결함과 보정 계획

## 실제 입력과 판정

#8337의 공개 재현 파일은 기존 `samples/issue7593/repro-equation-issue7593.hwpx`와 바이트가 같다. Git blob `d0c323cd037a593f88292feb28651c0f50eaa78b`, SHA-256 `88a3683c34e92f8887e54b9b379f8c399a8ec844e46e127793c6486cba8d31ec`로 확인했다. 기존 독립 Print는 `pdf/issue7593/repro-equation-issue7593-hwpx-2020.pdf`다. 원 #7594의 기호 대체 해결 이력과 후속 #8337의 원 글꼴 부재 범위를 구분했다.

현재 통합 source `c2648ffdfe`의 실제 `native-skia` CLI에서 동일 입력·Print·96 DPI·compat 2022로 대조했다. 최소 5개 공개 글꼴 환경에서 본문용 HCR/Haansoft 프로그램을 추가 공급하되 HyhwpEQ만 뺀 대조군과 HyhwpEQ를 공급한 양성 대조군이다. 원 프로그램·파생 글꼴은 공개하지 않는다.

| 조건 | 전 1쪽 raw | 직접 판독 |
| --- | --- | --- |
| 최소 공개 글꼴만 | 89.97854% | 본문 글꼴 차이도 있어 이 점수로 숫자 원인을 단정하지 않음 |
| 본문 글꼴 공급, HyhwpEQ 없음 | 99.66% | AB는 복원됐지만 `[4점]`의 4와 `③ 3`의 3이 다른 글자 모양으로 나옴 |
| 동일 본문 글꼴 + HyhwpEQ 공급 | 99.63% | 같은 위치에 올바른 4·3이 나옴 |

[원 픽셀 확대 비교](issue8337/digits-font-control.png). 점수가 더 높아도 의미가 틀린 문자를 통과 처리하지 않는다. SVG 공통 producer에서도 4와 3의 표시 코드가 각각 `HyhwpEQ`의 U+E037·U+E036임을 확인했다. `hancom_glyphs.rs`의 Number 매핑과 일치한다. Native fallback은 표시용 사설 코드를 일반 대체 글꼴에서 그대로 조회하고 있어 원래 숫자의 의미를 보존하지 못한다. 기존 P79 합성 검사는 직접 ASCII/CJK를 담은 `GlyphRun`을 만들어 원 producer의 숫자 코드 경계를 덮지 않는다.

초기 판정은 **메인테이너 보정 승인 대기 / 머지 보류**였다. 정상 글꼴 공급의 올바른 원 glyph와 위치·전진폭·기준선·shear는 보존해야 한다. #8337을 완료했다고 닫지 않는다.

## 제안하는 제한 보정

1. 수식 공통 producer의 원문 문자 ↔ 표시용 glyph 코드 관계를 확인하고, 원 글꼴을 쓸 수 없는 경우의 의미 문자·스타일을 공통 결과 또는 동일한 매핑에서 얻는다. Native에서 문서 번호·특정 숫자의 좌표를 맞추지 않는다.
2. 원 face가 해당 표시 glyph를 제공하면 기존 경로를 유지한다. 누락된 경우만 의미 문자로 대체 글꼴을 선택하고 원 positions·baseline·shears를 사용한다. 사설 코드가 우연히 있는 다른 글꼴을 정답으로 취급하지 않는다.
3. 원 #8337 문서의 4·3·AB, 숫자 0–9·기호·이탤릭 변수와 원 face 공급 대조를 실제 producer 경로로 검증한다. 알 수 없는 family의 임의 코드 변환을 열지 않는다. 정상 문자·스타일·glyph 반복 cache와 같은 위치 보존을 확인한다.
4. 같은 문서 Native/fresh WASM 전쪽·독립 Print·직접 확대 판독을 완료하고, 보정 source에 영향받는 집중·Native/공통 회귀·lint를 재검증한다. 새 회귀의 시각 선행 조건을 지킨다. 기존 본문 폰트 예산·겹침 차이는 원인과 base를 먼저 대조하며 임의 기대값 변경이나 기본 한도 증가는 하지 않는다.

[AGENTS.md:304](../../../../AGENTS.md#증거와-기준값)의 직접 판독과 [334행](../../../../AGENTS.md#페이지별-tsv와-pr-재검토-필수-조건)의 reviewer 보류·메인테이너 대행 제한에 따라 이 기능 보정의 담당 범위 결정을 요청한다. 원격 댓글·push·merge는 별도다. 이미 승인된 Metal 실험 기반 예외를 이 숫자 결함에 적용하지 않는다.

## 작업지시자 승인

2026-11-07 작업지시자가 “메인테이너 보정·검증 승인”을 명시했다. 위 제한 보정·정상 대조·실제 재검증을 진행한다. 원격 댓글·push·PR·merge 승인은 포함하지 않는다.

## 승인한 보정의 Native 실행 결과

공통 producer의 기존 `glyph` 매핑을 재사용하는 의미 코드 복원을 Native fallback에 연결했다. source `cda7c8cb33`의 실제 `native-skia` CLI에서 같은 최소 글꼴·본문 subset 조건으로 원 사례를 다시 출력했다. 원 수식 face가 없을 때 4·3이 올바르게 표시되고 AB도 보존된다. [수정 전·후·독립 Print 확대](issue8337/digits-before-after-print.png), [실행 요약](issue8337/native-correction.json)을 따른다. 원 face를 공급하면 수정 전후 PNG가 바이트 단위로 같다. 원 위치·기준선·shear를 재측정하지 않는다.

검증용 파생 프로그램은 모든 대상 문서의 실제 glyph inventory로 제한해 기존 64 MiB 예산 안에 넣었다. 6개 프로그램 합계는 3,308,448바이트이며 숫자 음성 대조에서는 HyhwpEQ만 제외했다. 원본과 파생 글꼴의 hash·보존 메트릭·tool 제한은 로컬 원장에 기록했고 프로그램 자체는 커밋하지 않는다.

이 결과는 Native 경계의 확인이다. fresh WASM·공식 회귀와 공통 필수 검증 완료 전 최종 수용 판정은 보류한다.

## 생산·측정·배치의 소비 경로

`EqLayout::for_equation` → `hancom_glyphs::measure`가 원문 Number/Variable 역할과 해당 face의 표시 코드·positions·shears·전진폭·baseline_shift를 결정한다. `layout_for_paint`와 `PaintOp::Equation`의 같은 LayoutBox를 `skia::equation_conv::render_box`가 소비하며 `parent + lb.x/y`, `lb.baseline + run.baseline_shift`를 최종 paint 원점으로 전달한다. 이 기존 생산·측정·프레임 경로는 수정하지 않았다. `draw_hancom_glyphs`에서만 원 face coverage가 없을 때 기존 producer 매핑의 역관계로 의미 문자·내재 이탤릭을 얻는다. 최종 `canvas.draw_str`는 원 `x + offset`·baseline·shear를 그대로 사용한다. fallback font의 자연 전진을 다시 누적하거나 clamp하지 않는다. helper는 Native cfg이며 Canvas/SVG의 공유 producer 결과는 바꾸지 않는다.

같은 원본을 쓰는 기존 `issue_7593_print_placement`, `issue_7642_dyad_saved_reservation`, `issue_7642_dyad_outline_paint`, `issue_7445_negative_indent_ownership`과 page-count baseline을 점검했다. 해당 기존 검사는 문단·셀 소유, 문자 순서·기준선·탭 격자·예약 프레임 및 윤곽 관계를 보호하며 잘못된 실물 절대 px 핀을 교체할 항목이 없었다. 이를 유지하고 기존 Native equation font 모듈에 실제 4·3 producer 경계를 추가했다. 합성 font/surface 비교는 원 사례의 한컴 Print 전쪽·확대 판독과 구분한다.

## 실제 회귀의 수정 전·후 실행

검사 source `263b1e753c`의 실제 producer 검사를 그대로 두고 제품 두 파일만 보정 전 `c2648ffdfe`로 바꿔 실행했다. 기존 5개 검사는 통과하고 신규 실제 숫자 검사는 missing-ink assertion에서 실패했다. 최종 보정으로 두 파일을 복구한 뒤 같은 신규 검사를 포함한 Native 집중 회귀 194건이 모두 통과했다. [음성·양성 실행 기록](issue8337/producer-regression-results.json)을 따른다.

fresh WASM 전 1쪽 raw 99.48%와 원 숫자·AB의 확대 판독도 완료했다. 원 수식 face 공급 Native 전 1쪽은 99.60%, face 부재 대조는 99.63%다. 원 face 공급 조건의 base/후보 PNG는 같다. 이후 [최종 공통 필수 검증 17개 묶음](local-gates.json)도 모두 통과했다. 원격 수용과 기존 대조군 차이의 예외는 작업지시자 승인 대기다.

## 최신 devel의 원 이슈 재현 대조

같은 본문 글꼴 subset과 원 수식 face 부재 조건으로 최신 devel `b8dd67c500`의 보관된 Native 바이너리도 실행했다. 실제 1쪽에서 AB·CD·4·3이 빠지고 raw 93.84%였다. 최종 보정 `cda7c8cb33`에서는 모두 표시되며 raw 99.63%다. 같은 독립 PDF raster의 hash도 확인했다. [실행·수치](issue8337/latest-base-missing-face-comparison.json), [devel review](issue8337/missing-face-base-review.png)·[overlay](issue8337/missing-face-base-overlay.png), [보정 review](issue8337/missing-face-candidate-review.png)·[overlay](issue8337/missing-face-candidate-overlay.png)를 따른다.

이 비교는 원 이슈의 글꼴 부재 경계다. 원 face를 공급한 48개 base/후보 PNG의 동일성 결과와 조건을 섞지 않는다. 원 기여 P79에서 남은 잘못된 숫자와 최신 devel에서 아직 빠지는 문자도 구분한다.

## 원격 승인·실행 확정

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

위의 승인 대기·미수행 표현은 당시 로컬 단계의 기록이며 현재 상태는 [최종 운영 기록](remote/completion.md)을 따른다.
