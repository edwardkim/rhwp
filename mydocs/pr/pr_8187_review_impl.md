---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8187 메인터너 통합·검증 계획

관련 [검토 기록](pr_8187_review.md). **2026-11-03 작업지시자가 메인터너 보정·필수 로컬 검증을 승인했다.**
아래 범위의 통합·검증을 완료했으며 원격 push·댓글·merge는 승인 범위에 포함하지 않는다.

## 고정 범위와 목표

최신 devel `4e2cc6bb36576b95d58d4c76cd833281ea652554` 위에서 원 contributor head
`32df7b71a6e731f767e4945e3e1d79ca0cac3e58`의 커밋과 credit을 보존한다.
`review/pr-8187-logical-editing`에서 다음 기능을 현재 공통 엔진에 통합하고 필수 로컬 검증까지
한 단위로 완료한다. 작업 도중 절편별 승인을 반복해서 요청하는 계획이 아니다.

- 본문 논리 선택 삭제, 내부 클립보드 순수 서식 글 판정, 내부/HTML 서식 글의 논리 붙여넣기.
- 범위 재조판과 구역 갱신, 저장 분할 셀의 원본 문단 범위 노출.
- 명시적 Normal 단 전환의 내용 하단·뒤 간격·첫 줄 수용과 전체 폭 절대 단 너비 저장.
- 인라인 그림·주석·기준선 보정은 현재 devel의 동등하거나 개선된 공통 결과를 우선 보존한다.

#7444 전체 Studio 문제를 해결했다거나 소비 앱 이관을 끝냈다고 표현하지 않는다. 제품·회귀의
허용치를 낮추거나 최신 기능을 버려 원 PR에 맞추지 않는다. 원격 push·본문 갱신·comment·merge는
이번 로컬 계획 승인과 구분한다.

## 확인한 11개 충돌의 처리 방침

| 파일/묶음 | 보존·통합할 계약 |
| --- | --- |
| `commands/clipboard.rs` | 최신 `insert_text_from_paragraph`의 원본 메타데이터와 PR의 필드 안팎 소속을 공통 삽입 경로에서 함께 보존. 기존 API 동작도 검증 |
| `commands/text_editing.rs` | 최신 `StoredVerticalFrame`·공통 경계 삭제·이력/무효화 유지. 논리 경계의 개체 판정과 사전 거부를 연결하고 같은 데이터를 두 번 지우지 않음 |
| `paint/builder.rs` | 최신 분리된 `builder/tests.rs` 구조 유지. 새 `model_para_range` 초기화는 실제 이동한 생성자에 반영 |
| `composer/composition.inc.rs` | `try_refresh_equation_rows`·빈 줄 메트릭·Storage 예산과 실패 전달 유지. 인라인 그림 행 보정도 같은 실제 공통 결과에 연결 |
| `layout/paragraph_layout.rs` | 최신 글자겹침/각주 control 순서·공통 note reference를 유지하고 명시 번호 계약을 결합. 중복 마커와 개체 뒤 캐럿 반례 확인 |
| `layout/table_partial.rs` | 최신 parent-owned 셀 프레임·clip·이어받기 소유를 유지하면서 원본 문단 범위 전달 |
| `skia/text_shaping.rs` | 이미 반영된 실제 face별 `baseline_offset`을 유지. 옛 기준선 구현으로 후퇴시키지 않음 |
| `typeset/paragraph.rs` | Storage 기반 이동·빈 페이지 시작 흡수 보존. Normal 부분 쪽의 첫 줄 overflow 이동을 공통 진입 판정과 연결 |
| `typeset/paragraph/split_entry.rs` | #8149의 일반 첫 줄 앞 간격 차감을 보존. 특정 Normal zone에서만 차감하는 옛 분기로 대체하지 않음 |
| `serializer/hwpx/section.rs` | 공통 `write_section_with_storage`에 필요한 단 너비 정규화를 연결. 스트리밍/예산·일반 저장이 같은 규칙을 소비 |
| `tests/common/passthrough_exemptions.rs` | 최신 유효 면제 유지. 새 API의 실제 공통 위임과 거부 시 불변을 확인한 항목만 추가하고 무효화 보호 음성 대조 수행 |

충돌하지 않은 자동 적용 파일도 검토한다. 특히 공개 API의 실패 원자성, `CharOverlap`의 논리 한 칸
편입, 문단/셀 소유·숫자/주석, 절대 단 너비의 정규화와 저장 왕복을 빠뜨리지 않는다.

## 실행 순서와 완료 조건

1. 원 head와 최신 devel을 다시 고정하고 승인된 통합 방침을 같은 검토 branch에 적용한다.
   선행 해결 PR·실제 통합 커밋·독립 반례를 각 변경과 연결한다. 임의 ours/theirs 일괄 선택은 하지 않는다.
2. 작은 실제 경계부터 실행한다. 복사/삭제/붙여넣기 같은 논리 슬롯, UTF-16 보충 문자, 개체 앞뒤와
   같은 위치의 여러 개체, 필드 안팎·지원하지 않는 입력의 변경 전 거부, undo/redo·HWP/HWPX 왕복을 확인한다.
   새 API 5개 회귀 파일의 34개 원 검사를 현행 suite에서 실행하고 최신의 관련 기존 검사도 함께 실행한다.
3. 다단에서는 Normal→Normal과 Distribute/Parallel fallback, 부분 쪽의 빈 단, spacing_before/after,
   실제 내용 하단·뒤 간격의 단일 계상과 다음 본문을 확인한다. 분할 셀은 컷/문단 범위→요구/예약 높이→
   예산 실패 이월→실제 셀/후속 본문까지 코드 위치와 실행 결과로 연결한다.
4. 동일 원본·독립 Print PDF와 명시한 동일 글꼴 공급으로 **최신 base와 통합 후보의 전쪽 Native/fresh WASM**을
   비교한다. 원 PR의 giant 대표 4쪽을 전쪽으로 세지 않는다. raw 90% 미만·누락·쪽수 차이·2%p 초과
   후퇴를 보류하고 원인을 수정한다. KPS 66쪽은 직접 확인한 위 테두리·행/날짜/서명 위치를 포함한다.
   작성자의 옛 점수와 이번 진단은 글꼴 조건이 달라 코드 개선량으로 계산하지 않는다.
5. 각 실제 변경의 시각 선행 조건 충족 후 필요한 관계형 회귀와 기존 검사 정리를 완료한다.
   새 회귀 파일 첫머리에 유형·실제 측정·일시·source·전쪽 최저값과 안정 증거를 적는다.
   기존 결함이라는 주장만으로 활성 검사 제거·기준 완화를 하지 않으며, 동일 원본/미해결 범위와
   정본의 이관 요건을 먼저 확인한다.
6. 최종 source에서 Rust lint 묶음·suite/무효화 정책과 필요한 전체 회귀·Native 공식 검증을 순차 완료한다.
   원 head의 녹색 CI를 새 보정 코드의 성공으로 재사용하지 않는다. 원 코드와 source가 같은 범위의 기존
   producer/증적은 SHA와 조건을 확인한 뒤 재사용한다.
7. 검토 기록에 최종 head·현재 base·필수 검증·시각 판독·남은 실제 차이를 반영하고 원격 처리안을 준비한다.
   사용자 승인 전에는 contributor branch push·GitHub comment/approve·merge를 하지 않는다.

기준 미달이 나오는 경우도 위 범위의 원인 보정과 재검증을 계속한다. 통합을 완료하기 위해 엔진 전체
재작성이나 특정 문서용 수치 보정을 요구하는 계획은 아니다. 기능의 의존성이 실제로 달라져 승인한
통합 방침으로 처리할 수 없는 경우에만 구체적인 변경 범위와 근거를 보고한다.

## 실행 대장 — 2026-11-03

| 단계 | 상태와 증거 |
| --- | --- |
| 원 이력·11개 충돌 통합 | 완료, merge `29b0bb1184` |
| 공통 표 프레임·Native 기준선 보존 | 완료, `c33037bf2f`·`054ef1cd2d` |
| API·경계 집중 회귀 | 완료, 19모듈 72 PASS. 테스트·증거 commit `d69ab19b5e` |
| 전쪽 Native/fresh WASM·직접 판독 | 완료, 각 14문서 516쪽, 90% 미만/누락/기준 대비 하락 0쪽 |
| 저장 왕복·실제 브라우저/Studio | 완료, Chromium 36 PASS, Studio 2716 PASS·Node pkg 전용 2 skip, TypeScript PASS |
| 필수 lint·workspace·정책 | 완료, fmt·Native/WASM/전체 workspace Clippy·build·manifest/unit-tier PASS |
| release-test 전체·Native 공식 3종 | 완료, 12,415 PASS / 4,064 PASS |
| 최종 기록·원격 처리안 | 로컬 작성 완료. 원격 작업은 별도 승인 전 실행하지 않음 |

전체 결과·source·PNG·TSV·제한은 [검토 기록](pr_8187_review.md)과
[증거 안내](assets/pr8187/README.md)에 연결한다. 새 절편 승인 없이 승인된 로컬 범위를 마무리한다.
