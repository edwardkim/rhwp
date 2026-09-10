# #6040 재개 작업 결과 — 연속 줌과 페이지 교체 안정화

- Issue: #6040
- 판정일: 2026-09-10. 구현·로컬 검증·사용자 정성 수용 완료, 원격 제출·merge·종료 승인 전.
- 비교 devel: `37bd46a72f9fd9ffd709e35244df79c00e789780` (제출 준비 시 원격과 일치 확인).
- 검증 제품: `931306732b9c59b88063dd54caaf921143410d13`. 제출 자료 정리는 제품·테스트를 바꾸지 않는다.
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md),
  [최소 계측 증거](../working/assets/issue6040-post-stack/final-summary.json),
  [재현 도구 안내](../manual/studio_scroll_probe_guide.md).

## 해결한 문제와 사용자 변화

자동 열 선택·중앙 정렬은 기존 #6458에서 해결했다. 이번 PR은 이미 병합한 #6458·#6467·#6637
stack의 재제출이 아니라, 그 뒤 남은 연속 줌·정착 비용을 다루는 `devel` 대상 일반 PR이다.

| 상황 | 기존 경로의 문제 | 이번 변화 |
| --- | --- | --- |
| 문서를 줄이다 곧바로 다시 확대 | 같은 animation에서 preview 중복, 입력 사이 정착·구 배율 작업 실행 | 중복 preview 제거, quiet/수렴 대기와 generation 취소로 최신 요청 우선 |
| 축소로 여러 쪽이 보이기 시작 | 정착 시 활성 Canvas·layer·cache를 전역 해제하고 visible 동기 렌더 | 기존 화면을 유지하며 빈 visible 우선 페이지별 교체, visible 뒤 offscreen 작업 전에 frame 기회 양보 |
| 커서는 첫 쪽에 두고 다른 쪽 읽기 | 정착 후에도 예산 때문에 흐리게 남아 클릭해야 선명해짐 | 정착한 visible은 편집 focus와 무관하게 raw DPR 요청. 기존 canvas 안전 clamp 유지 |
| 고배율 확대 후 비가시 쪽 보존 | 기존 active는 신규 prefetch admission을 우회해 불필요하게 커짐 | 줌 정착 예산 초과 시 먼 offscreen부터 반환. visible preview·화질·focus는 보존 |

이미 시작한 동기 WASM/native raster는 중간에 중단하지 못한다. 새로 보이는 모든 페이지의 저화질
placeholder를 만드는 구현도 아니다. 아직 한 번도 그리지 않은 페이지는 첫 bitmap 준비까지 비어 있을 수 있다.
단일 authoritative geometry, pointer/중앙 앵커, live 열 전환을 유지하며 제스처 전체 열 동결은 하지 않는다.
geometry 전체 재계산을 없앤 것이 아니라 계측으로 확인한 중복 preview를 제거한 제한적 보정이다.

## 실제 검증과 계측

제품 head에서 `npm --prefix rhwp-studio test`: **1,643 pass / 2 skip / 0 fail**.
`npm --prefix rhwp-studio run build`: TypeScript·Vite·PWA 통과. 자료 정리 후 같은 제출 tree에서
두 명령을 재실행했고 다시 1,643 pass / 2 skip / 0 fail 및 build 성공을 확인했다.
Rust/Cargo·sample·golden·CI workflow 변경은 없어 Rust 전체 회귀를 추가 실행하지 않았다.
실제 Canvas2D/CanvasKit 줌 smoke와 마지막 Canvas2D 읽기 화질을 직접 확인했고, 사용자가 요청한
클릭 없는 읽기·줌 직후 스크롤·재확대 비교에 “문제없어”로 응답했다. 새 Firefox 프로파일 수치는 없다.
한컴 PDF fidelity/visual sweep 통과를 주장하는 변경이 아니며, 실제 UI 검사·자동 계약·사용자 수용을 근거로 한다.

### 소유 surface 비교

Canvas2D, Chromium 152, 1280×720 CSS px, DPR 2, `exam_kor.hwp` 20쪽. 같은 WASM·의존성·
관찰 adapter에서 before/보정 전 후보/최종 후보 각각 2회 실행했다. 각 변형의 checkpoint 값은 반복 간 같았다.

| 시나리오 | devel before | 보정 전 후보 | 최종 후보 (pixel) |
| --- | ---: | ---: | ---: |
| 첫 200% | 114,071,400 | 135,464,550 | 114,071,400 |
| 500% 최고 관찰값 | 268,450,552 | 402,101,695 | 268,450,552 |
| 500→100% | 33,874,172 | 33,874,172 | 28,524,200 |
| 34% | 23,103,360 | 15,677,280 | 15,677,280 |
| 34→100% | 55,267,322 | 55,267,322 | 34,169,202 |
| 비편집 쪽 읽기 200% | 48,125,352 | 221,018,100 | 85,553,550 |
| 재열기 100% | 33,874,172 | 33,874,172 | 28,524,200 |

500%에서 최종 후보가 제거한 133,651,143 pixel은 화면 밖 surface이며 보이는 쪽은 동일하다.
비편집 읽기 200%의 before 대비 **37,428,198 pixel 증가는 의도한 읽기 화질 비용**이다.
고배율 최종 후보가 배포판보다 적은 메모리를 쓴다거나 모든 조건에서 성능이 개선됐다고 주장하지 않는다.

위 수치는 소유 surface의 관찰값이지 GPU/RSS가 아니다. 짧은 native/WASM 임시 할당과 GC 대기 객체를
포괄하지 않으며, 관찰 peak는 하한이다. DOM 열거 계측을 켠 시간값으로 속도 개선율을 계산하지 않았다.
WASM은 Docker daemon 부재로 native locked `--no-opt` 진단 경로를 사용했고 SHA-256은
`6ee336d08591621b5e5da087014f5a3eebc999325eba726c1ddab8a2d943e719`다. 최적화 배포판과의 절대 시간 비교는 아니다.

### 작은 문서 재사용

4쪽 실문서, 한 쪽 보기 100%, 메모리 관찰 off. 최초 준비 및 34→100% 복귀 뒤 각각 20회 이동했다.
before/after 네 실행 모두 기록 구간 **추가 main/image raster 0회**, 모든 trace complete, 오류 0.
누적 LRU eviction 0, 소유 pixel 14,266,592로 같았다. cache hit도 각 시나리오 종료 누적 10/20으로
동일했다. cold 첫 렌더나 브라우저 전체 frame p95의 검증으로 확대 해석하지 않는다.

## 보존한 계약과 남는 한계

- 예산 안 warm surface와 기본 visible 32M / retained 40M 예산은 유지했다. 읽기 화질 보호를 위해
  기존 settled-visible 64M 하향 gate는 제거했다. 전역 해제·장치 성능별 정책은 도입하지 않았다.
- offscreen 편집 bitmap을 반환해도 문서·caret/focus 상태를 바꾸지 않는다. 재진입/strict 복원 회귀를 추가했다.
- 큰 preview가 남은 축소 정착에서는 prefetch를 보수적으로 거절할 수 있다. visible 완료 뒤 자동 재시도는
  추가하지 않았으므로 모든 cold 스크롤 지연 감소를 약속하지 않는다.
- 대형 Canvas 최초 raster/native 전달의 고배율 멈춤은 잔존한다. 타일/영역 렌더·Worker는 별도 조사
  로컬 초안 상태이며 새 이슈를 게시하지 않았다. #6821 눈금자 스크롤 지연도 이 PR 밖이다.
- 동일 physical surface 재사용·guide 분리 실험은 사용자 수용을 통과하지 못해 제외했다.
- 읽기 화질 보호는 정책 보정이다. “기존 DPR 정책을 전부 유지한 순수 CPU 최적화”라고 설명하지 않는다.

## 제출 구성과 다음 게이트

제품·회귀 테스트·재사용 측정 패널과 안내, 수행/구현 계획·단계별 판단, 이 보고서, 최종 요약 JSON만
제출한다. 중간 원시 20개는 [로컬 보존 안내](../working/assets/issue6040-post-stack/README.md)에 따라
개발/백업 branch에 유지하며 제출 branch의 부모 이력에서도 제외한다. 원본 사용자 첨부는 삭제하지 않는다.
새 PR 번호를 예상한 review 문서는 만들지 않았다. PR 본문 초안과 생성 명령은 로컬에만 준비한다.

추가 코드 리뷰, 최신 base·required checks·사용자 원격 승인 후에만 제출/merge/이슈 종료한다.
이번 문서는 GitHub approve나 merge 승인이 아니다.
