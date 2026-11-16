# 원 HFT 막대 칸의 중심 배치: Native 단계

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

2026-11-15, `review/pr-9075-rowbreak-integration`의 메인터너 보정이다.
[구현 근거](../hft-centered-bar-analysis.md)의 독립 Print 칸과 표시 glyph의 자연 전진을
공통 glyph-fit에서 구분한다. 기준 PDF를 다시 만들거나 글꼴 공급 조건을 바꾸지 않았다.

## 실제 검증

| 범위 | 결과 | 근거 |
| --- | --- | --- |
| 보정 전 공개 CFF 중심 배치 | 1 FAIL, raster 112바이트 차이 | [로그](../verification-record.md), [영수증](../verification-record.md) |
| 새 중심 배치와 미확인 원본 대조 | 2 PASS | [정식 suite 로그](../verification-record.md) |
| 관련 12개 모듈 | 93 PASS | [Native 실행 기록](../verification-record.md) |
| Native lib 전체 | 주 프로세스 3,853 PASS·12 ignored, 자식 실행 15·165·2 PASS | [전체 로그](../verification-record.md) |
| Native Clippy | exit 0, `-D warnings` | [로그](../verification-record.md) |
| 공용 helper 수집·격리 Node 검사 | 26 PASS | [로그](../verification-record.md) |
| 원 PR 입력 Native 전 14쪽 | 모두 100.00% | [TSV의 검증 요약](../verification-record.md), [실행 기록](../verification-record.md) |
| 중첩 표 입력 Native 전 17쪽 | 최저 98.59%, 보호값 대비 하락 0쪽 | [TSV의 검증 요약](../verification-record.md), [선행 검증](../verification-record.md) |
| sample10 8쪽·sample16 6쪽 선택 대조 | 하락 0쪽 | 같은 선행 검증 기록 |

최종 Native 바이너리 SHA-256은
`d918d1a05ab0433d3af01a1925eece0670fc7ad45287d8c806732eb21e98d7a3`이다.
제품·테스트 source SHA, 실제 명령·로그·종료 코드는 위 실행 기록과
[단계 영수증](../verification-record.md)에 있다. 회귀는 `target/pr-review`와 최소 공개 글꼴 격리를
사용했고, 시각 비교에는 기존 승인된 일반 시스템 글꼴·명시 환경을 그대로 사용했다.

첫 suite 빌드의 `E0432`는 새 테스트의 공용 helper 소비자 등록 누락이었다.
[실패 기록](../verification-record.md)을 보존하고 기존 생성기에 소비자 한 행을 등록했다.
제품 코드·Rust assertion을 바꾸지 않고 완료된 라이브러리 빌드를 재사용했으며, 새 생성
harness에서 실제 두 검사가 실행됐음을 확인했다. 개별 CI 호출은 추가하지 않았다.

## 같은 페이지의 직접 판독

12쪽의 겹침 번호·뒤 제목 원점은 보존되며 raw 100.00%를 유지한다.
16쪽은 전진폭 보정 뒤 남아 있던 대시의 왼쪽 치우침을 고쳐 99.96%에서 99.99%로 상승했다.
같은 원본 raster 좌표의 3배 확대에서 대시의 가로 중심과 뒤 본문 원점을 대조했다.
대체 glyph의 자연 윤곽 폭·안티앨리어싱이 PDF와 바이트 단위로 같은 것은 아니다.
그 차이를 숨기거나 출력 위치에 문서별 상수를 더하지 않는다.

| 쪽 | review | overlay |
| --- | --- | --- |
| 12 | [Native·PDF·차이](../verification-record.md) | [차이 단독](../verification-record.md) |
| 16 | [Native·PDF·차이](../verification-record.md) | [차이 단독](../verification-record.md) |
| 16쪽 대시 확대 | [동일 좌표·3배 확대](../verification-record.md) | [동일 overlay 확대](../verification-record.md) |

확대는 같은 좌표의 crop과 최근접 배율 변경만 사용했다. 위치 정렬·마스킹·색상 보정은 없다.
독립 PDF의 기존 SHA와 입력 SHA는 단계 영수증에 기록했다. Print 출력은 재산출하지 않았다.

## 남은 필수 검증

이 단계의 Native 선행 결과를 최종 통합 완료로 표시하지 않는다. 새 공유 코드의 fresh WASM,
전 36개 데이터셋 보호 검증, handbook gate, 최종 전체 Rust·lint, 대응 영역 직접 판독이 남았다.
그 결과를 수용한 뒤 상승한 공식 표시값을 정본 TSV·manifest·영수증·요약에 함께 반영한다.
원격 통합 PR 생성·CI·병합·원 PR 종료도 아직 실행하지 않았다.
