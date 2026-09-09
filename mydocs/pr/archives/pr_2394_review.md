# PR #2394 검토 — document-scoped CanvasKit auto 선택 (seo-rii 5번째)

- Refs #536, P34~P36 후속 (P37 계열). 58파일 +5,540/−281
- 본질: 브라우저 기본 요청 canvas2d → **auto** — fail-closed preflight 가
  문서 전체 단위로 CanvasKit 적합성을 판정(page/work/blocker/font 상한 내
  bounded 스캔, 전체 JSON/리소스 복제 없음)해 적합 revision 만 CanvasKit,
  나머지는 이유를 남기고 문서 전체 Canvas2D. per-op/mid-page 혼합 fallback
  없음. 명시적 override(renderer=canvas2d/canvaskit) 유지.

## 검토 소견

- **fail-closed 설계 일관**: 부적합·불완전·리소스/폰트 준비 실패·초기화 오류
  전부 Canvas2D 수렴 — 조용한 부분 렌더 없음. #2372 direct PDF 와 같은 결.
- **경계 규율**: unsupported op 신규 구현 없음, native Skia/PDF/SVG 불변,
  전체 corpus hard gate 승격 없음(대표 5종만) — "하지 않는 것" 명시.
- **Rust 측**: read_bin_data_limited 등 상한 있는 스트림 읽기(초과 시 명시
  에러), canvaskit_policy.rs typed preflight, wasm_api 질의 1 추가.
- **계약 테스트**: 선택 흐름 자체를 소스 가드로 고정 (auto→preflight→
  ensureCanvasKitRenderer 경로, fallback reason 스키마).
- **e2e/MANIFEST**: 신규 파일 없음, 72=72 정합. #2403 parser 변경과 자동
  병합 — 스위트로 상호작용 무결 확인.

## 잔여 질의 1건 (게이트 완화)

`renderer_baseline_manifest.json` table-core(hwp_table_test.hwp)
inkMaskMaxDiffRatio **0.0185→0.019** (+2.7% 상대) — 커밋("harden VS Code
package contracts")·본문 모두 근거 서술 없음. CanvasKit AA 차이 추정이나
시각 예산 완화는 근거 필수 — 컨트리뷰터 질의 대상.

## 로컬 재실증 (merged tree, devel 충돌 0)

| 게이트 | 결과 |
|--------|------|
| cargo fmt/clippy --all-targets -D warnings | 통과 / 0 |
| 전체 스위트(release-test) | 실패 0 |
| studio: npm ci→tsc→test→build | tsc OK / 388/388 / 빌드 OK |
| npm/editor node --test | 18/18 |
| CI (Render Diff·readiness 포함) | 전 항목 green |

## 판정 대기

1. 컨트리뷰터 질의(임계 완화 근거) 회신
2. **작업지시자 브라우저 시각 확인** — 기본 백엔드 선택 변경이므로 관례상
   최종 판정. `cd rhwp-studio && npm run dev` 후 대표 문서(표 중심 +
   이미지 중심 각 1)에서 auto 선택 백엔드 확인 (진단: 콘솔 renderer
   selection 로그 / F12).

## 작업지시자 실기동 판정 (2026-07-19) — changes requested

세 경로 실기동 검증(auto 선택/auto 수렴/명시 강제) 후 판정:

1. **auto-eligible 문서(hwp_table_test)조차 육안 fidelity 미달** — ①글자
   굵기(bold) 소실 ②기호(□)와 한글 사이 간격 소실·괄호 안 이상 간격
   ③글자 블럭 선택 품질. preflight 가 "지원"으로 판정한 범위 안의 차이라
   fail-closed 게이트가 거르지 못하는 종류.
2. **조판부호/문단부호 보기에서 문서 공백** — 표식 토글이 preflight 재평가
   없이 레이어 트리에 편집 표식 op 를 추가 → CanvasKit replay 실패 추정.
   기능 결함(P1급).
3. ink-mask 예산 완화(0.0185→0.019)의 성격 확정 — 근소 AA 가 아니라 실제
   fidelity 격차를 예산 확대로 숫자상 통과시킨 것.

**판정: 인프라(preflight/session/진단/계약 테스트) 수용 가치 크나, 기본
요청의 auto 전환은 시기상조.** 기본 canvas2d 유지 + auto 는 명시 opt-in
강등, ink-mask 원복, fidelity 3축 + 표식 공백 해소 후 기본 전환 재제안 요청.

## 재검증 (2026-07-19, 개정 head)

seo-rii 반영: ①기본 canvas2d 복귀·auto opt-in 강등 ②marks 를 preflight
blocker 편입(revision 재판정 + 복귀 회귀 테스트) ③ink-mask 0.0185 원복 —
나아가 간격 결함의 근인(일반 TextRun 의 serialized positions 미사용)을
규명해 drawGlyphs 확정 위치 재생으로 수정 (실측 0.0185→0.0079).

- 일괄 검증: Rust 스위트/clippy/fmt/tsc/studio 446/빌드 전건 green,
  wasm Docker 재빌드 후 실기동.
- 작업지시자 시각 판정 (스크린샷 002 2장): **통과** — 줄 배분·bold 개선,
  synam-001 1쪽 fidelity 상당 개선. 잔존(글자겹침·원문자 글리프 소실)은
  opt-in 경로 한계로 후속 개선 트랙.

## 처리 결과

merge 승인(작업지시자 통과 의견) — 개선 확인 코멘트 + 스크린샷 첨부 후 merge.
