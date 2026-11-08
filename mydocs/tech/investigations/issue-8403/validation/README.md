---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-8403/validation/README.md
last_verified: 2026-10-31
---

# #8403 시각 검증과 회귀 등록 근거

## 최신 devel 통합 후 최종 근거

기준 `99a176b7cb`, Native `21a4902f14`, 최적화 WASM `b13334f327`에서 재검증했다.
두 생산 코드의 차이는 테스트 helper 위치뿐이다. 최종 WASM SHA-256은
`b9a7ebc5875dee4cd2d068a2e3bf3283b8beede8fd8f26c24eca0cb0d1f78d78`다.
최종 자료는 [rebase/](rebase/)이며 아래 이전 결과와 구분한다.

| 입력/동작 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 비공개 원본 및 HWP/HWPX 재열기 | 각각 8 | 94.70% | 94.77% |
| 공개 패턴 및 HWP/HWPX 재열기 | 각각 5 | 99.98% | 99.98% |
| 공개 문자 획·14곳 편집 및 HWP/HWPX 재열기 | 각각 7 | 100.00% | 100.00% |
| 기존 PDF 대조군 직접 출력 | 12 | 99.20% | 98.85% |
| 기존 PDF 대조군 HWP/HWPX 재열기 | 각각 12 | 99.20% | 98.79% |

공개 CanvasKit 12쪽도 패턴 최저 99.98%, 문자 100.00%다. 실제 Studio 화면의
Canvas2D/CanvasKit PNG와 backend 판정은 `rebase/studio-screen/`에 보존했다.
원본은 Arial·Droid Sans Fallback 리소스 미준비로 Canvas2D fallback되어
원본 CanvasKit 성공으로 집계하지 않는다. 비공개 원본 화면은 공개하지 않는다.

지원되는 host font provider로 실제 글꼴을 공급하고 Studio bridge의 Canvas 경로로
출력했다. raw WASM Canvas를 직접 호출해 host 글꼴 정책을 우회한 초기 결과는
최종 판정에서 제외했다. 원본 글꼴은 `original-font-manifest.json`, 기존 대조군은
`local-font-manifest.json`으로 지정한다. 각 항목은 `path`, `aliases`, 선택적인
`weight`이며, 동일 파일은 browser FontFace와 host provider 양쪽에 공급한다.
큰 글꼴은 파일별로 전송한다. 빈 VTT 테이블의 기존 OTS 정규화 내역과 원본/처리 후
SHA는 `rebase/candidate/original-font-evidence.json`에 있다. 글꼴 바이트는 게시하지 않는다.

[검증한 HWP/HWPX 6개와 해시](inputs/README.md)를 첨부했다. Native에서 저장한
동일 바이트를 fresh WASM으로 다시 열어 38쪽을 비교했고 최저 99.98%였다.
검증한 제품 코드 이후에는 문서·증적·검증 하네스만 갱신한다.

## 리베이스 전 검증 이력

다음 수치와 이 디렉터리 루트의 TSV/PNG는 최초 검증 이력이다. 최종 제출의 새 기준
검증은 위 `rebase/` 근거를 사용한다.

2026-10-31. 제품 구현 `6d95c45f08`, base `3c01421e89a562de4ba8b021a4cdf943af60b58d`.
최적화 fresh WASM의 SHA-256은
`17942ab2a5243aa9be2aea558c7449519b3607ca20368c1c94eee41cb1c52f69`다.
시각·저장·편집·실패 후 복구 및 로컬 자동 검증을 완료했다. 전체 회귀의 기존 SVG reader
4건은 기대값을 유지해 보정한 뒤 해당 모듈을 재검증했다. [완료 보고서](../../../../report/task_m100_8403_report.md)와
`local-validation-final.json`에 최초 전체 실행과 보정 뒤 결과를 구분했다.

## 전쪽 결과

96dpi print profile, `visual_sweep.py --silhouette-only --png-pair`의 색상 경계 보정 전
2px `silhouette_raw_match_percent`다. 평균으로 미달 쪽을 상쇄하지 않는다.
독립 기준은 원본 PDF 자체의 Poppler, 공개 패턴의 MuPDF, 문자 PDF의 Poppler 출력이다.

| 입력/동작 | 전체 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 원본 PDF 및 HWP/HWPX 재열기 | 각각 8 | 94.70% | 94.60% |
| 이미지 패턴 및 HWP/HWPX 재열기 | 각각 5 | 99.98% | 99.98% |
| 문자 획 및 HWP/HWPX 재열기 | 각각 7 | 100.00% | 100.00% |
| 14곳 글자 치환 후 HWP/HWPX 재열기 | 각각 7 | 100.00% | 100.00% |
| 기존 PDF 대조군 직접 출력 | 12 | 99.20% | 98.75% |
| 기존 PDF 대조군 HWP/HWPX 재열기 | 각각 12 | 99.20% | 98.66% |

추가로 기존 #683 그림 간격 검사의 관측 경로를 확인하기 위해 `samples/pr-149.hwp` 전 1쪽을
독립 `pdf/pr-149-2022.pdf`와 대조했다. Native 99.98%, WASM 99.96%이며
`pr149-*.tsv`와 `png/pr149/`에 기록했다. 이는 기본 239개에 더한 2개 페이지 비교다.

실제 CanvasKit도 패턴 5쪽 최저 99.98%, 문자 7쪽 전쪽 100.00%다. Canvas2D를 대신한
결과가 아니다. 모든 경로의 전체 쪽수와 저장 전후 페이지별 텍스트 해시도 확인했다.
원본 8쪽의 내용·이미지는 공개 증거에 포함하지 않는다.

기존 PDF 대조군은 `pdf/pr7704/6778-square-2020.pdf`다. Native 직접 출력과 HWP 변환은
전체 12쪽이며, 전체 CLI HWPX 변환은 standard 누적 decoded 작업 한도에 도달했다.
상한을 바꾸지 않고 HWPX 및 WASM Worker에서 `sourcePages`로 1–6쪽·7–12쪽을 각각
가져와 전쪽을 비교했다. WASM의 `legacy-a`/`legacy-b` TSV는 각각 원본 1–6쪽/7–12쪽이다.
전체 HWPX 변환 성공이나 이 제한이 base에서도 같았다는 주장은 하지 않는다.

## 직접 판독과 색상 검사

원본 1·3·8쪽의 제목·로고·그래프, 공개 대조군의 타일 위상·Form·SMask·외부 alpha와
정상 단색 paint를 직접 대조했다. 문자 획은 채움+획·획만 표시·색/alpha·round join·
양자화한 두 축의 폭·hairline·중첩 Form·글자 치환을 확인했다. 글꼴 weight로 획을 대체하지
않으며 원본의 `2 Tr / 1.715 w`와 실제 내장 OpenType family를 보존한다.

패턴 3·5쪽은 저장 뒤 경계 타일을 다시 확인했다. 3쪽의 오른쪽 반쪽 타일은 2×2 원본 중
왼쪽 빨강/파랑 열만 선택해야 한다. 기존 1픽셀 무시 조건에서는 녹색/노란색 열까지 압축돼
보였지만 외곽 실루엣은 100.00%였다. 최종 코드는 선택 창을 그대로 사용한다. Native·
Canvas2D·CanvasKit·HWP/HWPX 재열기에서 실제 색상도 검사했다. 화면의 절대 위치를
고정하지 않고 SVG가 소유한 반쪽 타일의 내부에서 원본 색상 선택 관계를 검사한다.

실루엣 100.00%는 색상·안티앨리어싱까지 바이트 동일하다는 뜻이 아니다. 원본 내장 글꼴과
설치 family의 버전·농도 차이는 남는다. 폭 0 hairline도 Native/브라우저와 Poppler의
농도가 다르지만 획만 표시하는 의미와 내용은 유지된다. MuPDF의 hairline 생략은
ISO 32000-1 §8.4.3.2와 Poppler로 교차 확인했고 기준 PDF를 제품 출력에 맞추지 않았다.

원본은 Nanum 계열 로컬 글꼴, 공개 문자는 저장소 OFL Noto Sans KR을 사용했다.
기존 PDF 대조군은 로컬 바탕·굴림·함초롬·HYHeadLine 계열을 명시했다. 글꼴 바이트는
공개하지 않으며 이름·해시만 기록한다. 해당 대조군의 남는 Type3/대체 글꼴 표현까지
완전히 일치한다고 주장하지 않는다.

## 실행과 증거

`*.tsv`는 전쪽 원문이고 `final-measurements.json`은 집계다. `png/`는 공개 합성 문서의
독립 기준 및 실제 출력이다. `final-source-sha.txt`, `*-final-sha256.txt`, `served-assets.json`에
제품·Native·WASM과 실제 HTTP로 받은 JS/WASM의 신원을 기록했다. `browser-verify-result.json`은
열기·저장·14곳 편집·11개 거절 입력 뒤 정상 복구 결과다. 페이지 텍스트는 본문 전용
`getPageText`가 아니라 글상자까지 포함하는 `getPageTextLayout().runs`에서 수집했다.
텍스트 대조군의 빈 결과를 거절하고 저장 전후 내용·순서를 대조했으며, 원본은 두 형식에서
모두 2,987자를 보존했다. `browser-text-verification.json`에 수정한 검사와 동일 출력 117쪽의
PNG 해시 대조를 기록했다. Native 거절 결과는
`native-negative-final.json`, 공개 생성기 재현 결과는 `fixture-reproduction-final.json`이다.

`*-first-sha256.txt`, `native-fixture-sha256.txt`, `native-negative-results.json`은 초기 구현의
이력이다. 최초 원본 1쪽 88.11%, 첫 WASM HWP 패턴 98.91%, 이후 발견한 색상 crop 오류는
[작업 기록](../../../../working/task_m100_8403_visual_gate.md)에 구분했다. 최종 판정에 초기
산출물의 통과를 대신 사용하지 않는다.

최적화한 fresh WASM을 Studio에 적용한 뒤 실행한다.

```bash
cd rhwp-studio
node e2e/run-with-vite.mjs -- node ../mydocs/tech/investigations/issue-8403/validation/browser-verify.mjs --mode=headless
```

출력은 기본 `output/issue8403-browser/`다. `RHWP_8403_OUTPUT`으로 경로를 지정할 수 있다.
원본 보유자는 `RHWP_8403_ORIGINAL`에 원본 PDF 경로를 지정하면 8쪽과 두 저장 형식
재열기를 함께 실행한다. 기존 대조군의 로컬 글꼴을 명시할 때는 출력 디렉터리의
`local-font-manifest.json`에 `[{"path":"/local/font.ttf","aliases":["Family"]}]` 형태로
설정한다. 파일을 외부로 게시하지 않는다. 이 설정 없이 실행한 글꼴 환경과 위 명시 환경의
시각 결과를 같은 것으로 취급하지 않는다.

재현 PDF·생성기·OFL 출처는 [PDF 디렉터리](../../../../../pdf/issue8403/README.md)에 있다.
HWP/HWPX의 정밀 문자 paint는 제한된 RHWP 확장으로 보존한다. 한컴 등 다른 편집기의
확장 해석은 보장하지 않으며 저장의 `MetadataReduced` 진단과 사용자 가이드로 알린다.
