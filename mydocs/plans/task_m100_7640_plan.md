# #7640 재저장 수학 문서의 글꼴 프로그램·미주 흐름 보정

Issue: [#7640](https://github.com/edwardkim/rhwp/issues/7640)

## 범위와 선행 근거

- 사용자 지시에 따라 #7375 다음의 오래된 이슈를 같은 `task/7375-rectangle-corners` 브랜치에서
  보정한다. 부분 개선을 완료로 분류하거나 #7375 단독 PR로 중단하지 않는다.
- 입력은 `tests/fixtures/pr7600-equation-font-frame/nov-original-resaved-2020.hwp`, SHA-256
  `1392d5dab6350d232f39a06aa813f1797aec399e41a9e632ffd93e421916f72b`다.
  독립 한컴 2020 Print는 `pdf/pr7600-equation-font-frame-diagnostic/nov-original-resaved-print-2020.pdf`,
  SHA-256 `9c737bf4a8700c5e31d0eb05bba16ea8e6bad4a3bfed7c3bf88334053abcbd79`다.
- 원 PR #7600의 실제 병합은 `22ddc2ab81c405b0c3e9b1f25a397a87de109948`이며, 두 머리 화살표·
  dyad 보정과 이 문서의 전쪽 배치 완료를 구분한다. 인접 #7642의 원본과 재저장본도 섞지 않는다.
- 새 base `ece9c3383b`를 반영한 Native에서 전 21쪽 중 8·11·12·18·19·20·21쪽이 미달이다.
  원본·독립 PDF와 쪽수는 모두 21이다. 기존 7쪽 미달을 검사를 완화하거나 이관해 숨기지 않는다.

## 구현 전 확인한 원인과 소비 경로

- 8쪽의 원문과 render tree에는 정상 한글이 있다. Native 글꼴 추적은 본문 문단 349의
  `한컴돋움`을 실제 설치된 시스템 face로 선택했지만, 최종 그림에서는 다른 글자가 표시됐다.
- 현재 Mac의 `HDOTUM.TTF` 두 버전은 Unicode→glyph ID가 서로 다르다. 두 프로그램을 각각
  직접 열거나 시스템 face에서 얻은 바이트를 다시 열면 정상 한글이며, 시스템 face를 그대로
  그릴 때만 다른 글자가 나온다. Unicode cmap 자체의 오매핑 가설은 독립 CoreText/Skia
  대조에서 재현되지 않았다. 개인 글꼴 파일·설치를 수정하지 않는다.
- 공통 경로는 Native 후보 선택 → `text_replay::fonts::for_text` → 실제 Font 생성 →
  nominal/shaping/합성 굵기·윤곽 paint다. 시스템 조회의 논리적 face를 실제 OpenType 프로그램
  바이트에 묶인 face로 준비한 뒤 같은 Font를 소비하도록 보정한다. family·문서·쪽 번호로
  분기하지 않으며 기존 후보 우선순위·명시 공급·줄 폭·기준선·페이지 예산을 바꾸지 않는다.
- 개인 폰트에 의존하지 않는 프로그램·스타일·glyph 소유 관계 검사를 보강하고 실제 결함의
  수정 전/후 출력도 따로 남긴다. 캐시와 프로그램 준비의 크기·개수 상한, collection index와
  variation 좌표를 보존하며 준비할 수 없는 face의 기존 fallback을 유지한다.
- 미주의 단·쪽 내용 차이는 별도로 남아 있다. 첫 종료 컷·저장 줄·TAC 수식 상자·실제 점유를
  독립 Print와 대조한다. 글자 표시 한 곳을 고친 사실로 전쪽 완료를 주장하지 않는다.

## 검증과 다음 단계

- 진단·빌드·전쪽 PNG/TSV·로그는 `output/open-issues-oldest-20261105/`에 보존한다.
  새 base는 후보와 별도 Cargo target에서 검증하며 source snapshot의 포함 파일은 base blob과 대조한다.
- Native/fresh WASM 전 21쪽·쪽수·소속·누락·중복·대표 review/overlay를 확인하고 미달을 계속 보정한다.
  원본 수학 문서와 #7375의 50쪽도 변경 영향의 정상 대조군으로 사용한다.
- 새 devel #8640·#8637의 실제 기능 보존과 필요한 Rust·Studio·최소 글꼴·편람 검증을 유지한다.
  새 PR 제출은 사용자 승인을 받으며, 원격 이슈 종료는 실제 해결·통합 뒤에만 처리한다.

## Native 프로그램 보정 검증 (2026-11-05)

- 시스템 face를 실제 바이트 프로그램으로 열어 조회·paint의 glyph 소유를 일치시켰다.
  명시 공급·번들 선택과 줄 배치 계약은 유지했다. 준비 캐시는 thread별 개수·바이트 상한을 둔다.
- 보정 전 새 관계 검사 1 PASS/1 FAIL → 보정 후 2 PASS. 새 검사는 기존
  `skia_native_glyph_replay`에 통합했다. CI 변경 없이 원래 Native 명령으로 12 PASS이며,
  최소 글꼴 Mac에서도 12 PASS·OS 프로그램 4건을 확인했다.
- 전 21쪽 출력·쪽수 일치·누락 0을 확인했으나 기존 7쪽 미달은 남는다. 정식 단계 증적은
  [Native 프로그램 검증](../pr/assets/issue_7640/verification-20261105/native-program/README.md)이다.
  이전 전체 12,634 PASS는 이 소스의 전체 통과 증거로 재사용하지 않는다.
- 미주 첫 배치 차이는 문단 514·516·523에서 각 약 3px 전진이 늘어나 534의 수식이 다음 단으로
  밀리는 것으로 좁혔다. 원본/재저장 모두 세 문단의 저장 LH1125·baseline878은 같지만,
  원본 수식은 2400×1125/baseline78, 재저장은 2775×900/baseline86이다.
  현재 수식 줄 재발행은 앞쪽 원본의 높이만 909로 바꾼다. 독립 Print는 전쪽 RGB가 같다는
  기존 근거와 이 변환·줄 점유의 소비 차이를 대조하며 보정한다. 임의 9px 감산은 하지 않는다.
