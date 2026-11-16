# Native 합성 기울임의 같은 계열 분리 글꼴 공급

## 원 해결과 실제 공급 확인

PR #8397의 `99a176b7cb3`은 합성 Bold를 실제 보통 프로그램의 스타일로 되돌린 뒤
같은 family의 다른 글리프 공급자를 열거했다. `b44baac4df5`는 같은 형식에서 OS가
선택한 프로그램을 먼저 유지하도록 보완했다. 현재 구현은 그 경로에서 합성 Oblique를
되돌리지 않아, 실제로는 정체인 다른 한글·영문 프로그램을 후보에서 제외한다.

[실제 Native 공급 기록](verification-record.md)에서 sample10 12쪽의
기울임 ‘구’ 두 글자는 요청한 `Myeongjo` 대신 `Haansoft Batang`으로 표시된다.
같은 호스트의 `Myeongjo-HGMJ`는 해당 글리프를 가진 정체 CFF다. Fontconfig가
영문 `Myeongjo-ENMJ`에 붙인 Oblique는 실제 파일을 다시 열면 Upright로 확인된다.

위반된 계약은 같은 family의 분리 프로그램에서 글리프를 먼저 찾는 선택 순서다.
조판 폭·글자 원점·굵기 획·기울임 계수를 바꾸어 다른 프로그램의 잉크를 맞추지 않는다.

## 소비 경계와 보정

`text_typeface_candidates_for_source` → `typeface_candidates_for_families_for_source` →
실제 프로그램 스타일 → `matching_system_programs` → 문자별 공급자 선택 →
기존 `fonts::finish`·실제 paint 순서를 따른다. 추가 열거에 쓰는 스타일만 기존의
`unobliqued_typeface`로 복구한 뒤 합성 Bold 제거를 적용한다.

원래 OS 선택은 사다리 선두에 유지하고, 같은 family·같은 실제 스타일의 다른
프로그램만 보완한다. 실제 italic/oblique 프로그램과 활성 `ital`·`slnt` 축은 기존
복구 함수가 정체로 바꾸지 않으므로 그대로 유지한다. 다른 family·문서 ID·쪽 번호·
글자별 예외나 점수를 위한 수치를 추가하지 않는다. 레이아웃·원본·기준 PDF는 변경하지 않는다.

## 독립 반례와 검증 범위

MIT `RHWPHftOutline.otf`의 독창적인 사각형을 ‘가’와 ‘나’에 각각 매핑한 두 개의
CFF를 격리된 Fontconfig에 공급한다. 일반·굵은 합성 기울임에서 두 글자를 같은
family에서 선택해야 하며, 실제 최종 raster도 같은 프로그램의 명시 공급과 일치해야 한다.
설치 글꼴에 의존하지 않고 실제 문서의 절대 px 좌표나 기존 오답 PNG를 고정하지 않는다.
새 사례는 CI에서 이미 실행하는 `native_fractional_glyph_origin`에 포함한다.

보정 전 실제 반례는 `bold=false, char=나`의 공급자 선택에서 실패했다(exit 101).
실행 원문·입력 해시는 ignored output의 `synthetic-slant-family-before.json`과
`logs/synthetic-slant-family-before.log`에 기록했다. 이 선택 경계는 해당 rlib와 현재 base에서
같은 소스이며, 당시의 별도 그리기 후보가 이 실패를 입증하는 근거는 아니다.

## 채택하지 않은 앞선 진단

행렬·em 윤곽 후보는 공개 affine 관계 검사 7건을 통과했지만 실제 보호 TSV 하락을
해소하지 못해 작업 코드에서 되돌렸다. 기울임 스타일을 정체로 바꾼 진단은 글꼴 공급자도
바꾸므로, 그 개선을 em·행렬의 단독 효과로 해석할 수 없다. 실제 공급자 관측으로
원인 계층을 정정했다. 최종 제품의 전쪽 비교와 필수 Native 검사를 마친 뒤 채택한다.

## 보정 후 실제 실행

[실행 영수증](verification-record.md)에서 고정한 소스·바이너리로
최소 글꼴 환경의 집중 검사 76건과 라이브러리 4,035건이 모두 통과했다. 새 분리 CFF 사례는
보정 전 FAIL, 보정 후 PASS다. 실제 italic·굵기 소유 반례, Print 기하, HFT 윤곽,
그림 누락·직접 PDF 출력 검사 및 Clippy `-D warnings`도 통과했다.
[원 PR 입력의 Native 전쪽 TSV의 검증 요약](verification-record.md)는 전 14쪽 100.00%다.

[보호 선행 대조](verification-record.md)에서 sample10의
하락했던 12·15·17·18·20쪽은 각각 99.99%·100.00%·100.00%·100.00%·100.00%로 복구됐다.
선택 쪽 결과이며 전 763쪽 검증을 대신하지 않는다. 별도 중첩 표의 기존 하락은 이 선택
보정으로 해결되지 않았으며, 해당 결과를 기준 갱신이나 통합 완료의 근거로 쓰지 않는다.
