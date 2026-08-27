# Task M100 #6149 — 저배율 눈금자·페이지 경계 최종 보고서

- **이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **브랜치**: `codex/issue-6149-low-zoom-ruler`
- **기준 commit**: `upstream/devel` `9be8b0562`
- **완료일**: 2026-08-27 KST
- **검증 서버**: `http://127.0.0.1:7720/`

## 결론

저배율에서 가로·세로 눈금의 숫자와 세부 눈금이 뭉치던 문제를 화면 픽셀 기반 LOD로 바꿨다.
눈금자는 마지막 편집 focus 페이지의 전체 용지 범위와 일치하고, 보이는 모든 페이지의 세로 눈금을
중복해서 그리지 않는다.

페이지 간격은 10%에서도 최소 6 CSS px를 유지하고 100%의 기존 10px에서 고배율로 자연스럽게
늘어난다. 모든 배치가 같은 gap 계약을 공유하며 밝은·어두운 테마에서 페이지 루트 외곽선을 식별할
수 있다. 실제 브라우저 검증 중 발견한 Rust/Studio render scale 하한 불일치도 보정해, 저배율 canvas가
레이아웃 슬롯보다 넓어 페이지를 덮는 정확성 결함을 제거했다.

문서 용지 크기, 본문 여백, 편집 좌표, 인쇄·저장 좌표와 물리 bitmap 하한은 변경하지 않았다.

## 최종 동작 계약

### 눈금자

- 숫자 간격은 최소 30px, 세부 눈금 간격은 최소 3.5px다.
- 단계는 `1·2·5 × 10ⁿ mm`에서만 고른다.
- 10%에서는 불필요한 1mm 눈금을 숨기고, 배율이 커질수록 같은 규칙으로 더 촘촘해진다.
- 가로·세로 모두 마지막 편집 focus 페이지 한 장의 시작·끝 경계를 표시한다.

### 페이지 간격과 경계

- 화면 gap은 `max(6px, 10px × zoom)`이다.
- 자동·한 쪽·두 쪽·맞쪽·여러 쪽·가로 이동이 같은 값을 사용한다.
- 페이지 루트만 테마 기반 외곽선과 그림자를 가지며 overlay layer는 중복 경계를 만들지 않는다.
- renderer bitmap scale 하한과 DPR 계산이 일치해 canvas CSS 크기와 VirtualScroll 슬롯이 같다.

## 자동 검증

```text
$ cd rhwp-studio && npx tsc --noEmit
exit 0

$ cd rhwp-studio && npm test
tests 1178, pass 1177, fail 0, skipped 1

$ cd rhwp-studio && npm run build
233 modules transformed, build success

$ node --test \
    rhwp-studio/tests/ruler-scale.test.ts \
    rhwp-studio/tests/page-gap.test.ts \
    rhwp-studio/tests/render-backend.test.ts \
    rhwp-studio/tests/virtual-scroll-page-arrangement.test.ts \
    rhwp-studio/tests/virtual-scroll-horizontal-pan.test.ts \
    rhwp-studio/tests/virtual-scroll-grid-page.test.ts \
    rhwp-studio/tests/page-scroll-step.test.ts
tests 99, pass 99, fail 0

$ node scripts/rust-test-suite-manifest.mjs --prepare
32 harnesses, 9 exceptions 생성·확인 완료

$ cargo fmt --all
exit 0

$ cargo fmt --all -- --check
exit 0

$ git diff --check
exit 0
```

빌드의 500kB 초과 chunk 메시지는 기존 Vite 경고이며 빌드는 성공했다.

## 실제 브라우저 검증

macOS Codex in-app browser 1280×720에서 `samples/exam_kor.hwp` 20쪽을 URL 로드해 검증했다.

| 조건 | 관측 | 판정 |
| --- | --- | --- |
| 10% 자동, 어두운 테마 | 10열×2행, 가로 5.85px·세로 5.94px, focus 1쪽 눈금 경계 일치 | 통과 |
| 10% 자동, 밝은 테마 | 회색 작업 영역에서 각 페이지 1px 경계와 6px gap 식별 | 통과 |
| 10% 두 쪽 | 두 열과 다음 행 모두 약 6px 분리 | 통과 |
| 10% 여러 쪽 3×2 | 지정 열·행의 양축 gap 5.81~6.00px | 통과 |
| 10% 가로 이동 | 모든 쪽이 한 행, 인접 gap 5.85px, Y 정렬 동일 | 통과 |
| 100% 단일 열 | page CSS 폭 1123px, 세로 gap 9.90px | 통과 |
| 대표 10/20/25/50/100/500% | 단위 테스트에서 숫자·세부 눈금 최소 화면 간격과 gap 연속성 확인 | 통과 |

## 범위 밖

- 반응형 너비에서 눈금자를 숨기는 정책과 창 확대 중 깜빡임
- [#6040](https://github.com/edwardkim/rhwp/issues/6040) 줌 애니메이션·Canvas 토폴로지 성능
- [#6041](https://github.com/edwardkim/rhwp/issues/6041) 배율별 물리 픽셀 예산과 해상도 최적화
- [#6108](https://github.com/edwardkim/rhwp/issues/6108) 쪽 배치별 맞춤 배율 계산

## 작업 상태

로컬 구현·전체 Studio 회귀·프로덕션 빌드·실제 브라우저 검증을 완료했다. 원격 push와 PR 생성은
별도 사용자 승인 전에는 수행하지 않는다.
