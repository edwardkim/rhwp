# PR #8226 통합 검토

## 최종 판정

**메인터너 보정 후 수용 가능**. 원 head는 최신 devel과 충돌한다. 기여 코드와 Storage 호환만 적용한 후보에서는 저장 후 재열기에서 본문이 잘린다.
`24bb81d03`의 제품 보정과 `effcb0d60`의 추가 회귀를 포함한 통합 head가 검토 대상이다.
통합 PR의 최신 head CI 성공과 실제 병합에 대한 작업지시자 승인이 필요하다.

## 접수 정보

- 요청: 외부 PR의 저장 후 본문 잘림을 보정하고 `devel` 대상 통합 PR로 처리.
- 원 PR: https://github.com/edwardkim/rhwp/pull/8226
- 검토 base: `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`.
- 원 head: `b97b418a90d833af4ffbf2949f0e517ec15c61e0`.
- 경로: 충돌 및 `maintainerCanModify=false`이므로 사용자 승인된 체리픽 통합.
- 판정 대상은 보정된 통합 코드 head `effcb0d60241d41601fa3cc471c6e46050a60676`이다. 원 contributor head는 직접 병합하지 않는다.

## 문제와 보정

원 변경은 빠진 SectionDef/ColumnDef를 앞에 넣을 때 기존 Header/Footer 슬롯을 재사용하여
표의 문자 위치가 바뀌던 오류를 고친다. 최신 devel에서는 request Storage를 전달하고 두 벡터의
할당을 좌표 변경 전에 예약하도록 호환 보정했다.

추가로, 암시적 단 정의의 활성화는 본문 저장 줄 분할을 무효화한다. 저장기는 그 캐시를 제거하지만,
다시 열린 HWP의 머리말 정의 때문에 `inline_flow`의 두 수용 조건이 표·글줄 계획을 거부했다.
대체 경로는 작은 합성 글줄 높이를 사용해 본문이 잘렸다. Header/Footer는 별도 프레임의 정의이므로
본문 표·글줄 구성에서 허용하여, 실제 글자와 측정 표가 같은 물리 줄 계획의 높이·기준선을 사용하게 했다.
유효하지 않은 저장 캐시를 재사용하거나 출처 표지를 바꾸지 않았다.

입력에는 `META-INF/rhwp-hwp5-origin`이 있어 HWP 재저장에 HWPX 계보 표지가 없는 것은 정상이다.
직접 API와 CLI의 저장 바이트도 같았다. 따라서 표지 부착·serializer 캐시 보존은 해결책이 아니다.

## 기여자와 출처

| 원 commit | 통합 commit | 내용 |
| --- | --- | --- |
| `d2a7376aa00c6b4cc2fb1a7ad84240094879d8b8` | `d77cb886cbb04a7f98691da19ce696875bceaeb8` | 선행 정의 슬롯 보존 코드·검사 |
| `b97b418a90d833af4ffbf2949f0e517ec15c61e0` | `064c51cb69a7a2f96f14119d718ca77223951252` | 공개 재현 입력·독립 PDF·당시 검증 증거 |

두 체리픽은 `-x` 출처와 semanticist의 원 저자를 보존했다. `03300db4a`는 이전 로컬 검토
`42f6798a5198584832a82ce9da0fe5b8165f0108`의 Storage 호환 보정이며, `24bb81d03`는 별도의
통합 조판 보정이다. 기여자의 PR 본문과 원 브랜치를 수정하지 않았다. 통합은 #4680/#4687의
이 좁은 저장 경로를 다루며 이슈 전체 해결을 주장하지 않는다.

## 최신 devel과의 직접 영향 대조

| 기존 변경·계약 | 이번 통합의 처리·검증 |
| --- | --- |
| #8360의 request Storage 전달·할당 한도 | helper에 Storage/Result를 전달하고 control/병렬 record 용량을 좌표 변경 전에 예약했다. Native 단위 검사와 전체 회귀에서 기존 실패 전달 계약을 확인한다. |
| `851ca2705c`의 암시적 단 정의 활성화·저장 줄 무효화 | `stored_text_partition_dirty`와 무효 캐시 제거를 유지했다. 새 재열기 검사는 저장 줄이 없어도 본문 글자 상자를 보존하는지 확인한다. |
| #8302 계보 보존 및 #8310 저장 줄 캐시 계약 | HWP5 출처 표지를 유지하며 임의의 HWPX 표지를 추가하지 않았다. Native·브라우저 저장 바이트 동일성과 반복 재열기를 확인했다. |
| #8507의 글자처럼 취급하는 표 host 물리 행 | 기존 표·글줄 계획의 높이·기준선을 재사용한다. Header/Footer 정의 수용 조건 외에 표 측정·행 배치 알고리즘은 변경하지 않았다. |
| #8389 편람 품질 기준 | 동일 입력·독립 PDF의 Linux Native 383쪽 게이트 통과, 기준 대비 하락 0쪽. Mac 동일 환경 base/head는 383쪽 raw가 모두 같았다. |

## 실제 소비 경로 검증

독립 PDF는 원 PR의 공개 합성 입력을 Mac Hancom HWP 12.30.0에서 Print한 1쪽 문서다.
변경 코드로 만든 출력물을 정답으로 쓰지 않았다. 비교는 96 DPI·print·2022 호환이며
색상 경계 보정 전 2px 실루엣 일치율이다.

| 경로 | 원본 | HWP 저장 후 재열기 |
| --- | ---: | ---: |
| 고정 devel Native | 100.00% | 28.77% |
| 고정 devel fresh WASM Canvas | 100.00% | 28.75% |
| 기여 코드 + Storage 호환 Native | 100.00% | 52.66% |
| 본문 보정 Native | 100.00% | 100.00% |
| 본문 보정 fresh WASM Canvas | 100.00% | 100.00% |
| 본문 보정 Native SVG → Chrome | 100.00% | 100.00% |

[TSV](../assets/pr_8226_integration_20261107/repro-visual.tsv),
[producer·입력·바이트 해시](../assets/pr_8226_integration_20261107/manifest.json),
[실행 시점·도구·글꼴 provenance](../assets/pr_8226_integration_20261107/repro-provenance.json),
[Native 직접 판독](../assets/pr_8226_integration_20261107/native-saved-review.png),
[WASM 직접 판독](../assets/pr_8226_integration_20261107/wasm-saved-review.png).
표 전후 본문·머리말·셀 내용이 한 번씩 온전히 표시되는 것을 review·overlay에서 확인했다.
[동일 영역 확대 판독](../assets/pr_8226_integration_20261107/saved-content-zoom.png)은 render tree의 TextRun·Table 상자 합집합에서 영역을 정하고 네 출력을 같은 비율로 확대한 것이다. 글자 모양·공백·표와 문장의 간격을 확인했다. Native/WASM의 표 선과 SVG의 표 선에는 래스터 농도 차이가 남지만 본문 잘림이나 순서 이동은 없다.

Native binary는 checkout `064c51cb6` 위의 두 파일 미커밋 보정으로 빌드했고, 그 제품 변경을 `24bb81d03`로 커밋했다. 원 실행의 HEAD 표기는 보존하며 binary SHA로 산출물을 식별한다.

기준 devel과 보정 코드의 fresh WASM 최적화 빌드를 각각 완료했다. 기준의 직접 API 저장 3종도 기준 CLI와 같은 바이트다. [기준 producer·실행 기록](../assets/pr_8226_integration_20261107/base-provenance.json), [기준 WASM 저장본 직접 판독](../assets/pr_8226_integration_20261107/base-wasm-saved-review.png)에서 본문 소실과 표의 오른쪽 이동을 확인했다.

fresh WASM 최적화 빌드를 완료했다. 브라우저 `exportHwp()`와 `exportHwpWithReport()`를
글꼴 준비 전에 실행하고 준비 후에도 다시 저장했다. 세 파일은 Native CLI 산출과 SHA-256이
같으며 contentLoss count는 0이다. 이 동일한 HWP를 Native와 fresh WASM에서 재열어 대조했다.

`leading_definition_slots`에 공개 입력을 사용하는 관계형 회귀를 추가했다. 글자 크기를 담는
본문 상자, 표 앞뒤 순서, 본문·머리말·셀 내용의 누락/중복, 저장의 읽기 전용 동작과 두 번의
재열기를 검사한다. 절대 px 좌표를 고정하지 않는다.

공통 소비 경로는 `Header/Footer 정의 → 표·글줄 계획 → 본문 상자/기준선 → render tree → Native Skia·WASM Canvas·SVG`다.
SVG는 전체 글꼴을 로컬로 공급한 실제 Chrome raster에서 원본·저장본을 비교했다. 두 출력 모두 1쪽·실루엣 100%, flagged 0쪽이다.
[SVG 직접 판독](../assets/pr_8226_integration_20261107/svg-saved-review.png), [전체 지표](../assets/pr_8226_integration_20261107/repro-metrics.json).
SVG의 글꼴 내장 파일은 private output에만 두고 PNG 증적만 공개한다.

## 제출 게이트 상태

- Native/fresh WASM 재현 문서 시각: 통과.
- 브라우저 직접 저장 경로: 통과.
- 고정 base 대비 integration manifest(2100 sources, 48 targets), unit-tier(4099 tests) 정책 검사: 통과.
- focused `leading_definition_slots`: Linux Native Skia에서 4건 통과(새 재열기 검사 포함). 테스트 commit `effcb0d60`.
- Native Skia 3종: 통과. lib 총 4040건 통과(12 ignored), 그림 누락 표시 2건 및 직접 PDF 출력 검사 22건 통과.
- Rust fmt와 Native lint: 통과. WASM lint도 통과했다. workspace 빌드도 통과했다. 전체 대상 lint도 통과했다. 전체 회귀도 12,770 passed / 0 failed / 45 skipped로 통과했다(실행 723.936초). 최초 전체 실행의 16건 실패는 sparse checkout의 기존 runtime fixture 누락이었다. 해당 디렉터리를 복원하고 제품 코드·검사·기대값 변경 없이 전체를 재실행했다. Mac focused 최초 실행은 테스트 작성 오류와 sparse include 누락을 수정했으며, 재실행 세션은 완료 결과 없이 종료돼 통과로 세지 않는다. Linux lib 최초 실행은 sparse `template/` 누락으로 3건 실패했다. 기존 파일을 복원한 뒤 전체 lib를 다시 실행해 통과했다.
- [로컬 검증 명령·결과·로그 해시](../assets/pr_8226_integration_20261107/local-validation.json).
- 최소 글꼴 회귀 환경: 로컬 Linux aarch64 컨테이너, Rust 1.93.1, Node 22.23.4, nextest 0.9.151. VM 4 CPU·8 GiB 메모리·8 GiB swap에서 실행한다. OS DejaVu/Liberation과 저장소의 명시 fixture만 노출하고 Mac 사용자·공유 글꼴 디렉터리는 마운트하지 않는다.
- 편람 Mac Native: 383쪽 완료. 현재 devel과 후보의 페이지별 raw 점수는 전부 동일하지만,
  저장소의 #8389 기준에는 미달한다. 27쪽 88.65101%, 183쪽 86.06407%, 277쪽 85.95300%.
  [base/head 대조](../assets/pr_8226_integration_20261107/handbook-base-candidate.json), [383쪽 전체 TSV](../assets/pr_8226_integration_20261107/handbook-base-candidate.tsv).
  같은 제품 코드의 격리 Linux Native에서는 383쪽 전부 #8389 기준을 유지하여 정식 게이트가 통과했다(최저 99.96%, 하락 0쪽).
  [Linux 게이트·해시·글꼴 공급](../assets/pr_8226_integration_20261107/handbook-linux.json), [전체 TSV](../assets/pr_8226_integration_20261107/handbook-linux.tsv).
  Mac의 큰 장 번호 글꼴 차이는 기준·후보에 공통이며, 실패 결과를 삭제하거나 통과로 바꾸지 않았다.
- 원 PR의 과거 CI와 증적은 출처 기록이며 이번 코드의 검증을 대신하지 않는다.
- 사용자 지시에 따라 이 검토 묶음을 같은 통합 브랜치에 push하고 Open PR을 제출한다. 외부 댓글·Approve·merge·원 PR close는 수행하지 않았다.

추가 `samples/` 입력은 없으므로 신규 samples 시각 ratchet 대상은 없다. 공개 재현 입력과 독립 PDF는 기여자 커밋의 경로와 바이트를 보존했다.

통합 PR에는 `Supersedes #8226`, 기여자와 위 출처, 최종 검증 결과를 기록한다.
원 PR은 통합 PR이 실제로 병합된 뒤 별도 승인된 후속 처리에서만 close한다.

## Merge 후 contributor PR comment 계획

[Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
이번 검토의 원본·저장본 대상은 각각 1쪽이며 Native/fresh WASM 네 실행 모두 flagged 후보 0쪽이다.
저장본 1쪽의 `pixel_match_percent`는 Native 99.76269%, WASM 99.76516%이고,
`visual_accuracy_proxy_percent`는 각각 50.61844%, 51.10904%다. 2px 실루엣은 둘 다 100%다.
실루엣 100%는 픽셀 완전 일치를 뜻하지 않는다. 직접 판독에서는 표 전후 문장·머리말·표 셀의
내용과 순서, 잘림 해소를 확인했으며 글리프의 농도와 가장자리 래스터 차이는 남는다.

대표 이미지는 `mydocs/pr/assets/pr_8226_integration_20261107/native-saved-review.png`와
`wasm-saved-review.png`다. 통합 PR이 실제 병합되고 이미지가 devel에 존재하는 것을 확인한 뒤,
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_8226_integration_20261107/native-saved-review.png`
형식의 고정 URL을 쓴다. 원 PR에는 기여자 감사, 통합 PR·실제 merge SHA, 위 체리픽 대응표,
최신 CI 결과와 이 시각 판독을 설명하는 대체 통합 댓글을 준비한다. 댓글 게시·원 PR close는
승인된 후속 단계에서만 수행하고, 게시 후 API로 Markdown과 이미지를 재확인한다.
#4680/#4687의 전체 해결을 주장하거나 자동 종료하지 않는다.
