---
kind: snapshot
status: active
canonical: mydocs/pr/pr_7212_review_impl.md
last_verified: 2026-09-17
---

# PR #7212 시각 검증 환경 보정

## 분석

초기 캡처의 D2Coding/HCR Dotum 대체를 원 face의 굵기 증거로 사용할 수 없었다.
원 face의 glyph·advance를 유지해 검증 도구에 명시적으로 공급하고 동일 PDF와 비교한다.
renderer의 0.02em 상수를 픽셀 점수에 맞춰 조정하지 않는다.

## 보정·검증

- Visual Sweep에 CLI와 같은 --embed-fonts=full 및 반복 --font-path 옵션을 추가했다.
  Native와 WASM은 동일한 폰트 CSS만 사용하고 text/geometry/render tree는 각 backend에서 얻는다.
- mode와 font file SHA-256을 provenance에 넣어 폰트 변경 시 resume 재사용을 거부한다.
  기본 local alias 경로는 유지한다. Python sweep 검사 53개 통과.
- win10-ted의 Windows Fonts H2HDRM.TTF/gulim.ttc를 비공개 scratch로 읽고 fontTools로
  검증 문자의 subset을 만들었다. 각 105개 cmap 문자의 decomposed outline과 hmtx가 원본과
  동일함을 검사했다. 실제 Chrome 선택은 GulimChe/HYHeadLine-Medium이며 OTS 오류가 없다.
- Mac session 등록을 시험했으나 LastResort였으므로 등록을 해제했다. 사용한 최종 경로는
  유효한 폰트 테이블을 가진 subset을 full 모드로 공급한 scratch SVG다. 폰트를 설치·커밋하지 않았다.
- base Native, 통합 Native, fresh WASM의 같은 원본 p1·p2 compare/overlay/review를 산출했다.
  수정 범위는 0.02em 명시 획이며 p2의 기존 표/본문 배치 차이는 해결 대상으로 승격하지 않는다.
  최종 head 재캡처·해시·개별 결과는 [검토 문서](archives/pr_7212_review.md)에 반영한다.

## 결과보고·커밋

환경 대체에 의한 필수 증거 부족을 해소할 수 있는 경로를 확보했다. source 폰트와 scratch SVG는
비공개로 유지하고 PNG만 커밋한다. 동일 native/WASM glyph 선택과 남은 차이를 최종 기록한다.
