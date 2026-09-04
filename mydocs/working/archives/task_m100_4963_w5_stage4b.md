---
kind: working-note
status: completed
issue: 4963
stage: W5-4B
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 4B — disposable Hancom canary

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../../plans/archives/task_m100_4963.md)
- **선행 계약**: [`task_m100_4963_w5_stage4a.md`](task_m100_4963_w5_stage4a.md)
- **단계 상태**: controlled run·acceptance profile·공개 projection·메인테이너 판정 완료

## 1. 환경 attestation

외부 Hyper-V control plane에서 Standard checkpoint를 실제 복원했다. 공개 기록에는 VM·checkpoint 이름과
절대 경로를 넣지 않고 identity digest만 남긴다.

| 항목 | 관찰값 |
| --- | --- |
| VM identity SHA-256 | `466349124f6411dc1460697f8c2959256c28d682daf5f48376d05c83a1f5346d` |
| checkpoint identity SHA-256 | `a6fcfda6066c3e5545e517f5c3a8fd35f5eea64ea1473016ddb157373d72daab` |
| baseline font manifest SHA-256 | `796ba7d2a9759c63d71098c5d3182af2d1a653cc096c332d8e987347a45700fb` |
| unrelated projection SHA-256 | `6d0b71d5baf3ebc09d24c2698b096326435f4cd1f99cf75d63f939910172026e` |
| baseline entry 수 | 562 |
| Windows culture / UI / system locale | `ko-KR` / `en-US` / `en-US` |
| Hancom automation version | `11, 0, 0, 2129` |

복원 뒤 interactive desktop 1개와 HWP process 0개를 확인했다. 각 실행은 새 HWP process를 사용했고,
실행 뒤 process 0개와 baseline manifest 복구를 재확인했다. 관리 font 밖 projection 변화, private corpus
접근, 원격 upload, 한컴 bundle 변경은 모두 0건이다.

manifest helper는 locale-sensitive `Sort-Object sourceKind, identity` 대신 UTF-8 byte의 ordinal key로
정렬해야 반복 digest가 안정적이었다. 동일 562개 entry set에서 HFT 한 항목의 순서만 달라져 digest가
흔들리는 negative observation을 통해 이를 수정했다.

한컴 업데이트 뒤에는 최초 GUI 실행이 끝나지 않은 checkpoint에서 automation이 2분 timeout됐다. 작업과
HWP process가 0개로 정리된 것을 확인하고 update checkpoint를 복원한 다음, GUI 최초 실행·정상 종료를
완료한 후손 checkpoint를 별도 생성했다. 새 automation 기준선은 다음과 같다.

| 항목 | updated-base 관찰값 |
| --- | --- |
| checkpoint identity SHA-256 | `7961e64697b76e8985d918abcf52a8fa0eca1d7cb5d5d46bea0af7f926b4dbe8` |
| baseline font manifest SHA-256 | `3bcd379d1f7fc217aad47a0b44b952d993c86ebbfabf46009386e4b3de768b40` |
| unrelated projection SHA-256 | `437a36e513cce9d2909d904f3d07d2341051cc017e21be9ec6d35bbb9d87bc78` |
| baseline entry 수 | 575 |
| HWP executable SHA-256 | `7f00961398802c41620f5ef32fa2d2a26f7ff71f172723be36c660ea86a72bce` |
| Hancom automation version | `11, 0, 0, 9136` |

updated-base는 update checkpoint의 후손임을 Hyper-V snapshot id 계보로 확인했다. manifest와 projection은
연속 두 번 동일했고, 최초 실행 뒤 rank 7 canary가 timeout 없이 완료됐다. 아래 rank 1 결과는 2129
탐색 기준선, rank 7 결과는 9136 updated-base이므로 후보 간 수치를 직접 비교하지 않는다.

## 2. rank 1 문체부 바탕체

입력은 세 상태 모두 `deb4566e2be357959c22db06460f485a12bb61521b22179495fcb1cda79ca511`로
동일하다. exact font SHA-256은 `d10509215d923fef07c1f2dffe8ebf55cbca706476559a861dff6f7cf969ff44`,
fixture-declared substFont SHA-256은
`e3ee21a86b6a6728c567a95aaebd8883480f27ce4f230207b0d7266b5cb3fb18`이다.

두 바탕 글꼴의 역할을 섞지 않는다.

| 역할 | HWPX 이름 | 준비한 TTF의 영문 name | 관계 |
| --- | --- | --- | --- |
| 원문 exact | `문체부 바탕체` | `MBatang` | 같은 `MT.TTF`의 localized SFNT name 후보 |
| 문서 지정 subst | `KoPubWorld바탕체 Light` | `KoPubWorldBatang Light` | fixture가 `<hh:substFont>`로 선언한 별도 글꼴 |

KoPubWorld 바탕은 문체부 바탕과 같은 글꼴·identity alias·official successor가 아니다. 이번 synthetic
fixture가 대체 후보로 명시했기 때문에 `document-substitution` 관계만 가진다.

| 물리 상태 | 관리 font | face probe | PDF font | line / span / glyph |
| --- | --- | --- | --- | --- |
| exact-only | exact 1개 | `문체부 바탕체` → `함초롬바탕`, `MBatang` exact | `HCRBatang-Bold`, `MBatang` | 30 / 992 / 1,556 |
| subst-only | subst 1개 | 한글 subst명 실패, `KoPubWorldBatang Light` exact | `HCRBatang-Bold` | 30 / 304 / 1,556 |
| none-related | 0개 | 세 probe 모두 `함초롬바탕` | `HCRBatang-Bold` | 30 / 304 / 1,556 |

첫 divergence는 단순 COM selection readback이 아니다. exact-only의 빈 문서 probe는 한글 요청명을
fallback으로 돌려줬지만, 같은 HWPX의 PDF export는 설치한 SFNT의 영문 name `MBatang` subset을 실제로
포함했다. 따라서 `document face selectable`과 `export-selected bytes`는 서로 다른 feature로 기록해야 한다.

반대로 subst-only에서 영문 KoPubWorld face는 선택 가능했지만 fixture가 선언한 한글 `substFont`와 자동
alias 연결되지 않았다. subst-only와 none-related는 생성시각 때문에 raw PDF SHA-256은 달랐으나 font,
page, line, span, glyph, advance projection은 byte-exact하게 같았고 projection SHA-256은
`04afd08d1000dc38185e4d1f04df011b42569f245dd7884ce9cec68be6ef06b1`이었다.

updated-base에서 같은 세 상태를 단일 빈 문서 probe runner로 반복했다. exact-only는 다시 `MBatang`,
subst-only와 none-related는 다시 `HCRBatang-Bold`를 사용했다.

| 물리 상태 | PDF font | line / span / glyph | updated projection SHA-256 |
| --- | --- | --- | --- |
| exact-only | `HCRBatang-Bold`, `MBatang` | 30 / 992 / 1,556 | `eb44a80…a0042` |
| subst-only | `HCRBatang-Bold` | 30 / 304 / 1,556 | `c5b00c87…c5f36` |
| none-related | `HCRBatang-Bold` | 30 / 304 / 1,556 | `c5b00c87…c5f36` |

updated-base 내부의 첫 의미 있는 divergence는 glyph observation index 18, U+AC00 `가`다. exact-only는
`MBatang`의 normalized advance 1.0과 PDF advance 8.012243을, none-related는 `HCRBatang-Bold`의
normalized advance 0.97과 PDF advance 7.774934를 기록했다. 이는 selection→advance 단계에서 이미
갈라지며 최종 30줄·1쪽이 우연히 같아도 같은 조판이라고 볼 수 없음을 뜻한다.

2129와 9136 exact-only는 font set·30줄·992 span·1,556 glyph가 같지만, glyph index 1381의 fallback
영문 `F`부터 PDF position과 advance가 달랐다. 그러므로 build 번호로 정책을 분기하지는 않되 Oracle
environment identity는 결과 해석에 반드시 포함한다.

## 3. rank 13 휴먼명조

`none-related` 기준선에서 관리 TTF가 0개인데도 `휴먼명조`가 TTF type 1로 exact readback됐다. 따라서
`exact-removed`와 `all-related-fonts-missing`은 성공 관찰이 아니라 계약대로
`blocked-immutable-or-unmanaged-font`다. 한컴 bundled HFT 또는 관리 범위 밖 font를 손상시키지 않고는
missing 상태를 격리할 수 없다.

이 PDF는 유효했고 `qpdf --check`를 통과했지만, 한컴 subset font 이름의 legacy Korean byte와 UTF-8 XML이
섞인 `mutool trace`를 만들었다. 관찰기는 invalid name byte만 U+FFFD로 대체하고 구조·glyph·advance를
보존하도록 수정했다. 그 결과 1쪽, 30줄, 304 span, 1,556 glyph를 회수했으며 Stage W5-2 회귀 8건이
통과했다. 원시 font name을 복원했다고 주장하지 않는다.

## 4. rank 7 KoPubWorld돋움체 Light

updated-base에서 동일 입력
`1cc8062c6fd0da39cfddc4182115226717516d4250e693b43596293374236f9e`로 세 상태를 실행했다. exact
font SHA-256은 `069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f`이고 substFont는 rank 1과
같은 KoPubWorld 바탕 Light bytes다.

| 물리 상태 | 관리 font | face probe | PDF font | line / span / glyph | 조판 projection SHA-256 |
| --- | --- | --- | --- | --- | --- |
| exact-only | exact 1개 | 한글 요청 실패, `KoPubWorldDotum Light` exact | `KoPubWorldDotumLight` | 30 / 304 / 1,556 | `726ffb04…81e4b` |
| subst-only | subst 1개 | 한글 subst명 실패, `KoPubWorldBatang Light` exact | `HCRBatang-Bold` | 30 / 304 / 1,556 | `c5b00c87…c5f36` |
| none-related | 0개 | 모든 probe 실패 | `HCRBatang-Bold` | 30 / 304 / 1,556 | `c5b00c87…c5f36` |

rank 7도 COM의 한글 face readback과 export-selected font가 갈라졌다. exact-only PDF는 설치한 공식
KoPubWorld 돋움 bytes를 사용했지만, 한글 `substFont`는 설치된 영문 KoPubWorld 바탕 face로 이어지지
않았다. subst-only와 none-related의 font·page·line·span·glyph·advance projection은 byte-exact했다.

## 5. 현재 판정과 다음 게이트

1. font bytes 설치 여부, COM face 선택 가능 여부, HWPX export의 실제 subset 선택을 분리한다.
2. localized SFNT name은 identity 후보일 뿐 한컴의 document alias로 자동 승격하지 않는다.
3. rank 13은 immutable/unmanaged source 때문에 missing 비교를 중단한다.
4. rank 7의 세 상태는 updated-base에서 완료됐다.
5. rank 1 updated-base 반복에서 첫 divergence가 유지됐다.
6. raw local evidence는 rank 1·7의 8개 path-free acceptance profile, 두 ladder, disposable 환경
   attestation과 공개 projection으로 정규화했다. profile·ladder validator와 file-hash 연결 검사를
   통과했다.
7. subst-only와 none-related는 조판 projection이 같았다. 이는 KoPub 바탕을 실제로 사용했다는 뜻이
   아니라, 설치된 영문 KoPub face가 fixture의 한글 `substFont`와 연결되지 않아 둘 다 HCR fallback으로
   귀결됐다는 관찰이다.
8. 메인테이너가 이 acceptance projection을 승인한 뒤 W5-5A의 17개 queue 판정으로 이관했다.

공개 산출물에는 raw VM/checkpoint name, 절대 경로, font bytes, private corpus 문서나 식별자를 넣지 않는다.
