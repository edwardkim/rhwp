# #6852 결과보고서 — 일반 사각형의 명시적 실선 보존

- Issue: [#6852](https://github.com/edwardkim/rhwp/issues/6852).
- 작성일: 2026-09-08.
- 상태: **로컬 최종 검증 완료, 결과보고 승인 요청. 원격 통합 전.**
- 검증 후보: `393401e5a`; 제품 변경은 `232edb22b`, 집중 시험 후보는 `9f4f451b7`과 같다.
- 계획: [수행계획](../plans/task_m100_6852.md).
- 증거: [Stage 2](../working/task_m100_6852_stage2.md), [Stage 3](../working/task_m100_6852_stage3.md).

## 1. 결론과 수용 범위

원본 5쪽 ‘기존 선발 방식의 문제점’ 앞 작은 묶음에서 앞쪽 흰 사각형의 검정 실선을 복원했다.
메인테이너가 SVG 시각 판정을 통과시켰다. 뒤쪽 그림자 사각형, 기하·그리기 순서는 유지한다.

원인은 #1681에서 도입된 경험적 보정이 글상자가 없는 일반 흰 사각형까지 마스크로 취급해
원본에 명시된 선을 제거한 것이다. 해당 B 조건을 제거하고 글상자가 없는 도형을 보정 경로에서 제외했다.
명시적 ‘선 없음’을 실선으로 강제하지 않는다.

최초 API/UI 진단에 따른 확장 구현은 실선 렌더링 누락을 해결하지 못했으므로 baseline에서 재착수했다.
앞선 코드는 보존 브랜치에 남겼으며 이번 통합에 포함하지 않는다. 그룹 속성 집계·일괄 편집·
혼합값 UI를 해결했다고 주장하지 않는다.

## 2. 실제 입력과 출력 확인

| 입력 | SHA-256 |
| --- | --- |
| `samples/issue6797/156160455-social-pig-farm-income.hwp` | `1b99b763aac36a14a9f463e35ee894a23eb1083780040eab5e0f02a481c694b8` |
| `samples/hwpx/156160455-social-pig-farm-income.hwpx` | `3194188fb93047684c560f346d13015f31bf77a4d1ca02eed6ba3a19f0fa9f53` |

- 실제 HWP/HWPX 모두 11쪽을 유지한다. 5·6·7·8쪽 일반 흰 사각형 총 7개에 검정 stroke가 복원됐다.
- 추가 stroke 속성을 제외한 나머지 SVG는 수정 전과 바이트 동일했다. 7쪽 전체 SVG가 같다는 뜻은 아니다.
- 원래 편람 HWPX는 현재 기준 382쪽이며, 최소 수정 전후 전체 SVG가 동일했다.
  이 결과를 남아 있는 A 글상자 보정의 일반적 정당성 증거로 사용하지 않는다.
- 메인테이너가 확인한 원본 5쪽 SVG:
  `output/6852/restart/after-hwp/156160455-social-pig-farm-income_005.svg`.
  HWP/HWPX의 해당 5쪽 SVG SHA-256은 모두
  `81312df7ceace805dd48154b79d55791f4c506f026d3858befbc7db5ed875e9c`다.

## 3. 검증 결과

최신 원격 devel `7138fe784`를 fetch하여 작업 브랜치가 이미 포함함을 확인했고 merge-tree는 clean이었다.
기존 review worktree와 고정 Cargo target를 재사용했다. Cargo 명령은 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| 실제 원본 RED → GREEN 및 집중 회귀 | 수정 전 실제 HWP/HWPX 2건 실패 → 수정 후 6건 통과 |
| fmt, native/WASM32/workspace all-target Clippy, workspace build | 모두 통과 |
| integration prepare·manifest | 통과, 파생 파일은 제출 제외 |
| release-test 전체 nextest (원장 정정 후) | **9,224 PASS / 0 FAIL / 46 skipped**, test 323.175초, compile 포함 약 565초 |
| 신규 HWPX 보안 검사 | 실제 파일 1개를 명시적 JSON 입력으로 전달하여 통과 |
| Native Skia lib | workspace 합계 4,112 PASS / 0 FAIL / 13 ignored |
| Native Skia 그림 placeholder | 2 PASS; 필터 제외 169건은 실행 통과로 세지 않음 |
| Native Skia 직접 PDF | 4 PASS; 필터 제외 167건은 실행 통과로 세지 않음 |
| Docker 최적화 WASM·HTTP 확인 | 최적화 포함 빌드 exit 0, 실제 WASM HTTP 제공 파일 hash 일치 |

최종 WASM SHA-256은
`f64fdc47e0847cde8adb2b7044c22ab281ab3dd6bade9411e8f405f9b8423bd0`이다.
새 WASM에서 두 원본 모두 11쪽을 유지하며, 1·5·7쪽 SVG 6개가 기존 검증 CLI와 바이트 동일하다.
5쪽은 메인테이너가 수용한 SVG hash와 같다. 기존 Studio `http://localhost:7700/`는 유지했고,
실제 Vite WASM 제공 경로의 응답도 새 산출물과 hash가 같다. 브라우저의 기존 인스턴스는 새로고침해야 한다.

### 신규 fixture의 문단 ID 원장

1차 전체 회귀에서는 신규 HWPX의 `raw_header_extra` 네 경로가 등록되지 않아 IR 왕복 검사 1건이 실패했다.
수정 전 `ee794278f` 제품을 별도 대조 빌드했고, 동일 원본의 전후 왕복을 전수 비교했다.
네 경로별 값은 **1 / 403 / 10 / 212**, 총 **626**으로 수정 전후 상세 값까지 모두 동일했다.
모든 차이는 문단 ID를 담는 6..10 바이트 영역이었으며 두 번째 왕복은 차이가 0이었다.

원인은 기존 HWPX serializer의 문서 전역 문단 ID 재부여다. 신규 fixture에 대한 해당 네 행만
`tests/fixtures/ir_field_sweep_baseline.tsv`에 등록했다. 다른 sample의 기준을 완화하지 않았고
등록 후 전체 회귀를 다시 실행해 통과했다. 상세 경로·값·소스 근거는 Stage 3에 기록했다.
이 등록은 기존 문단 ID 정책이 모든 외부 소비자에게 무해하다고 인증하는 것이 아니다.

## 4. 한계와 분리한 후속 작업

- IR에서 글상자 구조를 내용과 독립적으로 보존하는 개선과 A 조건 재검토는
  [#6856](https://github.com/edwardkim/rhwp/issues/6856)에 등록했다. 이번 구현의 추가 조건이 아니다.
- clipping controlset 92건의 원본은 이 환경에 0건 존재하며 신규 sample도 해당 목록에 없다.
  clipping gate를 통과했다고 보고하지 않는다. 신규 기준 PDF는 추가하지 않았다.
- nextest 0.9.137은 최소 요구 버전을 충족하지만 권장 0.9.140보다 낮고 JUnit `report-skipped`
  옵션 경고가 있었다. 실제 테스트 결과와 구분하며 도구 버전을 변경하지 않았다.
- 시각 수용은 원본 5쪽의 대상 선 복원에 대한 메인테이너 판정이다. 전체 문서의 한컴 조판 완전 일치를
  선언하지 않는다. 독립 픽셀 점수를 새로 측정하지 않았으므로 임의 점수를 기재하지 않는다.

## 5. 남은 통합 절차

이 보고서의 결과 승인을 받는다. GitHub 본문은 아직 최초 API/UI 범위이므로
실제 수용 범위와 #6856 분리를 반영한 정정 초안을 준비했다. 원격 게시는 아직 하지 않았다.

그다음 최신 devel 재확인·충돌 확인 → 승인된 이슈 본문 정정·push·PR 생성 → CI 완료 →
self-review → 승인된 merge commit 병합 → 실제 devel 포함 확인과 #6852 close 순서다.
IR 확장을 재편입하지 않으며 필요한 대표 시각 증적의 장기 보존은 PR 기록 절차를 따른다.
