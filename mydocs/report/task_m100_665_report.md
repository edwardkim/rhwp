# #665 — 시험지 ingest 이미지 삽입 결과

Issue: #665

## 결과와 범위

기존 [#665](https://github.com/edwardkim/rhwp/issues/665)를 재개해 이미지 placeholder 대신
실제 Picture/BinData를 만드는 CLI와 byte-only Rust API를 구현했다. 신규 중복 이슈는 만들지 않았다.
2026-11-03 최종 전체 회귀와 Native Skia 검증을 마쳐 로컬 구현·검증을 완료했다.
코드 push·PR 생성·merge·이슈 재종결은 이번 승인 범위에 포함하지 않는다.

- branch: `task/665-ingest-images`
- base: `ea86816b5316dfeeb47ecf20df97abfd4b02da73`
- 실제 Native/fresh WASM producer의 제품 source: `d78048a6f0e5f71aa3bdb7d32e703bd5971e3019`
- 이후 커밋은 검사·문서·검증 자료 보완이며 제품 소스는 같다.
- [수행 계획](../plans/task_m100_665_ingest_images.md),
  [작업 기록](../working/task_m100_665_ingest_images.md),
  [전쪽 측정·판독·provenance](../tech/investigations/issue-665/README.md).

## 사용할 수 있는 동작

`build-from-ingest`가 PNG·JPEG·GIF·BMP·TIFF를 실제로 읽고 검증해 HWPX에 포함한다.
생략한 media root는 JSON 파일의 디렉터리이며 `--media-dir`로 바꿀 수 있다.
above/between/below/inline, 문장 중간의 `paragraph.runs`, 공유 지문·보기 안팎의 그림 소유,
반복 이미지 자원 재사용과 HWP/HWPX 저장 재열기를 확인했다.

```bash
output/issue665/head-rhwp build-from-ingest tests/fixtures/issue665/runs.json \
  -o output/issue665/manual-runs.hwpx --json
```

위 실행 파일은 로컬 검증용 producer다. 일반 배포 binary를 갱신하거나 기존 Studio의 pkg에
이번 WASM을 덮어쓰지 않았다. 보존한 생성 문서는
[`runs.hwpx`](../../tests/fixtures/issue665/generated/runs.hwpx)와
[`mixed.hwpx`](../../tests/fixtures/issue665/generated/mixed.hwpx)에서 확인할 수 있다.

누락·손상·미지원 형식·경로 이탈·잘못된 선언 치수·출력 크기·자원 한도 초과는
`INGEST_IMAGE_*`로 실패하고 기존 출력 파일을 보존한다. CLI의 이미지 한도 5종과
지원 형식·배치·inline 입력 방법을 [CLI 매뉴얼](../manual/cli_commands.md)에 반영했다.
시험지 스킬·참조 문서·예제·생성기도 실제 구현 계약으로 현행화했다.

Rust 소비자는 `IngestImage::from_bytes`, `IngestImageLimits`,
`build_exam_paper_with_images`를 사용한다. 코어는 파일을 읽지 않는다.
기존 `build_exam_paper`의 텍스트 projection은 유지된다. 공개 enum `StemBlock`을
빠짐없이 match하는 코드는 새 `Paragraph { runs }` variant 처리를 추가해야 한다.

## 검증

| 항목 | 결과 |
| --- | --- |
| 신규 그림 구조·저장·오류 계약 | 최소 글꼴 격리에서 9/9 통과 |
| 기존 ingest / HTML 그림 집중 검사 | 14/14, 8/8 통과 |
| 전체 release-test | 12,378/12,378 통과, 기존 설정에 따른 45개 skip |
| Native Skia 공식 3종 | lib 4,040개 + placeholder 2개 + 직접 PDF 22개 통과, lib 기존 ignore 12개 |
| Rust lint | fmt, Native/WASM Clippy, workspace all-target Clippy 통과 |
| test manifest / unit-tier | 고정 base를 전달한 검사 통과 |
| 새 문서 보안 검사 | HWP/HWPX 24개 × CLI 탐지 3종 = 72회 clean |
| CLI 오류·기존 출력 보존 | 14개 오류 경로 통과 |
| 문서·스킬 | 기존 계약 48개 통과, 최종 스킬 본문 변경 후 36개 재확인 통과 |

전체 회귀 첫 실행의 12,378건 중 2건은 검증 경로 보완 뒤 전체를 다시 실행해 통과했다. 하나는
공통 model helper로 이동한 DocInfo 무효화가 기존 소스 스캐너 범위 밖이 된 것이었다.
기존 #2724의 원 PR·실제 통합·독립 삭제 실증을 추적하고, 기존 7곳과 이동한 1곳을
각각 검사하도록 보완했다. 이동한 코드를 삭제한 음성 대조는 실제 실패한다.
다른 하나는 `samples/` 전용 검사에 목적별 `tests/fixtures/` 경로를 전달한 실행 오류였다.
탐지 규칙은 바꾸지 않았으며 새 fixture는 실제 CLI 72회로 별도 확인했다.
최종 명령·종료 코드·개수는 [Rust 검증 기록](../tech/investigations/issue-665/rust-validation.json)에 있다.

## 독립 출력과 성능

한컴 2024 `13.0.0.3901`의 독립 Print와 같은 문서 전체 페이지를 Native/fresh WASM으로
비교했다. 배치 4종·문장 내 그림·혼합 그림·이미지 우선·저장 왕복·JPEG/GIF/BMP/TIFF는
양쪽 100.00%다. shared-boxed의 최저는 Native 91.04%, WASM 91.00%이며 같은 입력의
base/head 값과 실제 SVG가 동일하다. 각 문서는 1쪽이고 쪽수 불일치·누락·90% 미만은 없다.
그림 내용·비율·문자 순서·문항 번호·선택지·공유 지문 1회·보기 안팎 소유를 직접 확대 판독했다.
shared-boxed의 기존 border·간격 차이는 그대로 기록했고 절대 px 핀은 추가하지 않았다.

텍스트 100문항을 교대로 21회씩 생성한 중앙값은 base 5.38ms / head 5.25ms이고,
ZIP 내부 내용이 같다. 측정 범위에서 속도 저하는 관측되지 않았다. 같은 그림 100개와
1,000개 생성 중앙값은 7.05ms / 25.67ms, peak RSS는 15,256KiB / 24,732KiB이며
두 경우 모두 BinData 1개다. 브라우저 로딩이나 다페이지 편집 성능 측정으로 해석하지 않는다.

기준 PDF·입력 이미지·JSON·HWPX/HWP·전쪽 TSV·대표 확대 PNG·producer hash·보안 검사·
성능 원자료는 [정식 검증 자료](../tech/investigations/issue-665/README.md)에 보존한다.

PR 준비 때 동일 입력 15개 시나리오의 Native/fresh WASM 30개 전쪽 비교를 완성했다.
고정한 devel 대비 최대 하락폭은 0.00%p다. 제품·테스트 source는 바꾸지 않았으며
완료한 전체 회귀와 lint를 반복하지 않았다.
