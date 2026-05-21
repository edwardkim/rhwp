# PR #950 최종 보고서 — HWPX fragment paste support (close)

- PR: [#950](https://github.com/edwardkim/rhwp/pull/950)
- 제목: feat: add HWPX fragment paste support
- 작성자: dragonnite1221-lgtm (Lee doyun) — 메모리 룰 등재 누적 컨트리뷰터
- base ← head: `devel` ← `dragonnite1221-lgtm:public/hwpx-fragment-paste`
- 이전 PR: [#880](https://github.com/edwardkim/rhwp/pull/880) (closed, 5/15 머지 → 1시간 후 revert)
- 결정: **CLOSED — 메인테이너 hands-on 테스트에서 동작 결함 2건 발견, 재제출 권고**
- 일자: 2026-05-21

## 1. 결정

**close 수용.** 작업지시자 메인테이너 직접 테스트 결과 사용자가 자연
스럽게 사용할 수 있는 시나리오가 아니라고 판단. 컨트리뷰터에게 정중히
재제출 요청.

핵심 기능 (raw HWPX fragment paste primitive, WASM bridge, HWPX 전용
gating, bundled examples, 카탈로그 보안 강화) 의 코드 측 설계는 양호.
PR #880 잔존 6 항목도 대부분 성실히 반영. 그러나 end-to-end UX 결함
2건이 메인테이너 첫 테스트에서 즉시 드러나, 작업지시자가 PR #880 close
시 강조한 "end-to-end testable by maintainer" 게이트를 본질적으로
충족하지 못함.

## 2. 검증 결과

| 게이트 | 결과 |
|--------|------|
| CI: Build & Test / Analyze / Canvas / CodeQL | ✅ 전부 pass |
| 본 환경 cargo build (devel 머지 + PR #950) | ✅ |
| cargo fmt --check | ✅ exit 0 |
| cargo test --release --lib | ✅ 1377 passed, 0 failed (devel 1308 + PR 신규 모듈 +69) |
| cargo test --release --test fragment_paste_* | ✅ 3 passed |
| WASM Docker 빌드 | ✅ Done in 1m 33s (pkg/rhwp_bg.wasm 4.7M) |
| rhwp-studio TypeScript 빌드 | ✅ npm install 후 `npm run build` 성공 |
| **메인테이너 hands-on UX 테스트** | ❌ **결함 2건 — close 사유** |

## 3. 메인테이너 hands-on 테스트 결과

### 테스트 경로
1. `samples/hwpx/blank_hwpx.hwpx` 열기
2. 상단 메뉴 → 입력 → 서식 조각 → 다이얼로그 표시 OK
3. 제목박스 탭 → "문서 제목 박스" 항목 클릭 → 자동 삽입 동작 OK
4. 확인 버튼 클릭 → **다이얼로그 닫히지 않음** ⚠️
5. 취소 버튼만 동작

### 결함 1 (B2) — 다이얼로그 종료 UX 결함

**증상**: 삽입 후 확인 버튼을 눌러도 다이얼로그가 닫히지 않음.

**원인** (`rhwp-studio/src/ui/yangsik-parts-dialog.ts:101`):

```typescript
protected onConfirm(): boolean {
  return false;
}
```

`ModalDialog` base contract (`dialog.ts:127`): "false 반환 시 대화상자 유지"
+ `dialog.ts:56-57`: `if (shouldClose !== false) this.hide();`

→ `YangsikPartsDialog.onConfirm()` 이 명시적으로 확인 버튼을 무시.
카드 클릭 즉시 삽입 패턴이라 확인 버튼이 의미 없는 것으로 의도된 듯
하나, 사용자에게는 "확인 버튼이 보이는데 동작하지 않는다" 로 인지.

**해결 방향 후보**:
- (a) 확인 버튼을 다이얼로그에서 숨김 (base class 옵션 추가)
- (b) 확인 = "닫기" (true 반환)
- (c) 단일 액션 다이얼로그 (popover, side panel) 로 UI 재설계

### 결함 2 (B1) — 빈 HWPX 문서에서 삽입 실패

**증상**:
```
paste failed: malformed fragment: after_para_idx 1 out of range
(section has 1 top-level paragraphs)
```

**원인 추정**: frontend `pos.paragraphIndex` 가 1 로 전달
(커서 본문 첫 paragraph), Rust 측 `paste_paragraphs_into_section` 의
`after_para_idx` 검증 (`fragment_paste.rs:416`) 은 0-based 유효 인덱스
기대 → off-by-one 또는 frontend ↔ Rust semantic 불일치.

**영향**: PR 자체가 fixture 로 추가한 `saved/04-blank_hwpx_empty.hwpx`
또는 일반 빈 HWPX 에서 end-to-end 실패. 메인테이너가 PR #880 close 시
요구한 "end-to-end testable by maintainer" 게이트 미통과.

## 4. PR #880 잔존 항목 처리 현황 (참고)

PR #880 close 시 작업지시자가 정리한 6 항목 중 본 PR 처리:

| # | 항목 | PR #950 본문 주장 | 실제 |
|---|------|-------|------|
| 1 | Dual implementation 명료화 | Architecture note 본문 설명 | ✅ |
| 2 | `(this.doc as any)` 제거 | "Removed by generated WASM binding type" | (V 검증 미수행 — close 결정으로 우선순위 떨어짐) |
| 3 | 환경 의존 테스트 제거 | "Removed local-machine yangsik smoke tests" | ✅ saved/04-blank_hwpx_empty.hwpx 자체 fixture |
| 4 | 주석/코드 정합 | 정렬 주장 | (V 검증 미수행) |
| 5 | 미사용 함수 제거 | `build_occupied_sets` 제거 주장 | (V 검증 미수행) |
| 6 | 예시 fragment 추가 | 3 파일 bundled | ✅ basic-two-cell-table / document-title-box / manifest |
| (추가) | HWP5 호환 | canExecute guard 추가 | ✅ `sourceFormat === 'hwpx'` |

코드 측은 대부분 정직 반영. **그러나 동작 측 end-to-end 테스트가
누락된 것이 본질**. fragment paste integration tests 3 건은 통과하나
실제 사용자 시나리오 (빈 문서 + 확인 버튼 누르기) 는 미점검.

## 5. 본 PR 자체 추가 우려 (수정요청 항목)

본 검토 §4-5 에서 정리한 항목. 재제출 시 함께 반영 권고:

| ID | 항목 | 분류 |
|----|------|------|
| B1 | 빈 HWPX 문서 fragment paste 실패 | 필수 (merge blocker) |
| B2 | 확인 버튼 무동작 + 다이얼로그 종료 경로 모호 | 필수 |
| M1 | 비공개 task 참조 주석 (`task_local_yangsik_paste_*`) 제거 | 필수 |
| M2 | `cross_document_migrate.rs` (1011 줄, 본 PR 미사용) 별도 PR 분리 | 필수 |
| R1 | `yangsik` 명명 UI ↔ 코드 불일치 해소 | 권고 |

## 6. close 코멘트 처리

PR #950 에 close 코멘트 게시 (https://github.com/edwardkim/rhwp/pull/950#issuecomment-4504702165):
- 메인테이너 테스트 경로 + 결함 1, 2 사실 + 코드 위치 명시
- 핵심 기능 설계 양호 + PR #880 잔존 항목 성실 반영 평가
- 재제출 시 권고 사항 (필수 4 + 권고 1) 정리
- 정중한 톤 (메모리 룰 `feedback_pr_comment_tone` 정합 — 차분, 사실 중심)

## 7. 처리 절차 정합 점검

PR 처리 간소화 4단계:
1. ✅ PR 정보 확인 — PR #880 → 머지 → revert → PR #950 follow-up 이력 파악, 작업지시자 5/18 사전 우려 (용어/거버넌스) 파악
2. ✅ 검토 문서 작성 → 작업지시자 "수정요청 전제 검토" 승인 → 본 검토 문서 §1-7
3. ✅ 메인테이너 hands-on 테스트 준비 (test-pr950 임시 브랜치, 네이티브/WASM/rhwp-studio 빌드 완료) → **테스트 결과로 close 결정**
4. ✅ 본 보고서 + close 코멘트 + PR close

Hyper-Waterfall 정합 — 각 단계 작업지시자 승인 후 진행, 임의 결정 없음.

## 8. 메모리 룰 관련성

본 처리에서 적용된 권위 사례:

- **`feedback_visual_judgment_authority`**: 컨트리뷰터 self-validation
  (cargo test + tsc 통과) 과 작업지시자 실제 hands-on 테스트의 격차.
  CI/단위 테스트가 통과하더라도 메인테이너 직접 UX 점검이 본질 게이트.
- **`feedback_pdf_not_authoritative` / `feedback_v076_regression_origin`**
  의 일반화 — "자기 환경 자가검증으로 충분하다 주장하는 컨트리뷰터"
  패턴의 v0.x 단계 일반화 사례. 본 PR 의 fragment_paste integration
  tests 3건 통과는 사실이나 사용자 시나리오 (빈 문서 + 확인 누르기)
  미커버.
- **`feedback_machine_vocabulary`**: 작업지시자 5/18 "양식 부품 용어
  기계적" 우려 정합. UI 라벨 변경 ("서식 조각") 했으나 코드 식별자
  `yangsik-*` 잔존. 일관성 권고.
- **`feedback_external_docs_self_censor`**: 비공개 task 참조 (`task_
  local_yangsik_paste_*`) 가 공개 PR 코드 주석에 잔존 — 외부 공개
  자기검열 누락 사례.
- **`feedback_small_batch_release_strategy`**: `cross_document_migrate
  .rs` 1011 줄 미사용 모듈 동시 도입은 작은 단위 PATCH 회전 정책에
  반함.
- **`feedback_pr_comment_tone`**: 본 처리 코멘트는 차분 + 사실 중심
  톤 유지, 과도한 표현 자제.

## 9. 산출물

- `mydocs/pr/archives/pr_950_review.md` (검토 문서)
- 본 보고서
- PR #950 close 코멘트 (GitHub)

코드 변경 없음 — 본 환경 devel 무영향. 임시 테스트 브랜치 `test-pr950`
정리됨.

## 10. 후속

- **컨트리뷰터 응답 대기** — 위 결함 반영한 새 PR 제출 시 재검토.
  응답 없을 시 별도 follow-up 없음 (PR #880 패턴과 동일).
- **관련 메모리 룰 정리 task 후보** (별도, 본 처리와 독립):
  - 컨트리뷰터 hands-on 검증 게이트 (`feedback_visual_judgment_authority`
    확장) — 단위 테스트만으로 충분하다 주장하는 PR 의 검증 패턴.
  - cherry-pick `--theirs` 주의 (PR #1031 후속 메모리 룰 후보) — 본
    처리와는 별개로 누적 정리.
