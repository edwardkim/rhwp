# 제출 code head의 실제 GitHub CI

- head: `529ba31acbdea187a2ed5d9528be419f26686132`, 제품·생산 source: `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`.
- base: `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0`. 원 기여 head 두 개를 보존했다.
- 모든 required check와 정책 상태는 성공이며 `MERGEABLE / CLEAN`, 실패·대기 0건을 확인했다.
- 조회 시각: 2026-11-16T13:41:23+09:00. workflow와 각 job의 실제 step·로그를 대조했다. 재실행·취소·workflow 변경은 없었다.

| 검사 | 실제 결과와 실행 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/43798244784) | Lint·Native·Frontend·archive A–E·Build & Test 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43798244728) | Rust·JavaScript/TypeScript·Python·Actions 분석과 GHAS CodeQL 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43798244634) | 최적화 WASM·Native CLI 빌드, 실제 Canvas/PDF 비교·CanvasKit 준비 검사 성공 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/43798244724) | 실제 inter-diff worker 성공 |
| [Proptest](https://github.com/edwardkim/rhwp/actions/runs/43798244816) | 실제 property roundtrip worker 성공 |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/43805281894) | 동일 head의 `CI Impact Policy=success` |

Archive A/B/C/D/E의 실제 nextest는 각각 **3,796 / 2,330 / 2,340 / 2,722 / 2,250 PASS**, 합계 **13,438 PASS·0 FAIL·기존 45 SKIP**다. 집계 job의 archive 예상 수량 대조도 성공했다. 이는 CI archive의 집계이며, 로컬 전체 nextest 13,632건과 구분한다. #9076과 #9082의 신규 6개 회귀가 Archive C에서 실제 PASS한 것을 로그에서 확인했다. 정상 skip인 tag-only WASM·미해당 운영 job을 실제 제품 검사 성공으로 세지 않았다.

원격 성공은 [최종 로컬 검증](latest-devel-revalidation.md), 전쪽 TSV·독립 Print 직접 판독을 대체하지 않는다. 문서-only trailing head의 실제 fast-pass와 required check는 다음 단계에서 별도 확인한다.
