# Stage 2 처리 결과 — #6138 기본 도구 상자 한 줄 그룹 스크롤 구현

- **이슈**: [#6138](https://github.com/edwardkim/rhwp/issues/6138)
- **기준**: `upstream/devel@1011a8947`
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **구현일**: 2026-08-26 KST
- **상태**: Stage 2 완료, Stage 3 통합 검증 대기

## 1. 단일 DOM·한 줄 구조

외부 `#icon-toolbar` ID는 그대로 두고 내부에 이전 버튼, `.tb-scroll-viewport`, `.tb-scroll-track`, 다음
버튼만 추가했다. 기존 `.tb-group`, `.tb-sep`, button·split menu는 같은 순서와 identity로 track 안에
한 번만 존재한다.

CSS는 root를 `flex-wrap:nowrap`, 56px 고정 높이로 바꾸고 1023px 이하의 label 숨김·36px button 축소와
mobile wrap 규칙을 제거했다. viewport는 native `overflow-x:auto`, `touch-action:pan-x`를 사용하고 시각
scrollbar만 숨긴다. 결과적으로 모든 너비에서 label 포함 44px desktop 밀도가 유지된다.

## 2. 동적 overflow·group 이동

`IconToolbarScroller`는 다음 상태만 소유한다.

- root·viewport·track `ResizeObserver`로 실제 `scrollWidth`와 nav 없는 가용 폭 비교
- 직계 group/sep의 `style`·`hidden`만 보는 `MutationObserver`로 mode 변경 감지
- 가시 `.tb-group`의 `offsetLeft`를 이용한 다음·이전 경계 이동
- ArrowLeft/ArrowRight·Home·End keyboard 이동
- 수평 wheel·touch의 native scroll과 offscreen command focus 보정
- 시작·끝 native disabled와 overflow가 없을 때 nav `hidden`

nav가 이미 보이는 상태에서도 nav 폭 48px을 가용 폭에 다시 더해 판정하므로 넓어질 때 버튼이 계속 남는
resize hysteresis가 없다. mode group이 바뀌면 시작 위치로 복귀한 뒤 overflow를 다시 계산한다.

## 3. 기존 기능 보존 보정

가로 scroll viewport는 내부 absolute popup을 자를 수 있다. 기존 찾기 split menu는 명령 DOM과 listener를
그대로 유지하고 열린 panel만 도형 picker와 같은 viewport 기준 `position:fixed` 좌표로 배치했다. 375px에서
menu가 toolbar 아래와 화면 좌우·하단 안에 완전히 표시되는지 E2E로 고정했다. toolbar를 스크롤하면 열린
split menu와 `aria-expanded`를 함께 닫는다.

`main.ts`의 머리말/꼬리말·주석 mode selector는 새 single track 직계 group을 가리키도록 범위만 갱신했다.
#6115의 visibility는 외부 ID가 그대로이므로 nav·viewport를 포함한 전체 shell이 함께 숨고 복귀한다.

## 4. 브라우저 실측

| viewport | toolbar | group 행 | label | nav | 내부 viewport |
| ---: | --- | ---: | --- | --- | ---: |
| 1920px | 56px | 1 | 표시 | 숨김 | 1904/1904px |
| 1280px | 56px | 1 | 표시 | 숨김 | 1264/1264px |
| 1024px | 56px | 1 | 표시 | 표시 | 1219/960px |
| 976px | 56px | 1 | 표시 | 표시 | 1219/912px |
| 883px | 56px | 1 | 표시 | 표시 | 1219/819px |
| 768px | 56px | 1 | 표시 | 표시 | 1219/704px |
| 412px | 56px | 1 | 표시 | 표시 | 1219/348px |
| 375px | 56px | 1 | 표시 | 표시 | 1219/311px |

내부 viewport만 overflow하며 각 viewport의 document root와 외부 toolbar `scrollWidth <= clientWidth`다.

## 5. 검증 결과

- focused source/controller/theme 계약: 33 passed
- 전체 Studio test: 1146 passed, 0 failed, 1 skipped
- TypeScript `npx tsc --noEmit`: 통과
- Studio production build: 통과(230 modules transformed)
- responsive + theme + #6118 통합 E2E: 609 passed, 0 failed
- 실제 상호작용: group next, Home/End, horizontal wheel, offscreen focus, split menu, header/footer mode,
  toolbox hidden/shown 모두 통과
- theme: default/flat/oldschool × light/dark에서 56px, 경계와 nav contrast 4.10 이상
- page·style bar·toolbar 외부 가로 overflow: 0

## 6. Stage 2 종료 판정

- [x] 1920~375px에서 56px 단일 행과 desktop label 밀도를 유지한다.
- [x] nav는 실제 overflow 여부에 따라 숨김/표시된다.
- [x] group 경계·native scroll·keyboard·focus로 양 끝까지 도달한다.
- [x] 시작·끝 disabled와 accessible label을 제공한다.
- [x] mode·resize·#6115 visibility 뒤 상태를 다시 계산한다.
- [x] 기존 command DOM·순서·listener와 split menu를 보존한다.
- [x] #6118 서식 바 1·2행·더보기 계약과 동시 E2E를 통과한다.

Stage 2는 완료했다. 다음 단계는 전체 Studio test/build, 대표 screenshot과 세 skin light/dark 시각 검토,
문서·format 게이트, #6118+#6138 최종 통합 보고를 준비하는 Stage 3다.
