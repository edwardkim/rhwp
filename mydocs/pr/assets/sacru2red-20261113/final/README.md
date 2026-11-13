# #7620 메인테이너 간격 보정 — 최종 검증 원장

현재 제품 source는 `e5a6a1ff9f`다. 필수 전체 회귀·Native 공식 검사·정본 보호 전체는 진행 중이며,
이 원장의 집중 시각 통과를 PR 제출 준비 완료로 해석하지 않는다.

## 실제 개선과 독립 기대값

같은 원본 e·f의 Desktop Hancom 2020 `11.0.0.9136` Print(method 0, one-up)를 기준으로
Native Skia와 fresh 최적화 WASM Canvas를 96dpi·compat 2022에서 대조했다.
원본·기준 해시와 출력 경로는 [보정 전 원장](../README.md) 및 `samples/issue7620/MANIFEST.json`에 있다.

| 입력 | 전체 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 |
| --- | --- | --- | --- |
| e-cellbreak | 2 | 100.00% | 100.00% |
| f-rowbreak | 2 | 100.00% | 100.00% |
| footer 대조군 | 13 | 96.17% | 96.11% |

전쪽 TSV는 `tsv/`에 있다. `*-three-way.png`는 독립 Print·Native·fresh WASM을 같은 배율로
배치한 실제 출력이다. 행의 쪽 소속·모든 셀 글줄·표 외곽·뒤 문단·원래 빈 줄을 직접 확인했다.
대상에는 수식이 없으며 글자·줄·표의 실제 간격을 확대 대조했다.

빈 문단의 원본 LineSeg는 높이 1000HU·후행 간격 600HU이며 다음 줄까지 1600HU다.
`spacing-relation-probe.tsv`는 같은 원본의 실제 렌더 트리에서 이 전진 관계가 보정 전에는
Native/WASM 모두 FAIL, 보정 후에는 모두 PASS임을 기록한다. 절대 px 위치를 고정하지 않는다.
정식 Rust 회귀 2건에도 빈 줄·뒤 본문 소속과 원본 전진량 관계를 추가했다.

`devel-comparison.tsv`는 고정 devel `180a4ec972`와 보정 후보의 Native 17쪽·WASM 17쪽을
동일 조건으로 대조한다. 34행 모두 하락이 없고 footer의 양쪽 13쪽 PNG는 각각 보정 전후 바이트가 같다.
새 source binary와 fresh WASM의 정확한 SHA-256은 전체 검사 종료 뒤 명령·결과와 함께 기록한다.

## 공통 경로와 기존 검사

공통 `HeightCursor::vpos_adjust`가 실제 소비한 줄간격을 lazy 원점에서 중복 역산하지 않게 했다.
기존 단 원점이 있으면 그 축의 일치 검사도 유지하며, 실제 줄 하단 누락·불일치, 비연속·합성·편집,
모순되는 기존 원점의 bridge는 유지한다. fit과 paint가 같은 커서와 실제 줄 상자 하단을 소비한다.

기존 `lazy_path_applied_and_base_set`의 원점 부재 case 6은 실제 줄 하단 116px·흐름 124px,
저장 1000+1000+600HU를 주면서 간격을 한 번 더 계상하도록 기대했다. 독립 e·f Print와
실제 전진 관계로 이 전제가 틀림을 확인했다. 기대 상대 기준을 800HU로 보정했고 나머지 부정 대조군은 유지했다.
새 source-side 테스트 모듈이나 정본 수치·허용치를 추가·완화하지 않았다.

## 필수 검사 진행 상태

- Native Clippy·WASM Clippy·workspace build·workspace all-target Clippy: 통과.
- 최소 글꼴 집중 회귀: 8 PASS. 전체 release-test와 Native 공식 3종은 진행 중.
- 격리된 Fontconfig에 노출된 프로그램은 저장소의 NotoSansKR Regular/ExtraLight,
  NotoSansSymbols2, RHWPOldHangul, RHWPPublicBatang 5개다. 시각 검증 글꼴과 분리했다.
- 편람 HWPX Native 383쪽 `--handbook-gate`: 최저 99.96%, 공식 값 하락 0.
- 정본 보호 전체: 진행 중. 최초 편람 HWP 145쪽은 보정 전후 PNG가 같았으며,
  원 보호 기록의 `handbook145-print-observed-20261031` 글꼴 환경을 복원해 100.00%를 확인했다.
- #7905의 Native 표시값 2행은 같은 조건의 보정 전후 PNG가 전 6쪽 모두 같음을 확인했다.
  원 보호 조건과 source·출력 backend를 대조 중이다. 해소 전에는 보호 통과로 판정하지 않는다.

원시 로그·JSON·중복 래스터·독립 글꼴 표본과 로컬 글꼴 프로그램은 루트의 ignored `output/`에만 있다.
