# #7642 PDF 글꼴의 PostScript 프로그램 식별 보존

Issue: #7642.

## 독립 근거와 원인

`pdf/pr7704/6778-square-2020.pdf`의 첫 쪽 제목은 `HCRDotum-Bold`를
요청한다. 공식 설치 `HANDotumB.ttf`의 name ID 6이 이 이름이고, family
ID 1은 `HCR Dotum`이다. PDF subset의 57개 고유 윤곽·전진폭은 이 Bold
설치본과 모두 일치하며 Regular 설치본과는 윤곽이 하나도 일치하지 않는다.
명시적인 family 연결 진단은 raw 94.59070%를 95.53089%로 개선했다.
이 값은 진단이며 최종 후보의 통과 증거가 아니다.

현재 Native는 요청을 family 목록으로만 거른 뒤 일반 폴백으로 보낸다.
PDF가 보존한 PostScript 이름을 실제 글꼴의 동일 name ID 6으로 찾는 경로가
없다. 이미지 수 대조군의 `Arial-BoldMT`·`MalgunGothicBold`도 같은 경계를 사용한다.

## 구현과 반례

- 기존 family 조회와 출처별 우선순위는 보존한다. family가 없으면 실제
  Typeface의 PostScript 이름이 정확히 같은 프로그램만 허용한다.
- OS의 이름 조회가 반환한 일반 폴백은 PostScript 이름이 다르면 거절한다.
  필요하면 실제 시스템 style set을 색인하며 renderer의 시스템 목록과 함께 재사용한다.
- supplied catalog에서도 PostScript 식별자를 family 별칭과 분리해 저장한다.
  FULL_NAME 레코드·임의 접미사 제거·문서 ID·관측 좌표를 치환 근거로 쓰지 않는다.
- family 요청은 기존 CSS 굵기 선택을 유지하고, 명시적 PostScript 요청은
  그 식별자가 가리키는 face를 보존한다. 없는 이름·전체 얼굴 이름·출처 우선순위를 대조한다.

## 검증

기존 supplied family/full-face 계약과 독립 name ID 기반 합성 계약을 확인한다.
공식 PDF 15문서·364쪽의 Native/fresh WASM 및 전체 102문서에서 base 대비
하락 여부를 다시 확인하고 제목·날짜·본문·그림을 직접 판독한다.
새 실물 회귀는 전쪽 90%와 판독을 충족한 뒤 별도 이름으로 추가한다.
빌드 중인 af71 후보를 중단하지 않으며 다음 후보에 이 변경을 묶는다.
