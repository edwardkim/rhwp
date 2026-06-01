# PR #1162 검토 — diff-engine-readme UI 진입점 안내 추가

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1162 |
| 제목 | docs(compare): diff-engine-readme에 UI 진입점 안내 추가 (#571, #799 후속) |
| 작성자 | [@xogh3198](https://github.com/xogh3198) (taeho) — **기존 핵심 컨트리뷰터** |
| base ← head | `devel` ← `xogh3198:main` (cross-repo) |
| 상태 | OPEN, MERGEABLE |
| 변경 | +9 / -0, 1 파일 (문서 전용) |
| 라벨 | documentation |
| 마일스톤 | v1.0.0 |
| 연결 이슈 | #571, #799 후속 (closes 명시 없음 — 문서 보강) |

## 2. 컨트리뷰터 이력 (누적 점검)

`gh pr list --author xogh3198` 누적:
- #571 DIFF update (closed)
- #623 문서 비교·이력(compare + history) 및 diff-engine 정합 (closed)
- #625 페이지네이션 그림 높이 예약 + 빈 문단 높이 정규화 (closed)
- #799 Task #571 compare + history + diff-engine 정합 (closed)
- **#1162 (본 PR)** — 자신의 compare/diff-engine 작업에 대한 사용자 진입점 문서화

`project_external_contributors` 메모리 등재: "@xogh3198: PR #571 base skew 옵션 C 처리 사례". README.md:114 v0.7.13 사이클 기여자 목록에도 이미 포함.

## 3. 변경 내용

`rhwp-studio/src/compare/diff-engine-readme.md` §6.1 신설 (§6 함수 매핑 표 직후):

```
### 6.1 rhwp-studio 진입점
| 기능 | 메뉴 | 단축키 | 엔진 |
| 문서 비교 | 편집 → 문서 비교 | Alt+Shift+V | compareDocuments — strategy:'alignment' (외부 두 파일) |
| 문서 이력 | 편집 → 문서 이력 관리 | Ctrl+Shift+H | compareSnapshots — strategy:'identity' (같은 세션·stable_id) |
+ 비교 결과 탐색·상세 창은 compare-result-window.ts, 세션 공유는 compare/session.ts 참고
```

## 4. 검증 — 문서 내용 ↔ 코드 정합 (전 항목 일치)

| 문서 기재 | 코드 실증 | 정합 |
|-----------|-----------|------|
| 메뉴 "문서 비교" | `edit.ts` `id:'edit:compare-documents' label:'문서 비교'` | ✅ |
| 메뉴 "문서 이력 관리" | `edit.ts` `id:'edit:document-history' label:'문서 이력 관리'` | ✅ |
| 단축키 `Alt+Shift+V` | `edit.ts:168 shortcutLabel:'Alt+Shift+V'` | ✅ |
| 단축키 `Ctrl+Shift+H` | `edit.ts:183 shortcutLabel:'Ctrl+Shift+H'` | ✅ |
| `compareDocuments` (외부 두 파일) | `diff-engine.ts:3095` | ✅ |
| `compareSnapshots` (세션) | `diff-engine.ts:2991` | ✅ |
| `strategy:'alignment'` (비교) | `compare-dialog.ts:14 strategy:'alignment'` | ✅ |
| `strategy:'identity'` (이력) | `history-dialog.ts:42 strategy:'identity'` | ✅ |
| `compare-result-window.ts` | `rhwp-studio/src/ui/compare-result-window.ts` 실재 | ✅ |
| `compare/session.ts` | `rhwp-studio/src/compare/session.ts` 실재 | ✅ |

**소견**: 문서 전용 변경이며 기재된 메뉴·단축키·함수·전략·파일 경로가 현 코드와 100% 일치. 코드/기능 변경 없음 → CI 빌드/테스트 영향 없음 (`no checks reported`는 head=fork main 브랜치라 정상).

## 5. 별도 사안 — 기여자 목록 계정 정정 (작업지시자 제기)

작업지시자: "이 컨트리뷰터는 기존 PR로 기여했으나 **GitHub 계정이 잘못되어 기여자 목록에서 누락**된 분."

- 과거 PR(#571/#623/#625/#799)의 **커밋 author 이메일/이름**이 GitHub 계정(@xogh3198)과 연결되지 않아 기여 그래프·목록에 미반영된 정황으로 추정.
- README.md 텍스트 기여자 목록(v0.7.13, line 114)에는 @xogh3198 이 이미 포함되어 있음.
- **이 PR 머지와 분리**하여 다룰 사항. 정정 범위(어느 목록/그래프, 어떤 올바른 계정으로)를 작업지시자 확인 필요.

## 6. 판단 (잠정)

문서 정합성 검증 통과, 컨트리뷰터 의도와 코드 일치. **merge 권장**.

- 처리 방식: squash merge (문서 단일 변경, fork main → devel)
- 기여자 계정 정정은 별도 처리 (5절)

> 승인 시 검증(이미 완료) + merge 진행 → `pr_1162_report.md` 작성.
