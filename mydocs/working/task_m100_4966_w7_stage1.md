---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-1 source inventory와 분리 전 기준선

## 1. 판정

Stage W7-1은 통과했다. runtime table은 교체하지 않았으며, 현재 source의 W1 candidate identity·순서와
W6 metric 계보, Studio 실제 lookup/plan 결과를 W7 migration 전 기준선으로 동결했다.

단, 계획서의 최초 `unknown` 처리 문구에는 수정이 필요했다. W1의 unknown 44개는 비활성 후보가 아니라
현재 실행 중인 layout-metric 규칙이다. 이를 projection에서 제거하면 행동 보존을 위반하므로 다음
계약으로 정정했다.

- 기존 active unknown 43개 metric alias: 원래 layout-metric 결정면의 legacy-preservation으로 유지
- unknown measurement predicate 1개: hand-written runtime reference로 유지
- 다른 relation·decision plane·backend로 의미 승격 금지
- 신규 unknown의 실행 규칙 추가 금지

이 정정은 scope 확대가 아니라 W7-I04·I06의 현재 동작 보존을 위한 필수 조건이다.

## 2. source inventory

현재 checkout에서 W1 collector를 다시 실행한 결과는 다음과 같다.

| 항목 | 결과 |
| --- | ---: |
| source boundary | 30 |
| rule candidate | 1,352 |
| W1 ledger rule | 1,507 |
| W1 대비 candidate 추가 | 0 |
| W1 대비 candidate 삭제 | 0 |
| candidate identity·순서 drift | 0 |
| ledger와 연결되지 않은 candidate | 0 |
| unclassified boundary | 0 |
| extractor가 인식하지 못한 mapping block | 0 |

30개 boundary는 `projection-input` 또는 `reference-only`로 전부 disposition했다. metric table·lookup,
measurement predicate, browser capability detection, native replay와 asset/license authority는 W7의 유한
projection으로 옮기지 않고 기존 hand-written owner 또는 W6 authority에 남긴다.

## 3. projection 기준선

| projection | rule | 핵심 관계 | SHA-256 |
| --- | ---: | --- | --- |
| Rust layout-name | 171 | style-fallback | `ea98fd563d6ffc3c8244f103647607369ada7af1e9046d7c7d9bf49fb90d1b4b` |
| Rust layout-metric | 67 | metric-surrogate 24, active unknown 43 | `c8100b8318a4a4607e95515327dcaa66811bc18b3ca55ef0b95cd5a6c109cd62` |
| Canvas2D paint | 281 | paint, successor, document, generic ordering | `7fcb86fa9ec007c0c7e9578c1f983a879b40d0d80d52551f12406670b7543023` |
| webfont supply | 153 | Canvas2D CSS supply | `00902102a7925f148eead227bdc361cf331bb8c7f5710b703480cb9d93075f01` |
| CanvasKit SFNT | 158 | SFNT supply 157, byte capability 1 | `dec6352305f23ad6ab3181b06b51790a19341a6654a8e0ac284bff2065705c9d` |

projection bundle SHA-256은
`0b6d3b50586099063df0ef4c3039dd572e75aa0343b00dc4c01eac0cc42d7ee9`다.

Canvas2D 기준선은 `SUBST_TABLES`만 세지 않았다. display chain, Canvas patch와 W1에서 byte evidence가
확인된 정부상징 official-successor도 별도 관계로 포함했다. 동일 source/condition의 복수 target과
first-match를 정렬하거나 dedupe하지 않고 W1 `order`를 보존했다.

## 4. W6 metric anchor

W6 600개 항목은 숫자를 W7 registry에 복사하지 않고 다음 anchor projection으로 동결했다.

| 항목 | 결과 |
| --- | --- |
| entry/current index | 600/600, `0..599` 연속 |
| anchor SHA-256 | `aa1f0e7cde72e4ed50ac571639e4ee05291f8bdcc9c1cd0cba0a401d3c5fead7` |
| composition | `d4cdac86b3c6ee55d8b1aa921d662f1fc1241c2809cb9c8ffe991d56a045e69a` |
| metric data | `025812eac4bad179c5b87e23b15fdf08a4e4fb3f19a6e453738e03110a140bcf` |
| exhaustive width | `2cd1389a14401f6488041af3c54ff0ba5e982d944acd0b5bb56147056e3a7d1b` |
| exhaustive lookup | `bb3008f9dc379bd580119a6a658388732e94358db2039dbb02d78c28ec992fdf` |

따라서 Stage W7-2 registry는 metric target을 W6 `entryId`로 참조할 수 있지만, historical generated
595개와 measured overlay 5개의 값을 다시 소유할 수 없다.

## 5. Studio 실제 실행 기준선

정적 TypeScript source 추출과 별도로 현행 함수를 실제 실행했다. fake DOM/FontFace는 URL을
다운로드하지 않고 등록 tuple만 기록한다.

| 실행 projection | 분모 | SHA-256 |
| --- | ---: | --- |
| substitution resolve/display candidate | 265 | `013037ce38c8fb332357fc1a1e8bbe48f59bd0d2a8d57c138afdb136ef559024` |
| 정부상징 successor availability 조합 | 65 | `58f20a5526166623a73c6a661bceac8e7b6808597ef6e477a110c73a762bee79` |
| generic/KoPub/mono/serif/sans display probe | 8 | `d03d324fe84bdc646d6377df83f8ffb2457e30a6f97765db6290805c64d09af6` |
| registered font 순서 | 153 | `36da093ae28426cdedb750ddcd7e85dd672a2124d01233ba262051220b0b39e6` |
| webfont supply snapshot | 153 | `1bb8f0b160e46aae28edcc2607a3ee3b2061e5e77fa946db4797a0c927976a92` |
| FontFace CSS/load tuple | 153 | `a10b365a27c71e7852fe3e537ac19acd5299694c1e5027045feac7877ddd7071` |
| CanvasKit online/offline plan | 153 | `137b1eb63ddcf357278511c2576e1f62e7da50f4dae413c534e3f53c4fc86c9b` |

정부상징은 두 legacy spelling 각각에 successor 5개의 모든 availability 부분집합 32개를 적용하고,
일반 font에 successor가 새지 않는 negative control 1개를 더해 65건으로 고정했다.

## 6. 구현 산출물

- `scripts/font_rule_projection_baseline.mjs`
  - 현재 W1 candidate 재수집과 기존 snapshot 전건 비교
  - 30개 boundary route 폐합
  - W1 rule·W6 entry join과 backend projection/hash 생성
  - active unknown 삭제, 순서·hash drift와 unclassified boundary fail-closed
- `scripts/font_rule_runtime_snapshot.mjs`
  - Studio substitution, webfont 등록과 CanvasKit plan의 실제 실행 snapshot
  - 외부 font를 다운로드하지 않는 fake DOM/FontFace harness
- `scripts/tests/font_rule_projection_baseline.test.mjs`
  - 결정론·분모·unknown·순서·metric anchor mutation 6개 contract
- `mydocs/tech/investigations/issue-4966/font_rule_projection_baseline.json`
  - Stage W7-1 pre-migration snapshot

## 7. 검증 결과

| 검증 | 결과 |
| --- | --- |
| W7 baseline generate/check | 통과 |
| W7 focused contract | 6/6 통과 |
| W1 boundary/candidate/ledger contract | 통과 |
| W6 metric lineage contract | 통과 |
| W1·W6 선행 Node 전체 | 36/36 통과 |
| Studio substitution·supply·CanvasKit·trace·local font | 34/34 통과 |
| Rust style resolver | 29/29 통과 |
| Rust font metric | 9/9 통과 |
| Markdown link·diff check | 최종 commit 전에 재검사 |

mutation contract는 Rust projection 순서 교환, W6 metric anchor 순서 교환과 active-unknown 삭제가
실제로 실패하는지 확인했다.

## 8. Stage W7-2 인계

Stage W7-2는 이 기준선에서 다음만 수행한다.

1. 한 행이 하나의 relation·decision plane만 갖는 canonical registry schema를 정의한다.
2. 기존 active unknown을 `legacy-preservation`으로 표시하되 의미를 재판정하지 않는다.
3. W1 rule과 W6 entry를 one-time migration manifest로 연결한다.
4. relation/plane/backend allowlist, 중복·order·anchor와 신규 unknown negative contract를 구현한다.
5. 아직 Rust/TypeScript runtime consumer를 registry로 전환하지 않는다.

Stage W7-2 진입은 메인테이너의 별도 승인을 받는다.
