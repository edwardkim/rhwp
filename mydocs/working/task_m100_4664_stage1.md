# 처리 결과 — [운동장] 테마파크 (#4664)

## 무엇을 했나

운동장(gym)에 놀이공원을 입혔다. 방문 동기를 만드는 장식과, 실제로 탈 수 있는
양극단 어트랙션(입문·보스), 그리고 친구를 부르는 초대 메커니즘을 더했다.

| 갈래 | 산출물 | 성격 |
|---|---|---|
| 🐉 보스존 | `gym/packs/expert-challenges/` (XC01~XC05) | tier 4~5 고난도, 한 단만 틀려도 판정 막힘 |
| 🎠 입문존 | `gym/packs/casual-rides/` (CR01~CR04) | tier 1, 부모님·친구도 성공 |
| 지도 | `gym/PARK.md` | 테마존 mermaid 지도 |
| 휴게실 | `gym/tutorial/README.md` | 첫 방문 5분 걷기 안내 |
| 초대 | `gym/INVITE.md` + `leaderboard.py invite` | 외부 참가자 등재 + 판 지문 확인 |
| 코스 | `gym/profiles/{family,boss}.json` | 가족·보스 프로파일 |

- **12 pack · 과제 100건 · 만점 221** (기존 10 pack·91·194 → +2 pack·+9·+27).
- 티어 상한 3→5 확장(`gym/core/schema.py`): 입문(1)부터 보스(5)까지.

## 보스 어트랙션 (검증 사다리를 한 체인으로)

| 과제 | tier | 완주 체인 | 최종 판정 |
|---|---|---|---|
| XC01 사다리 완주 | 5 | keygen→sign→anchor→settle→policy | 적합성 **L5** conformant |
| XC02 오염 리콜 드릴 | 5 | 2세대 계보(a→b) | recall-scope: unaffected 0 · affected ≥2 |
| XC03 정산 완주 | 4 | propose→record→verify | 캡슐·게이트·원장·워크오더 4관문 |
| XC04 계보 완주 | 4 | 3세대 사슬(a→b→c) | lineage valid · depth 3 |
| XC05 감사 표준 발급 | 5 | sign→anchor→audit-report | 적합성 **L3** + 리포트 발급 |

## 검증 (기준 풀이 왕복)

모든 신규 과제는 `build_baseline.py` 왕복으로 "풀 수 있음"이 실측됐다.

```
기준 풀이 실행: 성공 9 · 실패 0
expert-challenges  23/23  (5/5 과제)
casual-rides        4/4  (4/4 과제)
```

전 12 pack 회귀: **218/221** (core-cli 3점 갭은 1부 유산, 무관). 계약 테스트
**58 passed, 1 skipped**(신규 가드 4: 티어 범위·양극단 존재·resolve 다중 치환·판 지문).

## 트랩 (정직 기록)

1. **`(2,0)` 셀은 편집 불가.** table-001.hwp 는 rows 0~18 이 있지만 `set_cell
   (2,0)` 은 원본에서도 engine exit 2 로 막힌다(병합/헤더 셀). XC04 의 3세대를
   `(0,0)·(1,0)·(0,1)` 로 잡아 우회. → 좌표는 실측으로만 고른다.
2. **`resolve()` 가 첫 `{sub:}` 만 치환하던 결함.** 다세대 계획서는 input·output
   을 모두 `{sub:}` 로 가리키는데, 첫 하나만 바뀌어 출력이 리터럴 이름
   (`{sub:o2.hwp}`)으로 저장되고 다음 세대가 입력을 잃었다. 전부 치환하도록
   고침(`build_baseline.py`) + 가드 테스트.
3. **`replay` 는 문서를 안 쓴다.** 영수증/캡슐만 낸다 — 시각 증거용 산출 문서는
   `run` 으로 생성해야 한다(전/후 렌더 파이프라인에서 확인).
4. **`convert` 는 ImageMagick 이 아니다.** 이 PC 의 `convert` 는 Windows 디스크
   변환 유틸이라 SVG→PNG 가 안 된다. `~/.cargo/bin/resvg` 로 래스터화.
   `export-png` 은 native-skia 미빌드라 불가 → `export-svg`+resvg 경로.
5. **`export-svg -o x.svg` 는 폴더를 만든다.** 페이지별 SVG 를 `x.svg/x.svg` 로
   넣는다 — 중첩 경로를 가리켜야 한다.

## 정직 조항

테마는 장식이다 — 존·티켓·전당은 은유일 뿐, 채점과 판정 논리(pack 별 점수
보존·라이브 오라클·unavailable 구분)는 그대로다. 보스 과제도 예외 없이 기준
풀이 왕복을 통과해야 등재됐다.

## 시각 증거

`mydocs/report/edit_demo_4664/boss_xc01_before_after.png` — 보스 XC01 이 실제로
`table-001.hwp` 의 표 (0,0) 을 '사다리완주'로 편집·서명한 전/후(rhwp export-svg
렌더). 하단에 L5 conformant · 왕복 27/27 판정.
