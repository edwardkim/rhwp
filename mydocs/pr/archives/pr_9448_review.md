# PR #9448 오래된 이슈 7건의 조판·인쇄 회귀 검토

- 코드 head `fc631323409c7708c9b4b2cd7865f5716330ffb5`에서 Build & Test, Native Skia, Render Diff, CodeQL, Adapter inter-diff, Proptest roundtrip과 CI 정책 확인이 성공했다. [PR](https://github.com/edwardkim/rhwp/pull/9448) · [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/45970620797) · [CI 정책](https://github.com/edwardkim/rhwp/actions/runs/45974802993)
- 최신 `devel` `f941c1df6743` 기준으로 문단·표·꼬리말 소속, 중첩 표의 이어받기 경계, HML 수식의 편집·저장과 클릭 소유, KTX 목차와 표 뒤 본문 흐름을 검토했다. 대상 7건의 Native 집중 회귀 12건, Rust 서식과 diff 검사, Studio 단위 검사 92건 및 타입 검사가 통과했다.
- 리베이스로 반영한 #9447의 HWP·HWPX 저장 왕복을 새 Studio와 기존 WASM 조합에서 확인했다. 같은 글꼴 환경에서 독립 Print와 비교한 HWP 2쪽은 기준과 후보의 PNG가 같았다.
- 전쪽 측정 후보 `930e6f77a543`는 이슈별 각 320쪽에서 Native·fresh WASM 최저 98.16%였다. 보호 4,046쪽에서 하락이 없었고 Rust 회귀 14,230건이 통과했다. 리베이스 후 로컬 검사는 작업 지시대로 기본 범위로 실행했다. [전쪽 TSV 및 대표 PNG](../assets/issues7777_7787/README.md)
- 이 기록을 추가한 head의 필수 검사 결과를 확인한 뒤 병합한다.
