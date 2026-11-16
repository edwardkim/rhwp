# 최신 devel #9106 반영 후 재검증

상태: 필수 로컬 재검증 완료. Native 10쪽 기존 1행 차이는 [작업지시자가 한정 수용을 승인](native-floor-decision.md)했다. 나머지 gate도 모두 완료했으며 기존 승인된 PR·CI·self-review·merge·후속 처리를 계속한다.

- 이전 제출 후보: `105be723b2f850ba5482fd28aa07e6a74d23e7d2`
- 새 devel: `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0` (#9106)
- 충돌 없는 실제 병합 source: `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`
- 새 보호 TSV SHA-256: `2280e97844b2cb262ea73e9c49f09f56c434097fed49af75fd9a7f6198d43da7`

## 반영 기능과 실제 경계

#9106의 분할 셀 프레임·바깥 여백·Print 정적 TTF 표시 크기·SVG 가는 괘선·Native SVG 글꼴 준비·CanvasKit 사전 검사 예산을 보존한다. 원 해결 PR의 검토·갱신 영수증·독립 PDF·실제 소스와 새 base/head를 대조했다. 기존 두 기여의 제품·assertion은 바꾸지 않았다.

새 base와 후보의 Native/fresh WASM 생산을 분리했다. 공용 target이 다른 worktree의 rhwp 산출물을 재사용한 진단 결과는 채택하지 않았다. 저장소 매뉴얼에 따라 rhwp package의 해당 profile 캐시만 무효화한 뒤 실제 컴파일한 새 Native와 WASM을 사용한다.

## 독립 HFT 기호 공급 복원

선행 #9106은 별도의 `independent-symbols-v2` 프로그램을 사용했다. 현재 로컬의 이전 독립 프로그램은 U+2027만 있어 5쪽의 U+203B가 대체됐다. 원본 HWP의 해당 run은 실제 한양중고딕 HFT 13pt다. 깨끗한 별도 표본의 11·13pt U+2027/U+203B를 관리형 한컴 2020 `11.0.0.9136`의 실제 Print (method 0, one-up)로 출력했다. 대상 PDF를 복원 원료로 사용하지 않았고 이동·배율·윤곽 보정을 하지 않았다.

- 입력 SHA-256: `5f0b7005a448e0a8e18fa4d5996ffda174e3c5f63159dd2b2b0fae53800769e0`
- 독립 Print PDF SHA-256: `72d4e189e5bc32ed3fbda41463035d88fc36160fe449253b668a79d861948c4f`
- job: `2070f627-2e3a-4954-a1ba-64da0504607f`
- 원본 Type3 U+203B 프로그램 SHA-256: `d12a47be06f15ebccc180d2cc925555f6e2d9e250eadadf61e53242906b208bb`
- 로컬 글꼴 프로그램 SHA-256: `bdd255d079fceef18d0ce19e057af0db8cfb6956b2a0d2a170eedad5c336e90f`
- 지속 로컬 경로: `/home/edward/mygithub/ttfs/local-validation/hft-symbols-v2/`

기존 U+2027 프로그램은 보존했다. 복원 스크립트의 재실행도 같은 새 컨테이너 SHA를 산출했다. 사설 글꼴과 복원용 PDF는 Git·배포물에 넣지 않는다. [5쪽 직접 비교](latest-devel/hft-reference-mark-restored.png)에서 기호 모양을 확인했다.

## 보호 하한 대조의 현재 결과

동일 입력·PDF·96dpi·compat 2022·2px raw의 전 17쪽 TSV를 먼저 산출했다. 기호 복원 후 Native 5쪽과 base WASM 5쪽은 100.00%로 회복했다. 남은 Native 10쪽은 base·후보 모두 99.85%이며 새 보호값 99.86%와 차이가 남는다. [10쪽 PDF/Native/WASM 같은 영역 확대](latest-devel/nested-p10-line-inspection.png)로 확인했다.

선행 Native 공급 굴림 SHA는 `4b9ac63e8920ed1bae29c068025ed30493464c18ee18617887daa18e59189226`이며 현재 공급본은 `05a38ff214ee932c6147a4e697b494040fd68ce654795f10f723c01d7bb8ee12`다. 선행 revision은 실제 head 값 5.029998(5.03), 현재 파일은 5.039993(5.04)로 다르다. 선행 366글리프의 윤곽·전진 일치를 현재 공급본이나 파일·렌더 조건 전체의 동일성으로 주장하지 않는다. 현재 TTC를 단독 SFNT로 추출한 대조에서는 head checksum을 제외한 모든 table 바이트가 같았으며 Native 10쪽은 그대로 99.85%였다. 이 진단을 정식 통과본으로 채택하지 않았다.

선행 기록이 보존 대상으로 지정한 `visual-sweep-protected-fonts-20261115.zip`은 현재 호스트에 없어 위치를 문의했다. 이 차이를 기존 base와 같다는 이유로 면제하거나 기준값·허용치를 바꾸지 않는다.

나머지 전체 회귀·lint·Native 공식 검사·fresh WASM·편람·공개 공통 검사는 새 source에서 실제 완료했다. 직전 후보의 13,614개 성공을 새 검증으로 재사용하지 않았다.

## 추가 완료 사항

- 새 후보의 최적화 WASM build 종료 0, 실제 wasm-opt 포함 478.9초. WASM SHA-256: `ff8f71e51aeb68f7e1388d19d3a5504617bc6ff82246e5d301a0323da099d12e`. base WASM `34ad4429c129014ae367c32eea58a663048b8b8b58f6231867a4a57ce3cc6cb7`과 분리했다.
- 새 Native SHA-256: `5f12d9237fa578948cc01eaa233f16f213baf0a4d540d86b63fbb0ef8bac6c0e`; base는 `a1534f965026efa83c509f88cbe7713626bc9c0571767ef9152056b8839abfe2`.
- 새 base·후보의 각주 50쪽·중첩 표 17쪽은 Native/fresh WASM 각각 전 67쪽 PNG가 바이트 단위로 같다. 새 기여의 안내문 두 표 배치는 별도로 실제 개선을 확인했다.
- 실제 CLI의 한 글자·네 글자·안내문 3입력 반복 편집 저장과 HWPX 교차 저장을 새 Native로 다시 실행했다. 후보는 FIELD_END와 표 순서를 두 차례 보존하며, base의 값 있는 필드 끝 토큰 이동 오류를 재현했다.
- 복원한 기호를 공급한 중첩표의 전 17쪽 fresh WASM은 base·후보 모두 최저 99.86%, 공식 표시값 하락 0건이다. Native는 앞서 기록한 10쪽1건이 남는다.
- 대상 PDF의 글꼴은 **비교 진단에만** 읽었다. 현재 굴림 5.04와 독립 PDF 굴림 5.03의 관측372문자 윤곽·전진은 모두 같다. 이 사실로 전체 파일·raster 조건·보호값 통과를 주장하지 않으며, 대상 PDF의 font 프로그램을 저장하거나 렌더링 입력으로 공급하지 않았다.

## 목표 문서의 새 head 직접 판독

새 Native/fresh WASM의 안내문·누름틀 없음 각 1쪽은 양쪽 모두 raw 100.00%다. 새 review/overlay의 PNG는 실제 전쪽 TSV를 측정한 raster와 바이트가 같음을 확인했다. 글상자 안 표의 행·열, 두 표의 위아래 순서·간격·외곽 포함과 누락 없음은 같은 독립 PDF와 직접 판독했다.

- 안내문: [Native review](latest-devel/candidate/native/guide/review.png)·[overlay](latest-devel/candidate/native/guide/overlay.png), [fresh WASM review](latest-devel/candidate/wasm/guide/review.png)·[overlay](latest-devel/candidate/wasm/guide/overlay.png)
- 누름틀 없음: [Native review](latest-devel/candidate/native/no-field/review.png)·[overlay](latest-devel/candidate/native/no-field/overlay.png), [fresh WASM review](latest-devel/candidate/wasm/no-field/review.png)·[overlay](latest-devel/candidate/wasm/no-field/overlay.png)

전쪽 TSV는 `latest-devel/<base-or-candidate>/<native-or-wasm>/<입력명>/`에 구분했다. 이 목표 통과로 Native 보호 하한의 미완료1건을 면제하지 않는다.

2026-11-16 작업지시자 결정: Native 10쪽 기존 1행 차이의 한정 예외 승인. 보호 정본과 측정값은 변경하지 않으며, 다른 실패나 새 회귀에는 적용하지 않는다.

## 새 source 전체 Rust 검증의 실제 완료

- 최소 글꼴 전체 회귀: **13,632 passed / 0 failed / 기존 skipped 45개**, 실행584.016초·컴파일 포함1,216.2초, 종료0.
- 집중 9개 묶음30건: 모두 통과. 원 PR의 두 기능과 #9106의 분할 셀·바깥 여백·Print TTF em·SVG stroke·Native SVG 프로그램 준비·CanvasKit 예산을 실제 실행했다.
- format·Native Clippy·WASM32 Clippy·workspace build·workspace all-target Clippy: 모두 종료0.
- 명시적 Native 편람 gate 383쪽: 종료0·passed·하락 0건. [전쪽 TSV](latest-devel/handbook-gate/silhouette.tsv).
- 사설 글꼴을 가린 공개 4문서 × Native/fresh WASM 8개 실행: 모두 종료0·passed·raw 최저 100.00%.

Native 공식 3종은 4,073건 모두 통과했다. 전체 WASM 보호도 완료했으며 아래 최종 판정에 연결한다.

## 최적화 WASM의 실제 반복 저장

시각 캡처와 별도로 같은 최적화 web package를 Node v24 WebAssembly에서 초기화하고 공개 HwpDocument API를 실제 실행했다. `insertText(0,0,0,"x") → exportHwp → 새 HwpDocument → deleteText(0,0,0,1) → exportHwp → 새 HwpDocument` 뒤 HWPX 저장·재열기·재저장을 수행했다. 삽입·삭제 응답의 ok/charOffset과 매 단계 필드 이름·값을 확인했다. 실제 브라우저의 WASM 시각 검증은 별도로 완료했다.

한 글자·네 글자·안내문 3입력에서 독립 Python CFB/DEFLATE/PARA_TEXT 토큰 판독을 적용했다. base의 두 기입 입력은 끝 토큰이 표 뒤로 이동했고, 후보는 세 입력 모두 source/첫 저장/둘째 저장의 FIELD_END·표 순서가 같다. HWPX는 base·후보의 두 차례 저장 모두 `fieldBegin → fieldEnd → tbl → tbl`로 기존 정상 순서를 유지했다. 원시 로그·응답 JSON·중간 저장본은 Git에 넣지 않는다.

## 최종 완료 판정

- Native 공식 검사: lib 4,035건(3853+15+165+2), 그림 placeholder 2건, direct PDF 36건으로 **4,073 passed / 0 failed**다.
- 최소 글꼴 Rust 게이트 19개 실행(집중 9묶음 포함)은 모두 종료 0이다. 전체 13,632건·집중 30건·Native 공식 4,073건·모든 lint를 구분해 집계했다.
- [정본 보호 전체 판정](latest-devel/protected/README.md): Native/fresh WASM 각각 1,771쪽, 총 3,542쪽. 승인된 Native 10쪽 한 행 외 하락·누락·쪽수 불일치·측정 실패는 0건이다. 상승도 0행이므로 정본을 수정하지 않았다.
- [실제 각주 Native 50쪽](latest-devel/incoming/native/footnote.tsv)·[fresh WASM 50쪽](latest-devel/incoming/wasm/footnote.tsv) 및 [Native SVG 7·28쪽](latest-devel/incoming/native-svg-footnote.tsv)을 확인했다. p7의 한글·빨강 강조·줄바꿈, p28의 상자·각주 9/10/11과 본문 분리, p37의 분할 셀 끝 괘선·24/25 각주 표시·뒤쪽 내용 포함을 직접 대조했다. source 전체가 같다고 추측하지 않고 base·후보의 실제 PNG 134개가 같음을 확인했다.
- 직접 판독: [Native SVG p7](latest-devel/incoming/native-svg-007-top.png), [Native SVG p28](latest-devel/incoming/native-svg-028-bottom.png), [PDF/Native/WASM p28](latest-devel/incoming/footnote-028-bottom.png), [PDF/Native/WASM p37](latest-devel/incoming/footnote-037-bottom.png).
- 원 기여 head 두 개는 현재 HEAD의 조상이며, 최신 base와 `git merge-tree --write-tree`는 종료 0이다. source/header·원본·PDF·실제 생산 프로그램은 별도로 고정했고 문서 기록은 제품 동작을 바꾸지 않는다.

현재는 로컬 검증 완료 단계이며 새 acceptance PR의 exact-head GitHub CI·self-review는 제출 후 실제 실행한다. 그 성공과 최종 병합 상태를 미리 기록하지 않는다.

모든 Rust gate의 실제 명령·격리·종료 코드·시간은 [최종 실행표](latest-devel/local-gates.md)에 보존한다.
