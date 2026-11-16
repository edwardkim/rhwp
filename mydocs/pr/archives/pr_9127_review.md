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


## 실제 Native 소비 확인

macOS 최소 글꼴 격리에서 Ubuntu 배포 Noto TTC의 KR face를 명시 공급했다.
원본 없는 돋움·굴림 요청과 명시 Noto Sans CJK KR 요청의 한글 2자씩 총 6자는
renderer 소유 관측에서 해당 custom 프로그램을 선택했고 실제 Native raster에 잉크가 있었다.
Mac 일반 고딕에서 Apple SD Gothic Neo가 먼저인 기존 순서는 변경하지 않았다.
Linux·Windows의 설치 목록 실행을 완료했다고 확대하지 않으며, 글꼴 바이너리는 Git에 넣지 않는다.
[실제 소비 기록](../assets/all_open_prs_20261116/noto-cjk-native-consumer.json).


## 통합 후보 최종 소스의 중간 검증 결과

제품 소스 `167921b3c5`와 검증 head `3e9ce11f9c`에서 최소 글꼴 macOS 전체 회귀는
**13,696 PASS·0 FAIL·45 skip**이다. Native Skia·세 Clippy·workspace build·fmt·
suite 정책·unit tier·문서 테스트를 포함한 [Rust 13단계](../assets/all_open_prs_20261116/rust-gates-final.json)는 통과했다.
통합 검토의 공식 데스크톱 Print 66종·74쪽은
[Native](../assets/all_open_prs_20261116/native-claims-final.json)·
[fresh WASM](../assets/all_open_prs_20261116/wasm-claims-final.json) 각각 전쪽 90% 이상이다.
이 집계는 다른 PR의 입력을 이 PR만의 개선으로 계산한 값이 아니다. PR별 주장·반례는 위 접수 범위와
개별 보정 증거로 판정한다. 관계형 검사 보정 뒤 9단계는 이 head에서 다시 실행했고, 제품 및
해당 검사 소스가 바뀌지 않은 Native Skia·직접 PDF·placeholder·doc 4단계는 원 실행 신원을
보존해 재사용했다. [최신 회귀 판정](../assets/all_open_prs_20261116/relation-test-final.json)을 따른다.

[Native/base 비교](../assets/all_open_prs_20261116/native-base-head-comparison.json)와
[WASM/base 비교](../assets/all_open_prs_20261116/wasm-base-head-comparison.json)는 66종·74쪽에서
각각 상승 59쪽·하락 0쪽이다. [공통 보호 8조건·명시적 편람 383쪽](../assets/all_open_prs_20261116/public-gates-final.json)도 통과했다.
[실제 소비](../assets/all_open_prs_20261116/actual-consumers-final.json)는 Native PNG 16개·브라우저 48개를
실행했고 runtime fallback/replay hard 위반은 없다. report-only 최소 잉크량 32 FAIL과 자동 선택
readiness 미실행은 그대로 공개하며 통과로 바꾸지 않았다. 정본 전체 비교의 원본 글꼴 공급 재현을
최종 확인 중이므로 아직 통합 PR 준비 완료나 머지 승인으로 기록하지 않는다.
