# #7902 6단계 — Visual Sweep의 PNG alpha 판독 교정

Issue: #7902. 이 단계는 비교 도구의 투명도 처리만 수정하며 렌더러의19문서 보정·최종 승인과 구분한다.
기준은 `upstream/devel`의 `e4e6d0c8d`(#7935)다. 렌더러 작업은 별도 기존 branch에 보존했다.

## 구현 전 관측과 기대 계약

투명 `(0,0,0,0)`은 흰 종이에서 보이지 않는다. 현재 `Image.convert("RGB")`는 alpha를 버려
검은 픽셀로 읽는다. RGBA PNG를 쓰는 silhouette·overlay·compare·내용 분석이 같은 오류를 공유한다.
실제 진단 PNG의1123번째 투명 행794픽셀이 이 오류의 재현 입력이었다. 위치·DPI·색상 임계값·
2px 반경·페이지 내용은 바꾸지 않고 흰 종이 위의 합성을 RGB 해석 전에 수행해야 한다.

- 입력 생성: 테스트에서 RGBA/LA/팔레트 PNG와 독립 RGB 정답을 직접 만든다.
- 독립 기대: alpha0은 흰색, 검정 alpha128은127 RGB, 불투명 RGB는 원래 바이트 그대로다.
- 호출 경로: PNG 열기/인메모리 입력 → 공통 흰 배경 RGB 결과 → 패딩/내용 마스크/overlay/review.
  `.convert("RGB")`를 먼저 호출해 alpha를 잃는 경로도 함께 제거한다.
- 반례: 불투명 잉크의2px 초과 이동은 계속 검출해야 한다. alpha를 이유로 실제 보이는 내용을 지우지 않는다.
- 범위 밖: 문서 레이아웃/paint/Native/WASM 바이트, 비교 위치·배율·임계값과 원래 페이지 수.

## 검증과 제출 계획

1. 기존 도구에서 올바른 RGBA/팔레트 기대값이 의도한 픽셀 불일치로 실패하는지 확인한다.
2. 공통 helper를 구현하고 기존 Visual Sweep Python 계약과 새 alpha 계약을 실행한다.
3. CLI `--png-pair --silhouette-only` 및 overlay/compare/review가 같은 RGB를 소비하는지 확인한다.
4. 기존 source에서 보존한 실제 Native/WASM PNG 전쪽을 같은 도구로 재평가한다.
   producer·입력/독립 PDF·DPI는 바꾸지 않고 이전 TSV/manifest SHA와 결과를 연결한다.
   이는 원래 source의 판독 비교이며 새 devel의 렌더링 완료로 보고하지 않는다.
5. Python 도구만 바꾸므로 Rust 빌드·전체 회귀를 반복하지 않는다. 실제 문서 렌더링 코드를
   보정한 별도 branch는 그 branch의 필수 검증을 계속 충족해야 한다.
6. 사용자 자동 승인에 따라 PR 직전 head 재확인 → 도구 PR → 정확한 head CI 모니터링 →
   옵션2의 별도 review·오늘할일 문서 PR과 merge·후속 처리를 진행한다. #7902는 닫지 않는다.

## 로컬 검증 결과

- 기존 source에서 최초 alpha 검사7개 중5개가 잘못된 픽셀 판독으로 실패했고2개는 통과했다.
  수정 후에는 alpha 계약11개를 포함한 Visual Sweep Python 검사152개가 모두 통과했다.
- 독립 venv에 `Pillow==12.3.0`, `fonttools==4.64.0`만 설치한 실행도152개 통과(28.098초).
  CLI2작업자 경로, overlay·compare 및 내용 분석을 포함한다.
- 실제 보존 PNG3,340쌍(19문서×835쪽×Native/WASM×base/candidate)의 재평가는406.167초에
  끝났고76개 TSV 모두 기존 바이트와 동일했다. 원래 renderer source는 각각 `2efefc6a1f`와
  `9c1de319b`이며 새 base의 시각 검증으로 승격하지 않는다.
- [합성 입력의 수정 전후 review](../pr/assets/visual_sweep_alpha_20261016/README.md)를 직접 판독했다.
  보이는 표·문구는 유지되고, 기존의 잘못된 하단 검은 띠만 흰 종이로 해석된다.
- `image_decoding`에 helper SHA와 흰 배경 정책을 기록해 resume의 판독 구현도 고정한다.
- 기존 lint job에 동일한 venv/discovery 명령을 연결했다. 별도 job·권한·trigger는 추가하지
  않았으며, 배선 검사로 alpha/worker 모듈의 CI 발견을 확인한다. `actionlint`와 배선4개 통과.
  전체 workflow 계약250개도 통과(30.002초)했다. sparse checkout에 없던 기존 검사 입력을
  포함한 뒤 재실행한 결과이며, 파일 누락 실행은 통과 증거에서 제외했다.
- 변경 Markdown3개의 링크, canonical guide 메타데이터와 `git diff --check`가 통과했다.
  저장소 전체 메타데이터 검사는 기존 누락·sparse 경로 오류가 있어 전체 통과로 보고하지 않는다.

증적은 ignored `output/pr-review/issue7902-render-repair-20261014/`의
`alpha-tool-before.log`, `alpha-tool-clean-tests.log`, `alpha-tool-real-png-rescore-result.json`,
`alpha-tool-asset-provenance.json`에 보존했다. TSV·JSON·로그는 커밋하지 않는다.

조판 원칙 검토: 제품 renderer·배치·글꼴·출력 경로 변경은 비해당이다. 투명도가 있는 비교 입력의
RGB 해석만 교정한다. 실제 내용의 이동·누락은 계속 실패하며, #7902의 별도 렌더러 검증과
미해결 쪽의 PR 보류 조건은 유지한다.
