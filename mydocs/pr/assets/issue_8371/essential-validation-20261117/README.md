---
kind: snapshot
status: active
canonical: mydocs/pr/assets/issue_8371/essential-validation-20261117/README.md
last_verified: 2026-11-18
---

# HWP3 그림 소속과 Native 글꼴 선택의 최종 검증

코드 producer는 `fcf2e11b1727297743343446333132e1409737f1`, rebase 기준은 `22da83586aab1baf960f2159abda347062ecf7ed`다. 세 보정 커밋을 유지하고 제출 증적만 별도 커밋한다.
[초기 제출 95a7fb1e8b의 제품·테스트·빌드 입력 동일성](source-identity.json), [빌드 신원](build-provenance.json),
[직접 판독](direct-inspection.json)을 함께 확인한다. 이슈 #8371·#8218은 기존 조사와 입력의 맥락이며 새 종료 대상으로 지정하지 않는다.

## 2026-11-18 확정 병합 결과

[PR #9157](https://github.com/edwardkim/rhwp/pull/9157)을 작업지시자의 명시적인 선 병합·후속 처리 지시에 따라 `af54158960ec91504d42faf842b0cb3229366236`로 병합했습니다.
보호 검증은 전 OS에 적용하며 현재 후보의 보호 18종 전쪽 검증은 여전히 미완료입니다. 기준값·하한은 변경하지 않았습니다.
[최종 head CI·duration·로컬 추가 확인·raw 이미지 hash 영수증](merge-followup-20261118/receipt.json)과 [최종 검토](../../../archives/pr_9157_review.md)를 따릅니다.
이하의 병합 보류 표현은 지시 이전의 검토 이력입니다. 이전 실행 producer와 실제 검증 범위를 보존하며 최신 head의 재실행으로 바꾸어 표시하지 않습니다.

## 병합 지시 이전의 증적과 판단

## 수정과 소비 경로

- HWP3 sample4의 NASM 그림은 25쪽 제목에 겹치던 상태에서 독립 한컴 Print의 26쪽으로 이동한다.
  실제 저장 줄·프레임 초과·명시적 쪽 경계·다음 쪽 예약 밴드를 확인해 공통 `ParagraphFloatPlacement`를 만든다.
  `typeset/section/controls.rs`의 이월 수용 결과를 `layout.rs`가 소비한다. 전진량 0인 그림은 뒤 글줄의 단 원점과 문단 상태를 덮어쓰지 않는다.
  최종 PageLayerTree를 Native PNG·SVG·Studio Canvas2D·실제 CanvasKit에서 확인했다.
- Native DWrite가 메모리 SFNT stream을 주지 않을 때 공통 table 복구로 빈 기호를 판정한다.
  이름별 예외 없이 다음 유효한 glyph 공급자를 선택한다. Windows 편람 실제 200쪽의 검은 원 기호 3개를 Print와 대조했다.
- 같은 요청 family·style에서 원본 glyf TTF → glyf TTC → CFF OTF 순서를 보존한다.
  파일 확장자 대신 실제 원본 프로그램을 확인하며, 원본을 OS가 감췄을 때는 PostScript 이름과 모든 table이 같은 설치 프로그램만 인정한다.
  스타일 일치와 원 family가 먼저다. [OFL 합성 자원](../../../../../tests/fixtures/fonts/RHWPNativeContainer.LICENSE.md)의 실제 RGBA 비교가 공급 순서·파일명 순서에 따른 TTC 승격을 막는다.

## 현재 head의 실제 검증

| 검사 | 결과 |
| --- | --- |
| 최소 글꼴 전체 nextest, release-test | 13,718 PASS / 0 FAIL / 45 skip, 1,061.189초 |
| 최소 글꼴 집중 Native / 새 base 기능 | 49 / 7 PASS |
| Native Skia lib / workspace lib | 3,853 PASS·12 ignored / 182 PASS |
| Native missing-picture / direct PDF | 2 / 36 PASS |
| doctest | 8 PASS·3 ignored |
| fmt·root/Native/WASM clippy·workspace build·all-target clippy·suite 정책 | PASS |
| suite 자동 수집 계약 | 24 PASS |
| fresh WASM | Windows locked wrapper, release + wasm-opt, 실제 root 재컴파일 29분 47초 / 전체 33분 |
| Windows DWrite 보조 실물 확인 | 현재 코드의 dev Native 빌드, 편람 p200 raw 100% |

[Rust 명령·집계·로그 해시](rust-validation.json) · [Windows lint](windows-lint.json) · [Windows p200](windows-symbol200.json).
공용 `target/pr-review`를 각 host에서 순차 재사용했다. Cargo 회귀는 Ubuntu의 bwrap 최소 글꼴 격리에서 실행했고 실제 노출 글꼴을 기록했다.
시각 비교는 고정한 원본 글꼴을 별도 공급했다. 새로 설치하거나 배포한 글꼴은 없다.

## 전체 페이지 원본 TSV

96 DPI, print, 색상 경계 보정 전 2px `silhouette_raw_match_percent`다. 평균으로 미달 쪽을 상쇄하지 않는다.
아래 Native/fresh WASM의 모든 페이지가 90% 이상이며 독립 Print와 전체 쪽수가 일치한다.

| 문서 | 전체 쪽수 | Native 원본 | fresh WASM 원본 | Native / WASM 최저 raw |
| --- | --- | --- | --- | --- |
| sample4 | 36 | [TSV](visual/native/candidate/sample4.tsv) | [TSV](visual/wasm/candidate/sample4.tsv) | 97.66586 / 97.66564% |
| 행정업무운영 편람 HWPX | 383 | [필수 handbook gate TSV](visual/native/candidate/handbook.tsv) | [TSV](visual/wasm/candidate/handbook.tsv) | 99.96504 / 99.96448% |
| 직선 띠 | 1 | [TSV](visual/native/candidate/band.tsv) | [TSV](visual/wasm/candidate/band.tsv) | 100 / 100% |
| 줄 밖 탭 | 1 | [TSV](visual/native/candidate/tab.tsv) | [TSV](visual/wasm/candidate/tab.tsv) | 100 / 100% |

[Native 전후 페이지별 수치](visual/native-summary.json) · [WASM 전후 페이지별 수치](visual/wasm-summary.json) · [base TSV 목록](visual/index.json).
sample4는 양쪽 모두 25·26쪽만 바뀌고 나머지 34쪽은 base와 PNG SHA가 같다. base 최저는 Native 96.00615 / WASM 96.00358%다.
[Native 확대 전후](images/native/source-layout-enlarged.png) · [WASM 확대 전후](images/wasm/source-layout-enlarged.png)에서 제목, 그림→설명→본문, 정상 21쪽 그림을 같은 영역으로 대조했다.
22쪽 및 26쪽 코드 배경의 기존 하단 밴드 차이는 남아 있으며 base와 같음을 확인했다. 이 PR이 문서 전체의 완전 일치를 주장하지 않는다.

공통 보호 4종 × 두 경로의 [8건 결과와 개별 TSV·review·overlay](contributor/summary.json)도 PASS다.
pic-crop의 두 자리 표시 100.00%는 반올림 전 완전 일치라는 뜻이 아니다.

## 실제 브라우저 소비

[원본 글꼴 Studio 3쪽 TSV](browser/original-fonts/silhouette.tsv)의 최저 raw는 99.37309%다.
같은 문서의 정상 파일 입력·initializeDocument 완료 후 공식 host-font API와 지정 환경을 사용했다.
선택적 로컬 글꼴 안내는 실제 닫기 버튼으로 닫았다. 실 캡처는 문서 쪽 사각형이며 DOM 그림과 별도 레이어를 포함한다.

- [원본 글꼴 Canvas2D 21·25·26쪽](browser/original-fonts/report.json): CanvasKit 요청에서 기존 미지원 CFF 자원 때문에 Canvas2D로 전환된 결과는 실제 CanvasKit 성공으로 세지 않는다.
- [지원 TTF paint 대조군의 실제 CanvasKit](browser/supported-ttf-paint-control/report.json): 변경 그림이 놓이는 26쪽, compat/default 모두 실제 software backend다.
  원본 저장 메트릭을 보존하고 paint에 공개 TTF를 공급했다. 원본 글꼴의 모양 일치 증거와 구분한다.
- [일반 CanvasView의 초기·zoom/DPR/resize·복원](browser/summary.json): 같은 26쪽에서 Canvas2D·CanvasKit 두 mode 각각 3상태를 확인했다.
  대화상자를 숨기거나 캔버스만 추출하지 않고 실제 화면을 캡처했다. 그림의 Print 대비 정규화 위치와 runtime 자원 실패를 확인했다.
- [실제 SVG browser raster 21·25·26쪽](svg-validation.json): 원본 font profile, 최저 99.38112%.

추가 탐색용 3쪽 대체 글꼴 대조군에서 발생한 21쪽 family 준비·25쪽 glyph 매핑 오류는
[base와 후보에서 같은 서명](browser/exploratory-control-base-comparison.json)으로 재현됐다.
그 탐색 실행을 통과로 표시하지 않으며, 원본 글꼴의 제품 경로와 위 26쪽 실제 소비 검증을 각각 기록한다. 가드·임계값은 바꾸지 않았다.

## #9156 결과의 재사용

[#9156의 정본 증적](../../open_prs_20261117/README.md)을 현재 base의 수용 근거, 원 이슈 조사, 독립 Print 출처와 보호 18종의 기존 결과로 참조한다.
그 source의 전체 회귀·18종 결과를 이 후보의 실행으로 바꾸어 표시하지 않았다. 이번 후보의 추가 3보정에 필요한 검증을 별도로 실행했고 18종 전체 sweep은 반복하지 않았다.
[고정 참조 정보](base-pr9156.json), [글꼴 바이트 공급 식별](font-supply.json)을 함께 남겼다.
공유 cache가 base WASM을 즉시 재사용한 첫 후보 패키지는 제출 근거에서 제외하고 실제 재컴파일한 `312c607c...` 패키지만 사용했다.

## Native 검사 자동 실행 경로 보완

제품 producer는 계속 `fcf2e11b1727297743343446333132e1409737f1`이다. 테스트 head
`699c0afbbc86db1252128b8dcc09630038a8d971`에서 원본 TTF/TTC 검사 2개를
`installed_font_program_order.rs`에서 기존 Native CI 호출 대상 `skia_native_glyph_replay.rs`로 이관했다.
[본문·helper 동일성과 기존 CI 호출](native-ci-registration/routing-and-equivalence.json)을 확인했고 workflow·분류 정책은 변경하지 않았다.

- [현재 테스트 head 실제 실행](native-ci-registration/tests.json): 기존 Native CI 명령 24 PASS(이관한 2개 포함), 집중 33 PASS, 전체 nextest 13,718 PASS / 0 FAIL / 45 skip. 전체 실행은 890.457초, 4 slow다.
- [현재 head lint](native-ci-registration/windows-lint.json): fmt, root/Native/WASM Clippy, Native all-targets, workspace build/all-target Clippy, suite 정책 PASS.
- [실제 최소 글꼴 노출](native-ci-registration/minimum-font-runtime-probes.json)과 [합성 자원 신원](native-ci-registration/fixture-identity.json)을 기록했다.
- [검증 입력 커밋 확인](native-ci-registration/input-commit-audit.json): 원본·독립 Print·환경 파일 18개의 실행 바이트와 초기 제출 commit이 같고, 테스트 이관에서도 변경되지 않았다.
- [새 base의 CI 운영 변경과 clean merge](native-ci-registration/base-advance.json)를 확인했다. 제품·WASM 소비 증적은 동일한 제품 입력으로 검증한 위 producer 결과다.

이전 게시 head의 [Proptest checkout 취소](native-ci-registration/previous-head-ci.json)는 테스트 실행 성공으로 세지 않는다. 새 제출 head의 GitHub CI를 별도로 확인한다.

정본 보호 18종은 **현재 후보의 통합 담당자 검증이 미완료**다.
[원본 글꼴 공급 확인 상태](native-ci-registration/protected-font-qualification.json)를 남기며, 동일 공급 조건의 전쪽 검증을 완료하기 전까지 병합을 보류한다.
위 4종 공통 보호·편람 gate·sample4 검증을 이 18종 전체 검증으로 확장해 표시하지 않는다.

## Code head CI와 현재 통합 보류

정확한 제출 code head `0b54ec2b3f4fcb416c215f3c152c37b299202443`의
[5개 workflow 실제 성공·PR 신원 영수증](native-ci-registration/code-ci-0b54.json)과
[Native 두 검사 실제 CI PASS 발췌](native-ci-registration/native-ci-job-excerpt.json)를 보존했다.
문서 trailing head의 CI는 별도로 확인한다.

#9156의 보호 18종·Native/fresh WASM 각 1,771쪽 무하락 통과는 유효한 base 검증이다.
현재 후보의 전체 보호 검증을 대신하는 결과로 사용하지 않는다.
[현재 공급 조건 확인](native-ci-registration/protected-font-qualification.json)은 선언된 309개 중
정확한 해시 286개를 찾았고 23개가 미확인인 재고이며, 실제 사용 프로그램의 확정 목록이 아니다.
현재 후보의 동일 조건 전쪽 검증이 남아 있으므로 병합은 보류한다.
