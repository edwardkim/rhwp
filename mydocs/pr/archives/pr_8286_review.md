---
kind: report
status: active
last_verified: 2026-11-07
---

# PR #8286 리뷰 — 병합된 #8226 이후 부분 문단·저장 재개방 보정

## 최종 판정

**메인터너 보정 후 수용 가능 — 최신 devel을 반영한 #8286 통합 후보의 로컬 필수 검증 완료.**
최신 code head의 전체 nextest·Native Skia·lint·시각·편람 검증을 완료했다. 원격 push, PR 생성, 승인, 병합은 수행하지 않았다. 기존 결과의 검증 기준은 아래 고정 SHA이며, 최신 devel 결과로 대신하지 않는다.

## 최신 devel 반영 후 추가 확인

2026-11-07 이후 병합 준비 재개 시 실제 devel `1e16a19aede5ccb2657d35974940eb596286cdf1`을
merge 커밋 `c95abf73d78fd2da2c83a59531fa02872f638e5d`에 반영했다. 기존 PR head `afc38f95dc`와 새 devel의 변경 파일은
겹치지 않았고, 제품 코드 충돌이나 추가 수정 없이 결합했다. 검증 tree는
`60f720a2b6a2a78ce59fce8ddc66e77a915240fb`다.

- 최신 base 기준 manifest·diff 검사 PASS.
- release-test 선택 회귀 22 PASS / 0 FAIL: 부분 문단·컨테이너, 선행 정의 슬롯,
  단색 배경 저장, 공유 레이어 캐시, 표 크기 속성 보존.
- 기존 전체 12,789개 및 Native/WASM·편람 시각 검증은 아래에 명시된 이전 코드 기준이다.
  이번 최소 결합 확인에서 전체 검증을 반복하지 않았으며 최신 결합본의 실행 결과로 바꾸어 쓰지 않는다.
- 새 원격 head의 CI는 push 뒤 확인한다. 실제 병합·원 PR 종료는 아직 수행하지 않았다.

[추가 검증 원장](../assets/pr8286_8226_integration/latest-base-validation.json)에
입력 head/base, merge tree, 선택 검사와 원본 로그 SHA를 보존했다.

## 접수와 통합 경로

- 원 PR: [#8286](https://github.com/edwardkim/rhwp/pull/8286), [#8226](https://github.com/edwardkim/rhwp/pull/8226), 작성자 `@semanticist21`.
- #8286 원 head: `bf5bfb34a3b54b91e8c320d622320fe3962a67ff`.
- #8226 원 head: `b97b418a90d833af4ffbf2949f0e517ec15c61e0`; contributor head 수정 권한이 없는 경로다.
- 고정 devel: `b8dd67c5001dfb0410e9e192528676396cdd6a4e`.
- 현재 검증 code head: `ab1491d40b43c7d54941337b852f7e7f0a5eff5e`.
- 이전 완료 코드: `170dd2412206b48cfbf925c5f5a67dcf93437e81`, 이전 base: `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`.
- 로컬 브랜치: `review/integration-20261107-roundtrip-layout`.
- 작업지시자가 두 PR의 통합과 추가 보정·전체 검증을 요청했으므로 체리픽 통합 경로를 사용한다.
  원 기여자 브랜치와 PR 본문은 수정하지 않았다.
- #8226은 별도 통합 [#8711](https://github.com/edwardkim/rhwp/pull/8711)로 이미 반영됐다.
  #8711은 2026-11-07 02:45:33 UTC에 `e724c6caff896a9478dc537cab4e99a57cf879cf`로 병합됐고,
  #8226은 같은 날 02:50:54 UTC에 대체 완료로 닫혔다. 이 작업은 그 원격 조치를 수행하지 않았다.
  최신 devel `b8dd67c500` 위에서 #8286 보정과 이미 반영된 #8226 기능을 함께 검증하는 후속 통합 경로로 정리한다.

## 기여자 크레딧과 원 커밋 대응

| 출처 | 원 커밋 | 통합 커밋 | 저자·처리 |
| --- | --- | --- | --- |
| #8286 | `bf5bfb34a3b54b91e8c320d622320fe3962a67ff` | `bf4b9e776615d992300ae13b6e87f7c86512aae1` | semanticist, `cherry-pick -x` 보존 |
| #8226 | `d2a7376aa00c6b4cc2fb1a7ad84240094879d8b8` | `80de584391e70c3fe04255abd1c087a221640582` | semanticist, `cherry-pick -x` 보존 |
| #8226 검토 | `42f6798a5198584832a82ce9da0fe5b8165f0108` | `a41fbcd03bb6bf17da3bbdd18e309388bbdd2f3d` | postmelee, request Storage 호환 보정 출처 기록 |
| #8226 증거 | `b97b418a90d833af4ffbf2949f0e517ec15c61e0` | `28ac0dd0e99f7e4abd7580f8db2ec8b22e33efa2` | semanticist, 증거와 author 보존 |
| #8711 조판 보정 | `24bb81d0320a8d666b76abda9ad2e991040f124a` | `6d916b00e08d658e9345fa08dc1582f4facf8923` | postmelee, 머리말·꼬리말 포함 TAC host 재조판 |
| #8711 추가 회귀 | `effcb0d60241d41601fa3cc471c6e46050a60676` | `01ad5cd9971e645dc8b899697dc1bfd855570792` | postmelee, 저장 재개방 본문 보존 |
| 통합 보정 | 이 브랜치 | `799bf9b7760349fcb8b6e43550c0271a89df4c70`, `170dd2412206b48cfbf925c5f5a67dcf93437e81` | 기여자 커밋 뒤의 별도 보정 |

기여자 author `semanticist <semanticist0@gmail.com>`와 원 SHA 출처를 보존했다. contributor 커밋을
amend·squash·force-push하지 않았다. 폐기한 중간 구현은 revert 커밋으로 제거했으며 최종 diff에 남지 않는다.

## 원인과 최종 소비 경로

#8286 원 기여는 부분 문단의 저장 줄 범위가 현재 조합 줄 수를 넘는 슬라이스 panic을 막는다.
그 뒤 실제 저장 재개방을 검증하면 표 앞 글자, 셀·글상자 어울림, 표 뒤 기준 문자 줄의 소유가
일치하지 않았다. #8226은 선행 정의 삽입에서 기존 8-unit 제어 슬롯을 보존하지만 현재 저장 경로가
무효화한 줄 캐시를 다시 구성할 때 머리말·꼬리말 정의가 포함된 host도 받아들여야 한다.

| 값 | 생산과 실제 소비 |
| --- | --- |
| 셀·머리말·꼬리말·글상자 어울림 | 유효 안쪽 폭과 문단 기준 개체의 배제 영역 → 공통 band frame → 저장 줄 수용 또는 재조판 → paragraph/table/shape 배치 |
| 인라인 표 줄 소유 | 현재 frame의 원시 제어 슬롯 → `composed_tac_line_assignment_with_storage` → `tac_flow`의 실제 inline owner |
| 표와 뒤 기준 문자 | U+FFFC 개체 경계와 폭 0 Header/Footer 토큰 → 서로 다른 물리 줄 → 부분 문단 꼬리 줄의 실제 전진 |
| 중첩 셀 높이 | frame이 재조판한 inline table 소유를 높이 측정까지 전달 → 과거 native HWP5 선언 높이로 다시 축소하지 않음 |
| 글상자 개체 원점 | 문단/단 기준은 안쪽 text frame, 글 앞/뒤 개체는 정렬별 바깥 여백 → 실제 paint 원점 |

문서 ID 분기, 점수에 맞춘 상수, 좌표 clamp, 원본 IR 변경, 렌더링 baseline 완화는 없다.
원본 storage가 필요한 HWP3 및 실제 frame 소유가 없는 native HWP5 높이 계약은 유지한다.
줄바꿈 경계는 [UAX #14 LB20](https://www.unicode.org/reports/tr14/#LB20)의 개체 경계와
여는 금칙 문자·명시적 결합 억제 규칙을 따른다.

## 독립 한컴 반례

[반례 설명](../../../tests/fixtures/container_charts/inline-anchor/README.md)과
[입력·HWP·PDF SHA 및 작업 ID](../../../tests/fixtures/container_charts/inline-anchor/manifest.json)에
공개 fixture의 원본, 좌우 바깥 여백 제거, 표 폭 축소, 머리말 제거, 꼬리말 치환을 고정했다.
한컴 2020의 독립 HWP 저장 줄과 Print PDF를 함께 사용했다.

원본은 표 물리 폭 42,566 HWPUNIT가 본문 폭 42,520을 넘고, 뒤 머리말 기준 문자는 다음 물리 줄에
남는다. 여백 제거·폭 축소에서는 같은 표 줄에 남으며, 머리말을 제거하면 초과 폭이어도 별도 꼬리
줄이 생기지 않는다. 꼬리말 치환은 같은 줄 소유를 재현한다. 따라서 문단 끝에 무조건 빈 줄을
더하는 보정 대신 실제 제어 슬롯·frame 배정을 사용한다.

5개 회귀는 원본/HWP/HWPX 재개방에서 독립 HWP가 기록한 **앞뒤 본문 줄 원점의 차**와 실제
배치의 차를 대조한다. 저장이 원본 IR을 바꾸지 않는지도 검사한다. 절대 페이지 좌표나 고정
영역으로 내용을 선택하지 않았다. 원본 fixture와 독립 PDF는 검토 code commit에 포함되어 있다.

## 최신 devel 통합 후 검증

#8711/#8226의 이미 병합된 변경을 보존해 `ab1491d40b43c7d54941337b852f7e7f0a5eff5e`에 통합했다.
제품 소스 충돌은 없었다. 충돌한 #8226 검토 문서와 테스트 근거 주석은 최신 devel의 내용을 보존했다.
현재 devel 대비 제품 diff는 #8286 보정 범위이며, #8226 serializer 변경을 중복 적용하지 않는다.

- Native Skia lib 3,857 PASS / 12 ignored, workspace 보조 lib 182 PASS, 그림 누락 2 PASS, 직접 PDF 22 PASS.
- fmt·Native/WASM/workspace Clippy·workspace build·doc·manifest·unit-tier 정책 모두 PASS.
- 관련 문서 32종 88쪽씩, Native/fresh WASM **176쪽**에서 최저 92.18% / 92.54%.
  최신 devel과 같은 입력·PDF SHA·글꼴 조건의 **점수 하락 0쪽, 최대 하락 0.00%p**이며,
  **176개 PNG 모두 바이트가 같다**. 새 devel의 #8555·#8556·#8557 문서 13쪽씩을 포함한다.
  [전쪽 대조 원장](../assets/pr8286_8226_integration/resync/devel-matrix-comparison.json).
- 원본·이전 저장본 5종과 독립 반례 5종은 양쪽 backend 모두 전쪽 90% 이상이다.
  원본·이전 저장본 PNG 10개도 이전 검증 코드의 출력과 바이트가 같다.
- 최신 저장 API가 생성한 HWP/HWPX 바이트는 사용자 승인 파일과 정확히 같다.
  [바이트 대조](../assets/pr8286_8226_integration/resync/resync-save-equivalence.json),
  [독립 PDF 재검증](../assets/pr8286_8226_integration/resync/resync-saves-verification.json):
  Native 95.51%, fresh WASM 99.56%, 모두 1쪽.
- 같은 최종 HWP를 최신 devel에서 열면 Native 50.91%, WASM 50.24%다.
  최종 HWPX는 devel에서 Native 줄 범위 panic / WASM unreachable로 실패한다.
  실패를 0점 TSV로 대체하지 않았다. 후보는 같은 두 입력을 모두 통과한다.
- 전체 nextest **12,789 PASS / 0 FAIL / 45 skipped**, 55개 바이너리, 557.763초.
  [최종 기록](../assets/pr8286_8226_integration/resync/full-nextest.json)에 명령·head·로그 SHA를 보존했다.
  캐시 ELF 5개의 실행 비트 누락을 복구한 뒤 전체를 처음부터 통과했으며 소스·검사·기대값 변경은 없다.
- 최신 code head의 Linux Native 편람 **383쪽 PASS**, 최저 **99.96%**, 공식 기준 대비 회귀 **0쪽**.
  [검증 원장](../assets/pr8286_8226_integration/resync/handbook.json),
  [공식 게이트](../assets/pr8286_8226_integration/resync/handbook-gate.json),
  [전쪽 TSV](../assets/pr8286_8226_integration/resync/handbook-native.tsv)에 실제 실행을 고정했다.
  편람 입력·독립 PDF도 실행 바이트와 검증 code commit의 Git blob이 일치한다.
  최저 점수 130쪽의 [독립 PDF](../assets/pr8286_8226_integration/resync/handbook-p130-reference.png)와
  [최신 Native](../assets/pr8286_8226_integration/resync/handbook-p130-native.png)를 직접 대조해
  본문·그림 틀·간격 보존을 확인했다. 글꼴 획·래스터 차이는 남는다.

[최종 저장본 Native review](../assets/pr8286_8226_integration/resync/final-save-native-review.png)와
[overlay](../assets/pr8286_8226_integration/resync/final-save-native-overlay.png),
[WASM review](../assets/pr8286_8226_integration/resync/final-save-wasm-review.png)와
[overlay](../assets/pr8286_8226_integration/resync/final-save-wasm-overlay.png)를 정본 Visual Sweep으로 생성했다.
표 경계·앞뒤 문단·셀 안 흐름·머리말·글상자 원점과 #8555 p1 / #8556 p1 / #8557 p3의
그림 위치·본문 순서를 독립 Print와 직접 대조했다.
[최종 저장본 수정 전후](../assets/pr8286_8226_integration/resync/final-save-before-after.png),
[#8555 p1](../assets/pr8286_8226_integration/resync/preservation-8555-p001.png),
[#8556 p1](../assets/pr8286_8226_integration/resync/preservation-8556-p001.png),
[#8557 p3](../assets/pr8286_8226_integration/resync/preservation-8557-p003.png)에 실제 래스터를 같은 배율로 보존했다.
글꼴 굵기·래스터 차이는 남는다.

## 이전 코드에서 완료한 로컬 검증

아래 결과는 `170dd241` / base `a7feb849`의 기록이다. 최신 head의 새 실행과 구분한다.


- Linux Native 편람 **383쪽 PASS**, 최저 99.96%, 공식 기준 대비 회귀 0쪽.
  [게이트 원장](../assets/pr8286_8226_integration/final-handbook.json)과
  [전쪽 TSV](../assets/pr8286_8226_integration/handbook-native.tsv)를 보존했다.
  최소 일치율인 130쪽의 [독립 PDF](../assets/pr8286_8226_integration/handbook-p130-reference.png)와
  [Native](../assets/pr8286_8226_integration/handbook-p130-native.png)를 직접 대조해 본문·그림 틀·간격 보존을 확인했다.
  초기 Git 소유권 오류는 이 전용 컨테이너의 정확한 checkout만 신뢰 대상으로 등록하고
  동일 ELF로 383쪽 전체를 새 출력에서 다시 실행해 해소했다.
- 독립 최소 글꼴 Linux 전체 nextest: **12,777 PASS / 0 FAIL / 45 skipped**, 55개 바이너리, 579.305초.
  초기 컴파일 메모리 초과는 동시성 1로 재실행했고, 실행 비트가 빠진 캐시 ELF 5개의 권한을
  복구한 뒤 전체를 처음부터 통과했다. 제품 코드·검사·기대값 변경은 없다.
  [최종 실행 기록](../assets/pr8286_8226_integration/final-full-nextest.json)에 결과·환경·원본 로그 SHA를 보존했다.
  실행 원본 로그는 저장소 규칙에 따라 ignored `output/`에 둔다.
- 집중 회귀 39개 PASS. 기존 #2319, #6601, #7419, #7679, #7872, #8226, #8399, #8418 관련 계약과 새 독립 반례 포함.
- 최적화 Native `release-test` 및 wrapper를 통한 fresh release WASM/wasm-opt 빌드 완료.
- Native Skia lib 3,857 PASS / 12 ignored, 그림 누락 2 PASS, 직접 PDF 22 PASS.
  첫 lib 실패는 격리가 테스트 생성 Noto 파생 글꼴까지 차단한 것이었다. 해당 임시 fixture 경로만
  허용하고 동일 컴파일 바이너리의 lib 전체를 다시 실행해 0 FAIL을 확인했다.
- fmt, root·WASM·workspace all-target Clippy, workspace build, doc test, 고정 base manifest·unit-tier 정책 PASS.
  포맷 뒤 파생 harness drift는 정본 생성기로 재생성한 뒤 manifest와 fmt를 각각 다시 통과했다.
- 원본·이전 저장본 5종: Native 최저 93.83%, fresh WASM 최저 95.83%, 모두 1쪽.
- 독립 반례 5종: Native 최저 93.50%, fresh WASM 최저 95.75%, 모두 1쪽.
- 관련 회귀 29종 75쪽씩: Native 최저 92.18%, fresh WASM 최저 92.54%.
  [devel 비교 원장](../assets/pr8286_8226_integration/devel-matrix-comparison.json)의 150쪽에서
  같은 backend·입력·PDF·글꼴 조건의 점수 하락은 0쪽, 최대 하락 0.00%p다.
- #8226 원본과 최종 CLI HWP 저장본은 Native/fresh WASM 모두 100%.
  저장본 SHA는 `729ed729fec4da78c376bcfb5b18eaf12497554b5daabb846ad57e2b35f1bbf2`다.

[전쪽 TSV 원장](../assets/pr8286_8226_integration/page-evidence.json),
[Native producer](../assets/pr8286_8226_integration/final-native-producer.json),
[WASM producer](../assets/pr8286_8226_integration/final-wasm-producer.json)에 실행 입력·산출물 해시를 연결했다.
원 devel의 과거 HWPX 저장본은 범위 초과 panic으로 PNG를 만들지 못했다. 이 실패를 0점 TSV로
조작하지 않았으며, 원 #8286 수정과 최종 후보에서 1쪽 전체 출력·내용 보존을 확인했다.

## 직접 판독과 남은 차이

[원본 비교](../assets/pr8286_8226_integration/source-review.png),
[이전 저장본 비교](../assets/pr8286_8226_integration/previous-save-review.png)에서 표 앞뒤 본문,
셀의 어울림, 글상자·머리말 원점, 표 뒤 간격을 같은 배율로 확인했다.
표·텍스트 상자와 일부 차트의 겹침은 원본 한컴 출력에도 있다. 차트 글꼴·격자선의 래스터 차이는
남는다. 공급한 HCR 글꼴에 수달 글리프가 없어 Native에서 빠지고 Chrome에서는 색상 emoji로
대체되는 차이도 있다. 실루엣 통과를 픽셀·글리프 완전 일치로 해석하지 않는다.

[관련 회귀 19쪽](../assets/pr8286_8226_integration/regression-8418-p019-review.png)과
[12쪽](../assets/pr8286_8226_integration/regression-8418-p012-review.png)은 글꼴 모양·굵기 차이가
남지만 내용 순서·줄 위치·다음 문단 간격은 보존되며 devel과 후보 점수도 같다.

DocumentCore의 원래 재저장 경로는 차트 OLE 그림을 보존하지 못한다. 이전·최종 저장본의 한컴
출력과 로컬 출력에서 구분하며, 이 통합이 차트 보존 기능을 해결했다고 주장하지 않는다.
CLI 직접 변환과 DocumentCore 저장 API는 별도 경로이므로 산출 바이트를 혼용하지 않았다.

## 최종 저장본 독립 검증

사용자가 정확한 최종 HWP/HWPX SHA의 동일 서버 전송을 승인한 뒤 한컴 2020
11.0.0.9136 Print method 0으로 각각 1쪽 PDF를 생성했다. `font_scope=verified`다.
이전 저장본 PDF를 재사용하지 않았다. 두 PDF와 입력 바이트의 SHA·작업 ID·전쪽 TSV는
[최종 저장본 원장](../assets/pr8286_8226_integration/final-saves-verification.json)에 기록했다.
Native는 두 파일 모두 **95.51%**, fresh WASM은 두 파일 모두 **99.56%**이며 쪽수도 일치한다.
HWP/HWPX의 기준 PNG와 각 backend PNG는 각각 바이트가 같았다.
[한컴](../assets/pr8286_8226_integration/final-save-reference.png),
[Native](../assets/pr8286_8226_integration/final-save-native.png),
[WASM](../assets/pr8286_8226_integration/final-save-wasm.png)을 직접 대조해 표 앞뒤 본문,
셀의 흐름, 글상자·머리말 원점, 표 뒤 간격을 확인했다. 위 글리프·기존 차트 보존 한계는 그대로다.

## 남은 병합 전 조건

1. 로컬 필수 검증과 입력 해시 대조는 완료했다. 검토 문서 확인·push 승인 뒤 통합 PR을 등록하고 해당 head CI를 확인한다.
2. 실제 병합 직전에 devel·원 PR head·통합 PR head와 최신 CI를 다시 확인한다.
3. 실제 병합과 원 #8286 후속 정리는 별도 승인 후 수행한다. 이미 닫힌 #8226/#8711은 다시 닫지 않는다.

## Merge 후 contributor PR comment 계획

[GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
실제 통합 PR 병합과 후속 comment·close 승인이 이루어진 뒤에만 #8286에 한국어 존댓말로 게시한다.
원 기여자에게 감사하고 `bf5bfb34` → `bf4b9e77`의 author·출처 보존, 별도 메인터너 보정,
실제 merge commit 및 그 head CI를 구분해 설명한다. 이미 대체 완료된 #8226/#8711에는 다시 게시하거나 닫지 않는다.

최종 HWP/HWPX는 각 1쪽이며 기준·backend별 PNG가 형식 간 바이트까지 같음을 확인했다.
대표 HWP 1쪽의 raw gate flagged 후보는 Native/WASM 모두 0개이며, 쪽수 차이·측정 누락도 없다.
Native pixel_match 99.25904%, visual_accuracy_proxy_percent 21.97698%, raw 95.51442%;
WASM은 각각 99.23782%, 21.58616%, 99.55876%다. 이 보조 proxy는 글꼴·가는 획·색상 래스터
차이에 민감하며 사람의 정확도나 배치 보존 비율이 아니다. 직접 판독은 표·앞뒤 본문·컨테이너
원점과 간격 보존을 확인했으며, 수달 글리프와 기존 차트 OLE 저장 한계는 그대로 명시한다.

대표 이미지는 `mydocs/pr/assets/pr8286_8226_integration/resync/`의
`final-save-native-review.png`, `final-save-native-overlay.png`, `final-save-wasm-review.png`,
`final-save-wasm-overlay.png`, `final-save-before-after.png`다. 병합 뒤 실제 SHA로 다음 형식의 URL을 채운다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr8286_8226_integration/resync/final-save-native-review.png`

해당 asset이 merge commit을 통해 devel에 존재함을 확인한 뒤 review와 standalone overlay를
Markdown 이미지로 넣어 `--body-file`로 게시한다. 게시 후 API로 실제 Markdown과 이미지 응답·해시를 재확인한다.
