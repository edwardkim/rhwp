# #6042 / PR #6637 최소 검증 증거

이 폴더와 인접 `issue6042-*` 폴더는 제품 runtime 자산이 아니라 이슈 검증 자료다.
[개발 패널 사용 안내](../../../manual/studio_scroll_probe_guide.md)로 새 측정을 수행할 수 있다.

## 보존 기준

2026-09-04 정리 기준 head는 `9b679f07a8b714d680ed822406e41cc62a6174ea`다. 당시 162개 자산
17,349,049 bytes 중 34개·5,163,080 bytes를 원문 그대로 보존했다. 중간 smoke, 중복 화면, 폐기한
계측 표본과 더 이상 입력을 보존하지 않는 집계기는 제거했다. 별도 영구 archive는 만들지 않았다.

**이 SHA는 정리 기준이지 아래 성능을 새로 측정한 head가 아니다.** 표본은 2026-08-31~09-02에
수집한 단계별 증거다. Stage 3은 LRU가 이미 있는 scheduler 비교 기준선이며, 배포판이나 PR의 전체
base와 같지 않다. rebase·후속 보정을 포함한 최신 exact head의 정량 재측정으로 주장하지 않는다.

선택한 시나리오의 A/B 반복은 전부 보존한다. 유리한 라운드만 추리지 않았으며 지연 증가 사례도
남겼다. 원시의 값·공백·순서까지 바꾸지 않았는지는 [manifest](retained-manifest.json)의 SHA-256으로
확인한다. 더 오래된 조사 경위는 단계 보고서와 historical summary를 따른다.

## 다시 계산할 수 있는 핵심 증거

| 주장/한계 | 보존 입력 | 비교 revision과 표본 수 |
| --- | --- | --- |
| cold 새 구간의 첫 visible 우선 표시 | [178쪽 cold A/B](../issue6042-stage5/hwpspec-4col-34-cold-alternating.json), [당시 요약](../issue6042-stage5/summary.json)의 `hwpspecFourColumns34Cold` | Stage 3 `5f5d60071` → 초기 Stage 4 `6f2d82d24`, 4열·34%, 20쌍. 후속 보정 전 측정이다. |
| warm 재사용·역방향 thrash 보정 | [correction 폴더](../issue6042-stage5-correction/), [요약](../issue6042-stage5-correction/summary.json), [전체 집계기](../issue6042-stage5-correction/summarize.mjs) | `5f5d60071` → `63d29e68b`, exam/hwpspec 4열·34%, 문서·revision당 2블록×20회. `*-a1/a2`, `*-b1/b2` 총 8원시. |
| 두 쪽에서 retained 지연 증가도 공개 | [expanded 폴더](../issue6042-stage5-expanded/), [당시 요약](../issue6042-stage5-expanded/summary.json)의 `performance.canvas2dExamDouble50` | `5f5d60071` → `63d29e68b`, exam 두 쪽·50%, revision당 2블록×20회. `canvas2d-exam-double-50-*.json` 총 4원시. |
| 정착 뒤 클릭 없이 화질 회복과 추가 비용 | [before](../issue6042-stage5-scroll-quality/exam-double-100-before-20.json), [after](../issue6042-stage5-scroll-quality/exam-double-100-after-20.json), [요약](../issue6042-stage5-scroll-quality/summary.json) | `5f5d60071` → `a762e58ea`, exam 두 쪽·100%, revision당 20회. `performance20Rounds` 시간·호출 수를 재계산한다. |

공통 주 환경은 Chromium 151 / Canvas2D / 1280×720 CSS px / 실제 DPR 2다. fixture·WASM·폰트
초기 hash는 [environment.json](environment.json), 각 비교의 revision/배치는 해당 요약과 원시를
따른다. 초기 환경 파일의 source hash를 후속 모든 측정의 source hash로 간주하지 않는다.

화질 요약의 `settledKnownWorkMs`는 trace의 `retainedComplete`다. runner의 추가 안정 프레임까지
기다리는 `samples[].knownWorkNextFrameMs`와 다르다. 전자는 before 332.3/349.4ms → after
365.6/403.5ms, 후자는 332.9/349.9ms → 366.6/407.8ms다. 기존 PR의 +33.3/+54.1ms는 전자의 차이다.

저장소 루트에서 Node.js로 다음 명령을 실행한다. 외부 패키지나 새 브라우저 실행 없이 **저장된
표본을 읽기만** 하며 해시·표본 수·핵심 p50/p95·호출 수·cold long-task 합계를 확인한다.

```bash
node mydocs/working/assets/issue6042/check-retained.mjs
```

correction 전체 통계/ledger/판정의 원 집계기도 입력을 모두 보존했다. 아래 명령은 해당
`summary.json`을 재생성하므로 실행 뒤 diff가 없는지 확인한다.

```bash
node mydocs/working/assets/issue6042-stage5-correction/summarize.mjs
```

## 시각 증거

- 개발 패널 UI 예시: [hwpspec 자동 34%](../issue6042-stage2/hwpspec-auto-34.jpg).
- Canvas2D 배치와 CanvasKit KTX 비교: [이미지 비교 결과](../issue6042-stage5-expanded/visual-comparison.json).
  두 before/after JPEG 쌍과 diff PNG를 남겼다. [비교기](../issue6042-stage5-expanded/compare-visuals.mjs)는
  macOS `sips`와 Studio의 `pngjs`/`pixelmatch` 의존성이 필요하며 diff/JSON을 다시 쓴다. 다른 OS에서
  그대로 실행할 수 있다고 보장하지 않는다. JPEG 차이는 배치·누락 보조 근거이지 glyph 화질 점수가 아니다.
- 클릭 전후 읽기 화질: [DPR 1](../issue6042-stage5-scroll-quality/exam-double-100-before-dpr1.png) /
  [정착 후 DPR 2](../issue6042-stage5-scroll-quality/exam-double-100-after-dpr2.png). quality 요약의
  `qualityAb`, `policyChecks`, `validation`은 당시 별도 관찰/검증 기록이며 시간 원시만으로 전부
  재계산되는 항목이 아니다.

## historical summary와 제거 자료

[Stage 1 요약](summary.json), [Stage 2 소규모 대조](../issue6042-stage2/browser-ab.json),
[초기 Stage 5 요약](../issue6042-stage5/summary.json),
[확장 matrix 요약](../issue6042-stage5-expanded/summary.json)은 당시 판단 이력도 보존한다.
핵심 표에 명시하지 않은 시나리오의 개별 원시는 현재 tree에서 제거했으므로 **이 요약 전체가 현재
파일만으로 재계산된다는 뜻은 아니다.** 초기 `accepted=false`나 미완료 gate도 역사적 사실로 남긴다.

이전 단계 보고서의 삭제 자료 링크는 정리 전 commit을 고정해 남겼다. 필요할 때 그 파일 하나를
열거나 아래처럼 확인할 수 있다. Git 이력 외 별도 장기 저장/다운로드 보장은 하지 않는다.

```bash
git show 9b679f07a8b714d680ed822406e41cc62a6174ea:mydocs/working/assets/issue6042-stage3/hwpspec-178p-warm-scroll.json
```

향후 작업에서는 이 숫자를 절대 성능 기준으로 재사용하지 말고 같은 방법으로 자신의 base/head를
측정한다. 새 PR마다 최종 핵심 표본·환경·방법을 남기고 중간 원시를 무제한 누적하지 않는다.
