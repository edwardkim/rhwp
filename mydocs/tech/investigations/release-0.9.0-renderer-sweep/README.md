# 0.9.0 렌더 검증의 본문 clip 보정 증거

[조사·승인·구현 기록](../../../working/release_0_9_0_renderer_sweep_findings.md).
현재 source `8118ccb14d092f767cd1caf1aaece4753a8e61d8`의 검증 자료다.
Native·fresh WASM 시각 검증과 필수 로컬 검증을 모두 완료했다. 원격 PR·CI·merge는 아직 실행하지 않았다.

## 입력과 출력 신원

- [수정 전 Native·CI WASM 빌드 출처](base-build-provenance.json): 최신 비교 base까지 제품 소스가
  같음을 Git diff로 확인했다. 이전 Native는 실제 로컬 `.crate` 설치 검증의 실행 파일이다.
- [입력 provenance](input-provenance.json): `samples/basic/BlogForm_BookReview.hwp`와
  public 표본의 바이트 일치, 독립 `pdf/BlogForm_BookReview-2020.pdf`의 Git blob·SHA-256.
- [최적화 WASM](wasm-build-manifest.json), [실제 초기화 응답 신원](wasm-consumption.json): Rust 1.93.1, Docker, release·wasm-opt 성공.
  검증 당시 실제 초기화 응답과 pkg/public의 JS·WASM 해시가 일치했다. 생성된 public JS는
  제출 변경에서 제외했으며, 증거는 검증 당시의 빌드를 가리킨다.
- 한글 2022 호환 세대(독립 2020 Print 대응), 96dpi, `/home/edward/mygithub/ttfs`를 명시했다.
  사용 글꼴은 HCR Batang·HCR Dotum이며 이 자료에 글꼴 프로그램을 포함하지 않았다.
- [Native 전쪽 TSV](native-silhouette.tsv)와 [fresh WASM 전쪽 TSV](wasm-silhouette.tsv): 각각 1/1쪽 raw 100%.
  [독립 Print](reference-print.png), [WASM Canvas2D Print](wasm-print.png), [Native Skia Print](native-print.png).
- [Native 빌드 신원](native-build-manifest.json), [실제 PNG export·글꼴 선택](native-print-export.json).
  Native screen/print도 수정 전후 PNG 바이트가 같으며, 이 동일성은 독립 PDF와의 전체 픽셀 일치를 뜻하지 않는다.

## Legacy/Layer 경계

- 원 11개 표본의 [수정 전](canvas-before-results.json)·[수정 후](canvas-results.json).
  기존 RGBA 임계치 0.05%를 변경하지 않았다. 각 표본의 1쪽을 검사하는 이 도구의 범위를
  문서 전체의 독립 PDF 검증으로 확대 해석하지 않는다.
- BlogForm은 570/440,960(0.12926%) → 0픽셀이다.
  [Legacy 수정 전](legacy-before.png)·[수정 후](legacy-after.png)·[Layer 수정 후](layer-after.png),
  [Studio 실제 화면](studio-blogform.png).
- [표식 다섯 조건의 실제 Canvas clip](actual-clip-marks.json): 숨김·문단 부호·조판 부호·둘 다·
  다시 숨김. 공통 경계와 기존 48px 표식 여백을 보존했다. 표시 glyph의 19/25픽셀 차이를
  별도로 기록하고 전체 이미지가 항상 동일하다고 주장하지 않는다.
- [Layer 화면·Print와 Native screen/print의 수정 전후 바이트 동일성](unchanged-layer-output.json).

## 직접 판독

[우단·행 경계·하단 모서리 4배 확대](right-border-zoom.png)는 원본 PNG를 보간하지 않고
같은 픽셀 영역으로 확대했다. 독립 Print와 Screen의 색·래스터 차이는 profile 차이로 구분했다.
표의 선언 폭·우단·행 분할·아래 모서리 연결을 유지하고, 수정 뒤 두 Screen 경로의 clip이
일치한다. Print의 비인쇄 입력 안내 제외와 문자·표 앞뒤 순서는 전쪽 이미지에서도 확인했다.

## 검증의 구분

독립 PDF 실루엣(raw 100%)과 두 Canvas의 RGBA 차이율(0%)은 서로 다른 지표다.
합성 helper 계산값만 검사하지 않고 실제 문서·Canvas 호출·화면·독립 Print를 대조했다.
기존 #6590→#7059·#6269 회귀와 정식 11개 Canvas 검사를 재사용하며 기대값·fixture·
임계치를 변경하지 않는다. 새 비시각 HTTP 회귀는 Studio의 기존 자동 수집에서 실행된다.

## 정상 대조군과 제출 경계

- [표식 문서의 screen/print 6개 capture](marks-profile-control.json): report-only parity 4개 통과.
  readiness gate 0개 평가이며 전체 representative matrix 완료를 뜻하지 않는다.
- [최소 글꼴 환경](minimal-font-environment.json): Rust 회귀에 저장소 공개 글꼴 5개만 노출했다.
- [최신 devel의 실제 병합 결과](latest-base-merge.json): 충돌 없이 운영 문서만 추가되며 제품 코드는 같다.
- [확대 이미지 생성 명령](make-zoom.mjs.txt): 원본 PNG 영역을 같은 배율로 보간 없이 표시했다.

확대 그림은 저장된 원본 PNG만으로 다시 만들 수 있다. 저장소 루트에서 아래 명령으로
재생성했으며 기록된 PNG와 바이트가 같았다. 출력은 Git에서 제외된 `output/`에 쓴다.

```bash
VISUAL_SWEEP_CHROME=/path/to/verified/chrome node --input-type=module \
  < mydocs/tech/investigations/release-0.9.0-renderer-sweep/make-zoom.mjs.txt
```

## 필수 로컬 검증 완료

2026-11-08 KST에 아래 항목을 실제 실행했다. 제품 소스는 위 SHA로 고정했으며,
이후 변경은 이 검증 자료와 작업 기록뿐이다.

| 검증 | 실제 결과 |
| --- | --- |
| fmt·root/WASM32/workspace all-target Clippy·workspace build·suite 계약 | 모두 통과 |
| 집중 회귀 | 9건 통과 |
| 전체 release-test nextest | 12,997건 통과, 기존 skip 45건 |
| Native 공식 3종 | 4,077건 통과(라이브러리 4,040·누락 그림 2·직접 PDF 35), 라이브러리 기존 ignored 12건 |
| Studio | 2,778건 통과, 기존 skip 2건 |
| TypeScript·production build | 통과 |
| 최적화 WASM·실제 Canvas 비교 | 빌드 통과, 기존 11개 표본 모두 통과 |
| 독립 Print 전쪽 비교 | Native/fresh WASM 각각 1/1쪽 raw 100%, 쪽수 일치·직접 판독 완료 |

[Rust 실행 명령·실제 요약·로그 해시](rust-validation.json),
[전체 회귀 패키지별 건수](full-regression-counts.json),
[Studio 실제 요약](studio-validation.json).
`focused` 및 두 Native 모듈의 나머지 테스트는 의도한 필터 제외이며,
전체 회귀의 skip 45건과 구분한다.
