# 용지 기준 글상자의 내용 하한과 아래 정렬

2026-11-06, 부모 `4b03792e56be4115145e3a20160da14378b3ab7c`에 두 제품 파일의 보정을
적용했다. validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7375/paper-textbox-minimum-20261106/validation.json`)에 source 파일·입력·독립 PDF·실행 파일·WASM·
글꼴 공급과 테스트의 SHA-256을 고정했다. 로그와 임시 driver는 ignored output에만 둔다.
실행은 Mac이며 fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt`
로컬 대체 빌드다. Docker 최적화 빌드의 결과가 아니다.

## 원본과 독립 기대값

사용자가 지정한 `46790458ceaad2abd5c86168161798cb01f0473e`의 다음 세 입력은
현재 파일과 바이트가 같다. #8696에서 사용한 PDF를 재인쇄·교체하지 않고 그대로 재사용했다.

| key | 입력 | 독립 한컴 Print PDF |
| --- | --- | --- |
| control | `samples/header-footer-paper-frame.hwpx` | `pdf/pblsketch_review_20261030/header-footer-paper-frame-2024.pdf` |
| repro | `pdf/issue8687/font_list_name_fallback.hwpx` | `pdf/issue8687/font_list_name_fallback-2024.pdf` |
| used | `pdf/issue8687/font_list_name_used_fallback.hwpx` | `pdf/issue8687/font_list_name_used_fallback-2024.pdf` |

세 입력은 같은 꼬리말 글상자 형상을 갖는다. 선언 높이1200HU에 현재 글자 높이900HU와
위·아래 안 여백283HU씩을 담을 수 없다. 저장 LineSeg의1000HU는 현재 글자 크기와도 다르다.
독립 Print의 사각형 경계는 세 PDF 모두 `(73.6399994, 679.0570068, 163.5919952,
693.6810303)pt`다. 높이는14.6240234pt이며, 현재 글줄과 여백의1466HU 내용 하한에 대응한다.
Print 장치의 좌표 양자화와 화면 좌표의 미세한 차이는 남으므로 RGB 완전 일치를 뜻하지 않는다.

선행 #8165는 머리말·꼬리말의 용지 기준 틀을 복구했고, #8696은 목록형 글꼴 이름의
오염과 정상 TTF 대체를 복구했다. 두 PR의 검토 문서는 BOX 높이 차이를 잔여로 명시했다.
이번에는 그 잔여를 직접 수정하며 기존 글꼴 대체·틀 소유 검사를 유지한다.

## 공통 보정과 적용 경계

`shape_layout::layout_shape`에서 현재 글줄·안 여백의 하한을 **원점 계산 전에** 확정한다.
공통 재조판으로 얻은 같은 글줄과 늘어난 높이를 위치 결정·쪽 안 제한·캡션·paint에 전달한다.
아래 정렬 글상자를 뒤에서 따로 이동하거나 저장 원본의 높이·LineSeg를 바꾸지 않는다.

대상은 크기 보호가 없는 가로 글상자 중 흐름을 밀지 않는 앞/뒤 배치다. 글자처럼 취급,
어울림 본문 점유, 명시적 clip, 세로쓰기, 제어 개체를 가진 문단, 변환·묶음·셀 직접 배치는
기존 프레임 소유를 유지한다. 현재 줄·여백이 선언 상자에 들어가면 기존 저장 줄을 그대로 쓴다.
문서명·쪽 번호·특정 너비에 대한 예외는 없다.

[한컴 글상자 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/insert/textbox/textbox.htm)의
내용 증가 시 높이 확장과
[크기 고정 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/insert/objectattribute/objectattribute(size).htm)을
대조했다. 고정 크기·빈 글줄·공백 글줄만 바꾼 세 반례를 별도 한컴2024 Print로 출력했다.
고정 크기는11.9870605pt를 유지하고, 빈 글줄·공백 글줄은 원본과 같은14.6240234pt가 된다.
보이는 글자가 없다는 이유로 글줄의 물리 높이를 없애지 않는다. 반례 원본과 PDF는
`pdf/issue7375/paper-textbox-{protected,empty,space}.*`에 보존하며 쪽수 원장에도 명시 등록했다.

새 하한 보정의 적용 경계도 검증했다. 저장 줄이 쪽 좌표인 경우, paint는 쪽 원점을
빼는데 초기 하한 계산은 이를 내용에 더했다. 같은 위치·내용의 상자/쪽 좌표 반례에서
19.54667px와39.54667px로 갈라지는 실제 실패를 확인했다. 기존 paint의 원점 판별·정규화
함수를 함께 사용해 하한도 같은 좌표계로 계산한다. 스트림 첫 유효 vpos로 좌표 소유를
확인하며 뒤쪽 일부 줄만 원점을 빼지 않는다. 독립 한컴2024 Print 두 건은 같은 상자 경계를
출력했다. `paper-textbox-vpos-{local,page}.*` 원본·PDF와 별도 관계 검사로 이 경계를 보호한다.

## 전쪽 Native·fresh WASM 결과

현행 Visual Sweep, 96DPI·compat2024·Native PNG·PDFium·색상 경계 보정 전2px raw다.
각 입력은 전1쪽이며 누락·쪽수 불일치는 없다. 원본3종과 반례5종을 섞어 동일 입력으로
주장하지 않는다. 모든 TSV는 해당 하위 폴더의 `silhouette.tsv` 원문이다.

| 입력 | 보정 전 Native | 보정 전 fresh WASM | 보정 후 Native | 보정 후 fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| control | 91.10022% | 91.14414% | 100.00000% | 100.00000% |
| repro | 91.21255% | 91.10704% | 100.00000% | 100.00000% |
| used | 91.21255% | 91.10704% | 100.00000% | 100.00000% |
| protected | 반례 별도 출력 | 반례 별도 출력 | 100.00000% | 100.00000% |
| empty | 반례 별도 출력 | 반례 별도 출력 | 100.00000% | 100.00000% |
| space | 반례 별도 출력 | 반례 별도 출력 | 100.00000% | 100.00000% |
| vpos-local | 초기 하한 보정100.00000% | 초기 하한 보정100.00000% | 100.00000% | 100.00000% |
| vpos-page | 초기 하한 보정88.68078% | 초기 하한 보정88.65435% | 100.00000% | 100.00000% |

보정 전은 새 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`와 리베이스 후보가 같은 결과다.
보정 후에는 동일한 입력·PDF·글꼴 공급을 사용했다. 시각 검증의 개인 글꼴 공급과,
시스템 글꼴을 차단한 회귀 실행을 구분한다. 글꼴 프로그램은 커밋하지 않는다.

세 원본의 [Native control review](native/control/review.png)·[overlay](native/control/overlay.png),
[repro review](native/repro/review.png)·[overlay](native/repro/overlay.png),
[used review](native/used/review.png)·[overlay](native/used/overlay.png)와
[WASM control review](wasm/control/review.png)·[overlay](wasm/control/overlay.png),
[repro review](wasm/repro/review.png)·[overlay](wasm/repro/overlay.png),
[used review](wasm/used/review.png)·[overlay](wasm/used/overlay.png)를 직접 열었다.
각 경로의 protected/empty/space 하위 폴더에도 review·overlay를 함께 보존했다.
머리말 표, 본문3줄, BOX의 글줄·상자와 같은 아래 기준을 쓰는 FOOTER C를 대조했다.
vpos-local/vpos-page도 양쪽 review와 overlay를 직접 열었다. 원본3종과 고정/빈/공백3종의
최종 fresh PNG12건은 초기 하한 보정에서 직접 판독한 PNG와 해시가 모두 같다.
최종 panel은 최종 source의 fresh PNG로 다시 합성했다.

## 회귀와 남은 검증

- 새 관계형7건: 현재 글줄과 두 안 여백의 포함, 아래 정렬 공유, 크기 보호,
  빈 줄·공백 줄, 저장 vpos 좌표계와 그리기 전후 원본 직렬화 보존을 검사한다.
- 새 고정형5건: 위 독립 PDF 경계를96DPI ±2px로 고정한다. 같은 페이지의 두 경로100%와
  직접 판독을 선행했다. 관계 검사는 별도 파일로 유지한다.
- 실제 새 base: 새12건 중2 PASS·10 FAIL. 보정 후: 새12건 모두 PASS.
  원점 관계1건은 초기 하한 보정에서 실제1 FAIL→최종1 PASS였다.
  빌드 실패나 실행 스크립트 오류를 결함 재현으로 세지 않았다.
- 기존 머리말·꼬리말1건과 #8696 API6건은 양쪽7 PASS. 관련 도형·글상자·표 쌓기32건도
  최종 후보에서 PASS다. 최종 집중 묶음은 합계51 PASS이며 설치 글꼴을 차단한 Mac 실행이다.

이 증거는 추가한 세 원본과 다섯 반례의 완료다. 기존 #4138의 남은5쪽 등 브랜치 전체의98%
요구, 최종 전체 회귀·게이트·실제 출력 소비 경로의 완료를 대신하지 않는다. PR 제출은 보류한다.
