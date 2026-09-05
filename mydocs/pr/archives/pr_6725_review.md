---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6725
author: yoonkhsc
---

# PR #6725 검토 - 한글 클립보드 붙여넣기와 HWPJSON 변환

## 결론

**메인터너 보정 됨, 수용 가능.**

PR #6725의 code candidate
`00efe0e00538aab312336b5dc20e119f0cb4f004`는 한글 CF_HTML의 HWPJSON을 HWPX 조각으로
변환해 기존 외부 문서 붙여넣기 경로로 삽입하고, HTML 붙여넣기 및 조판 회귀를 함께 정정한다.
기여자 head `7f7adb062fb91b102a13b06b1e89bc402ec92e50` 검토에서 발견한 데이터 유실 위험은
메인터너 보정 `d64c94ccaa2a97ed38b1969d3b5f0152586d4c2d`로 해결했다. 지원하지 않는
HWPJSON control, 누락한 그림 원본, 손상된 base64 그림을 더 이상 빈 XML/0-byte 그림으로
조용히 바꾸지 않고 명시 오류로 반환한다. 스튜디오 호출자는 기존 HTML 붙여넣기로 안전하게
fallback한다.

`00efe0e0...`의 실제 Full CI, Rust/Python/JavaScript CodeQL, Canvas visual diff,
Adapter inter-diff, Proptest가 모두 성공했다. 이 문서는 GitHub approve event나 merge가 아니며,
review/오늘 기록만 추가한 trailing head의 최신 required check와 `MERGEABLE`/`CLEAN` 재확인은
별도 병합 전제다.

## 대상과 provenance

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6725](https://github.com/edwardkim/rhwp/pull/6725) / @yoonkhsc |
| base at candidate CI | `devel@d1831146587b1ac2346f9ed1216a64c2943a02f9` |
| 최종 code candidate | `00efe0e00538aab312336b5dc20e119f0cb4f004` |
| 원 기여자 head | `7f7adb062fb91b102a13b06b1e89bc402ec92e50` |
| 규모 | 37 files, `+5,066/-99`; public HWPX sample 272,782 bytes 포함 |
| GitHub 상태 작성 시점 | Open, non-draft, `MERGEABLE`, `CLEAN` |
| closing reference | PR 본문에 `Closes`/`Fixes` keyword 없음 |

공개 재현 원본은
[`korea-gmn-198518529-physical-ai-strategy.hwpx`](../../../samples/hwpjson/korea-gmn-198518529-physical-ai-strategy.hwpx)다.
출처는 [과기정통부 공개 다운로드](https://www.korea.kr/common/download.do?fileId=198518529&tblKey=GMN)이며,
SHA-256은 `639c0c64030b98228046fc9562a61d7fb5d6334ad95d8a92b0058b13078e3e74`다.
이 파일의 원본 파일명, 출처, 취득일은 [sample README](../../../samples/hwpjson/README.md)에 기록했다.

## 변경 검토

### HWPJSON native paste 경로

- `document_core/hwpjson/`은 한글 클립보드의 document model을 HWPX header/section 조각으로
  만들고 기존 HWPX parser를 재사용한다. 별도 IR을 도입하지 않아 parser 계약이 이중화되지 않는다.
- `paste_hwp_json_native`는 변환한 외부 문서를 기존 `paste_foreign_document_native`로 넘긴다.
  field ID, 글꼴/스타일 ID, 표 셀, 그림/도형 caption과 그룹 자식의 재매핑은 외부 문서 삽입 경로에
  집중돼 있다.
- HWPJSON control 또는 그림 pack 단계의 오류를 호출 경계까지 전파한다. 이전처럼 지원하지 않는
  control을 빈 문자열로 대체하면 문서가 성공한 것처럼 보이면서 content가 사라질 수 있었으므로,
  이 fail-closed 동작이 필수다.
- field ID 상한 수집은 remapper가 진입하는 Header, Footer, Footnote, Endnote, HiddenComment,
  Field memo, 표/그림/도형 caption과 group child까지 따라가도록 보정했다. 기존 ID와 충돌하는
  새 field를 삽입하지 않도록 하는 production 변경이다.

### HTML/조판 범위

- CF_HTML의 한글 data 주석과 Office fragment를 정리하고, 이미지 data URI가 있는 실사용
  clipboard의 크기 판정을 완화한다. Word, Excel, PowerPoint의 일반 HTML 붙여넣기 경로를
  HWPJSON native path로 강제하지 않는다.
- 글꼴, 자간, 문단 여백, 표/개체 줄 및 저장 조판 없는 문단의 line-breaking을 정정한다.
  기여자 head의 음수 자간 기대값 변경은 `7f7adb...`에서 철회돼, 현재 candidate에는 포함되지
  않는다.
- 변경 범위가 renderer와 paste 경로 모두에 걸치므로 review-only fast-pass로 판정하지 않았다.

## 메인터너 보정

| commit | 보정 | 결과 |
| --- | --- | --- |
| `d64c94cc...` | unsupported control, missing image, corrupt base64를 `InvalidFile` 오류로 전파 | 무음 content 유실 제거, 기존 HTML fallback 유지 |
| `826e3840...` | Rust 포맷 적용 | CI `Format check` 실패 해소 |
| `00efe0e0...` | 새 source-side `cfg(test)` module 제거 | CI test-tier 기준선 정책 복원; production field-ID 보정과 HWPJSON integration 회귀는 유지 |

마지막 보정은 source-side test 총량을 PR base보다 올리지 않는 CI 정책에 맞춘 것이다. 정책 오류를
숨기거나 기준선을 완화하지 않았고, policy workflow 자체가 `v=6;cv=7;mode=full`로 성공했다.

## 실제 GitHub 검증

모든 결과는 code candidate `00efe0e0...`에 대한 실제 GitHub Actions 결과다.

| 검증 | 결과 |
| --- | --- |
| [CI 33947416378](https://github.com/edwardkim/rhwp/actions/runs/33947416378) | Full CI 성공: lint, clippy, test archive A-D, 네 default-feature shard, Native Skia, frontend package gates 성공 |
| [CodeQL 33947416463](https://github.com/edwardkim/rhwp/actions/runs/33947416463) | Rust, Python, JavaScript/TypeScript 분석 모두 성공 |
| [Render Diff 33947416206](https://github.com/edwardkim/rhwp/actions/runs/33947416206) | Canvas visual diff 성공 |
| [Adapter 33947416420](https://github.com/edwardkim/rhwp/actions/runs/33947416420) | adapter inter-diff 성공 |
| [Proptest 33947416413](https://github.com/edwardkim/rhwp/actions/runs/33947416413) | prop roundtrip 성공 |
| [CI Impact Policy 33948083554](https://github.com/edwardkim/rhwp/actions/runs/33948083554) | full mode 성공; Rust, package frontend, render, native Skia, 3개 CodeQL 언어를 실행 대상으로 판정 |

`WASM Build`, `Frontend unit gates`, `Refresh nextest target duration data`는 위 full-mode policy에서
expected skip이었다. 성공으로 오인하지 않았으며, 이 문서에서는 실행된 worker와 aggregate만 통과로
기록한다.

## 시각 증적의 범위와 잔여 위험

- PR 본문의 11-page 비교 PNG는 기여자 `pr-assets` branch 자산이다. 이 review는 그 PNG를 현재
  candidate에서 다시 생성한 독립 정본 증적으로 주장하지 않는다.
- 공개 HWPX sample의 provenance와 exact-head Canvas visual-diff 성공은 확인했다. PR 본문의
  `98.9%` 줄 일치 및 남은 두 차이에 대한 수치는 기여자 보고 수치로만 보존하며, 메인터너가
  동등한 Hancom oracle sweep을 재실행한 결과는 아니다.
- 지원하지 않는 HWPJSON control은 native paste가 아니라 안전한 HTML fallback으로 전환된다.
  native fidelity 확장이 필요한 control은 별도 fixture와 oracle를 갖춘 후속 PR에서 추가해야 한다.
- 넓은 layout 변경이라 public sample 하나가 모든 HWP/HWPX 조판을 대표하지는 않는다. 정확한
  candidate의 CI, existing regression archive, Canvas visual diff가 현 범위의 회귀 방어선이다.

## 최종 판정과 다음 조건

- 판정: **메인터너 보정 됨, 수용 가능**
- 판정 대상: `00efe0e00538aab312336b5dc20e119f0cb4f004`
- 이번 trailing commit: 이 review와 [2026-09-05 오늘 기록](../../orders/20260905.md)만 추가한다.
- 병합 전: trailing head의 required check 전부 성공 또는 expected skip, 최신
  `MERGEABLE`/`CLEAN`, 작업지시자의 별도 병합 승인을 재확인한다.
- 병합 후: `post_merge.md`에 따라 merge SHA 및 `devel` CI를 확인한다. PR 본문에 closing keyword가
  없으므로 별도 issue close를 추정하거나 수행하지 않는다.
