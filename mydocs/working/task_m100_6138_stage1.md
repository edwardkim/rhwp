# Stage 1 처리 결과 — #6138 기본 도구 상자 기준선·구조 계약

- **이슈**: [#6138](https://github.com/edwardkim/rhwp/issues/6138)
- **기준**: `upstream/devel@1011a8947`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **계측일**: 2026-08-26 KST
- **결론**: desktop 밀도 1179px, 한 줄 56px, 동적 overflow와 group 경계 이동

## 1. 재현 조건

Stage 1은 제품 source를 수정하지 않고 로컬 Vite `http://127.0.0.1:7718/`와 설치된 Google Chrome
headless에서 기본 문서·default/light 상태를 계측했다. 각 viewport에서 toolbar 높이, 가시 group의 top
좌표 수, group 폭 합과 label 표시 여부를 `getBoundingClientRect()`로 읽었다.

최신 `upstream/devel@1011a8947` 위로 #6118 로컬 커밋을 재배치한 뒤 측정했으며 #6118은 `#icon-toolbar`
제품 source를 변경하지 않았으므로 #6138 기준선과 분리돼 있다.

## 2. 현행 기준선

| viewport | 높이 | 행 | 가시 group 폭 | 밀도 |
| ---: | ---: | ---: | --- | --- |
| 1920px | 56px | 1 | 182/132/88/176/132/132/314 | label 포함 44px |
| 1280px | 56px | 1 | 182/132/88/176/132/132/314 | label 포함 44px |
| 1024px | 99px | 2 | 182/132/88/176/132/132/314 | label 포함 44px |
| 976, 883, 768px | 53px | 2 | 144/108/72/144/108/108/288 | icon-only 36px |
| 412, 375px | 77px | 3 | 144/108/72/144/108/108/288 | icon-only 36px |

separator와 좌우 padding을 포함한 desktop 밀도 콘텐츠 폭은 1179px, icon-only 콘텐츠 폭은 988px이다.
현재는 콘텐츠보다 `1023px`과 `767px` media query가 label·폭을 바꾸고 flex wrap이 1~3행을 만든다.

## 3. 고정한 구조·상태 계약

| 영역 | 보존·추가 계약 |
| --- | --- |
| outer | `#icon-toolbar` ID, #6115 visibility, theme와 editor focus 예외 유지 |
| 명령 | 기존 `.tb-group`·`.tb-sep`·button DOM 순서와 identity 유지 |
| 밀도 | 모든 너비에서 label 포함 44px button과 56px 높이 유지 |
| viewport | native horizontal wheel·touch pan, 시각 scrollbar만 숨김 |
| nav | overflow 때만 표시, group 경계 이동, 양 끝 native disabled |
| mode | 머리말/꼬리말·주석·개체 group 변경 뒤 시작 위치와 overflow 재계산 |
| keyboard | 기존 Tab 순서 유지, offscreen focus를 inline nearest로 노출 |

viewport는 device breakpoint가 아니라 실제 `scrollWidth > clientWidth`로 nav 필요 여부를 판정한다. 따라서
향후 group 추가·삭제이나 번역 폭 변화도 별도 breakpoint 없이 같은 정책을 따른다.

## 4. Stage 1 종료 판정

- [x] 현행 1920~375px의 높이·행·밀도를 동일 방식으로 계측했다.
- [x] desktop 밀도 콘텐츠 폭 1179px과 한 줄 높이 56px을 고정했다.
- [x] 외부 ID, 명령 DOM·순서, mode·visibility 보존 계약을 확정했다.
- [x] 동적 overflow, group 경계 이동, native scroll과 접근성 계약을 확정했다.
- [x] `rhwp-studio` 제품 source·test·E2E 변경은 0건이다.

Stage 1은 완료했다. 다음 단계는 기존 group을 단일 track 안에 유지하면서 scroll viewport와 좌우 이동
controller를 구현하고 source/browser 계약을 갱신하는 Stage 2다.
