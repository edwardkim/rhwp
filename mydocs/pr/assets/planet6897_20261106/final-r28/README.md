# R28 통합 후보의 실제 전쪽 검증 자료

이 디렉터리는 제품 커밋 `b6c02c1946`과 같은 소스에서 만든 Native 실행 파일과
최적화 fresh WASM의 실제 측정 자료다. [실행 파일과 소스 신원](producer-provenance.json),
[6,020개 소스 파일 해시](source-snapshot.json), [완료·대기 목록](inventory.json)을 구분한다.
목록의 `pending`은 측정 완료나 승인으로 세지 않는다. 최종 로컬 게이트 완료 전에는 PR을 제출하지 않는다.

## 측정과 증거 생성

- 원본·독립 한컴 Print PDF의 전체 페이지를 96 DPI·`print` 프로필로 비교한다.
  입력별 `compat`·기존 승인 글꼴 환경·원본/PDF SHA-256은 각 `provenance.json`에 기록한다.
- Rust 회귀의 최소 글꼴 환경과 달리, 시각 검증에는 전체 시스템 글꼴과 원본별 기존 대체 계약을 사용한다.
  원본·PDF·임계값을 바꾸지 않았으며, 상용 글꼴 프로그램을 이 자료에 포함하지 않는다.
- `native/<입력>/full.tsv`, `wasm/<입력>/full.tsv`는 해당 실행의 전쪽 TSV 원본이다.
  `review-NNN.png`와 `overlay-NNN.png`는 같은 실제 측정 PNG에서 만든 주장 쪽 자료다.
  패널 생성은 엔진 재렌더가 아니며 `page-NNN-source.json`으로 실제 원본 이미지 해시를 확인한다.
- 자료 생성 상태와 직접 확대 판독·최종 승인 상태는 별도다. 원 주장과 다른 입력·쪽으로 대체하지 않는다.
  PR #8642는 보고서 838의 PDF 91쪽 및 보고서 628의 PDF 95·96쪽도 포함한다.
- 기본 검증 대상은 56개 입력/PDF 조합·각 662쪽이다. 별도 보호 문서 17종·각 1,770쪽과
  PR #8389·#8397의 과거 2,153행/backend 무하락 대조는 통합 검증 기록에 따로 연결한다.

## 완료된 소비 경로 검사

최적화 WASM SHA-256은 `63b74db31592ee2b83c562155a4906694209f45eb9416aeb253d2347f67ee0bc`다.
같은 파일을 루트 `pkg/`와 Studio `public/`에 공급해 해시 일치를 확인했다.
Studio 프로덕션 빌드, Node 전체 2,750건, 공개 API·WASM 바인딩 3건이 모두 통과했다.
`wasm-pack 0.15.1`의 이번 빌드에는 버전 조회 경고가 없었다.

[실제 CanvasKit 소비 기록](canvaskit-consumer.json)은 같은 WASM이 만든 보고서 628의
18쪽 수식 좌표를 Chromium의 `CanvasKitLayerRenderer`에서 재생한 결과다.
실제 필요한 글꼴 프로그램 4개를 런타임에 공급했고, layout replay 1건 성공·실패 0건·
누락 글꼴 0건을 확인했다. SVG fallback을 제거한 소비 경로 검사이며 전쪽 Print 비교를 대신하지 않는다.

전체 최소 글꼴 Rust 회귀, Native Skia, lint, 나머지 전쪽 비교, 편람 gate와 개별 리뷰의
최종 판정은 진행 중이다. 위 완료 결과만으로 전체 PR 준비 완료를 주장하지 않는다.
