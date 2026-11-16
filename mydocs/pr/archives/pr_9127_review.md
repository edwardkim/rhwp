# PR #9127 리뷰 — 설치된 Noto Sans CJK KR 대체 선택

## 최종 판정

**머지 보류 — 대체 체인·실제 글꼴 소비와 보호 문서 무회귀 검증 진행 중.**

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9127), 작성자 `donggyun112`.
- 원 head `7e5bbc0a195a667bbdb467a769aeb599905b13e9`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`의 API 확인을 마쳤고 기존 reviewer는 유지했다. milestone은 `v1.0.0`이다.
- #9104의 일반·빈 이름·돋움/굴림·KoPub 돋움 Light 대체 목록에 설치 family
  `Noto Sans CJK KR`를 추가한다. 정상 원본 글꼴을 이 대체로 강제 교체하는 변경이 아니다.

## 검증과 조판 원칙

`legacy_font_fallback_order` 및 관련 Native 글꼴 검사를 실제 실행한다. 최소 환경에서 한글 glyph를
담은 설치 CJK 프로그램이 대체 경로에 들어오는지와, 원본 TTF→TTC→OTF/CFF 선택 순서를
앞지르지 않는지 확인한다. 새로운 비공개 글꼴 배포를 추가하지 않는다.
원본 글꼴을 공급한 일반 환경의 보호 문서 Native/fresh WASM 결과도 별도로 확인한다.

체인 문자열 검사만으로 모든 OS의 실제 출력 통과를 주장하지 않는다. 실행한 host·공급 프로그램·
실제 렌더링 경로와 미실행 범위를 구분한다. 다른 PR의 대표 이미지로 이 PR의 증거를 대체하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·CI와 실제 대체 선택 결과를 한국어 존댓말로 알린다. 시각 증거를 판정 근거로
사용하면 해당 입력의 raw·review·overlay를 본문에 표시한다. #9104의 실제 해결 범위를 확인한 뒤
원 PR과 완전 해결된 이슈를 종료한다.
