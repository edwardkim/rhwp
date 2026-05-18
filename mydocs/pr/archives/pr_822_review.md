# PR #822 검토

## 메타

| 항목 | 값 |
|------|---|
| PR | [#822](https://github.com/edwardkim/rhwp/pull/822) |
| 제목 | feat: Ctrl+E 지우기 커맨드 구현 |
| 컨트리뷰터 | @oksure (Hyunwoo Park) |
| 규모 | +30/-2, 3 파일 |
| 커밋 | 2개 (feat + Copilot 리뷰 반영) |
| mergeable | **CONFLICTING** |
| CI | no checks |

## 핵심 판정: devel에 이미 흡수

PR #822의 인도물이 현재 devel에 거의 전부 구현되어 있음:

| 영역 | PR #822 | 현재 devel | 상태 |
|------|---------|-----------|------|
| `edit:delete` 커맨드 활성화 | `canExecute: ctx.hasDocument` + `performDelete()` | 구현됨 (edit.ts:78-84) | **이미 devel에 있음** |
| `Ctrl+E` 단축키 매핑 | shortcut-map.ts 추가 | 구현됨 (shortcut-map.ts:19) | **이미 devel에 있음** |
| `performDelete()` 메서드 | input-handler.ts 추가 (+개체 삭제 포함) | 구현됨 (input-handler.ts:2414) | **이미 devel에 있음** |
| `Ctrl+ㄷ` 한글 IME 매핑 | shortcut-map.ts 추가 | **미구현** | **유일한 차이** |

## 옵션 분류

### 옵션 A: PR close + `Ctrl+ㄷ` 매핑만 메인테이너가 추가

devel 흡수 사실 안내 + 유일한 차이(`Ctrl+ㄷ`)를 메인테이너가 1줄 추가.

### 옵션 B: PR close만 (Ctrl+ㄷ 매핑 생략)

한글 IME 상태에서 Ctrl+E가 `ㄷ`로 입력되는 경우가 실제로 발생하는지 검증 필요. 다른 Ctrl+알파벳 단축키에 한글 매핑이 없는 것도 있으므로(Ctrl+A, Ctrl+Z 등) 일관성 측면에서 생략 가능.

### 옵션 C: 컨트리뷰터에게 devel rebase 요청

충돌 해소 후 차이점만 남기도록 요청. 다만 실질 차이가 1줄이라 비효율적.

## 작업지시자 결정 요청

옵션 A/B/C?
