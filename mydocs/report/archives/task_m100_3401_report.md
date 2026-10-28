# v0.8.1 릴리즈 최종 보고서

Issue: #3401
브랜치: `task/3401-release-v0.8.1`, `task/3401-changelog-3402`
태그: `v0.8.1` = `1dbf024fd`
릴리즈 범위: `v0.8.0..v0.8.1` **95커밋**

## 1. 결과

배포 대상 전부가 0.8.1 로 반영됐다.

| 대상 | 방식 | 결과 |
|---|---|---|
| GitHub Pages | 자동(main push) | success, 사이트 HTTP 200 |
| npm `@rhwp/core` | 자동(Release) | **0.8.1** |
| npm `@rhwp/editor` | 자동(Release) | **0.8.1** |
| VS Code Marketplace | 자동(Release) | **0.8.1** |
| Open VSX | 자동(Release) | **0.8.1** |
| Chrome / Edge / Firefox | 수동 업로드 | **제출 완료**(메인테이너, 2026-07-26) |

VS Code Marketplace 와 Open VSX 는 Release 직후 조회에서 0.8.0 으로 나왔다. 워크플로의
publish step 이 모두 success 였으므로 인덱싱 지연으로 판단하고 폴링했고, 3회차(약 2분)에
양쪽 모두 0.8.1 로 확인했다. **step success 를 곧바로 배포 완료로 단정하지 않고 실제
레지스트리 조회로 확인한 것이 판정 근거다.**

## 2. 릴리즈 범위

| 유형 | 주요 항목 |
|---|---|
| 렌더 정정 | 바탕쪽 머리말 개체 여백 이탈(#3402), HWP3 1쪽 글맵시 내장 OLE 렌더 + WMF 구멍 소실(#3363), 문단 테두리 '선 없음' 오렌더(#3303) |
| CLI 계약 정합 | export 계열 인자 파싱 통일(#3359 #3349), search 절단 가시화(#3353), thumbnail 종료 코드(#3366), capabilities 자기서술(#3357 #3329), ingest 미지 필드 거부(#3358), build-from-ingest 무테두리(#3355) |
| CLI 신규 | `edit fill-fields`(#3329)·`replace-text`(#3373)·`set-cell`(#3381), batch 축 확장(#3346), 기안문 표준 서식(#3372), 정답지 비교 하네스(#3389) |
| studio | 스타일 undo 라우팅(#3387), dev 전용 fetch 가드(#3348) |
| 의존성 | base64 0.23.0, snafu 0.9.2, GitHub Actions 7종 major |

신규 기능 6건이 포함되나 전부 CLI 도구 계층이고 라이브러리 공개 API 변경이 없어 PATCH 로
확정했다(작업지시자 판단).

## 3. 계획 대비 차이

수행계획서는 6단계였으나 실제로는 **선행 처리 2건과 중간 보완 1건이 추가**됐다.

| 계획 | 실제 | 사유 |
|---|---|---|
| — | dependabot 연작 선행 처리 | 작업지시자 지시. 개별 PR 9건이 열려 있어 릴리즈에 포함시킴 |
| — | lpaiu-cs #3400 통합 | 작업지시자 지시로 범위 추가 |
| 2단계에서 CHANGELOG 확정 | 3.5단계 보완 필요 | #3402 가 CI 대기 중 유입 |
| 버전 10파일 | **9파일** | chrome/edge 가 한 코드베이스 공유 — 계획 시 중복 계산 |

### 기준선이 세 차례 바뀌었다

| 시점 | 기준선 | 사유 |
|---|---|---|
| 착수 | `0ff167a44` | 최초 devel |
| 1차 | `ce2156dad` | dependabot 통합 2건 |
| 2차 | `6814bf431` | #3400 포함 |
| 확정 | `ace187d52` | #3409·#3402 유입 + CHANGELOG 보완 |

**기준선이 바뀔 때마다 이전 검증 결과를 재사용하지 않고 재실행했다.** dependabot 과 활발한
PR merge 가 공존하는 저장소에서 릴리즈 기준선은 고정 대상이 아니라 계속 확인해야 하는
값이라는 점이 이번 릴리즈의 실무 교훈이다.

## 4. 검증

| 항목 | 결과 |
|---|---|
| `cargo test --profile release-test --tests` | **4159 passed / 0 failed** |
| `cargo clippy --all-targets -- -D warnings` | 경고 0 |
| `cargo fmt --check` / `git diff --check` | 통과 |
| Docker WASM 빌드 | 성공 (4m18s, 7.2MB) |
| studio `npm run build` | 성공 |
| studio E2E | 10개 스위트 중 **9 통과** |
| PR #3417 CI(devel) | SUCCESS 21 / SKIPPED 1 |
| PR #3418 CI(main) | SUCCESS 40 / SKIPPED 2 |
| `web-ext lint`(Firefox) | **errors 0**, warnings 7 |

### main 통합 무결성

v0.8.0 에서 ancestry 소실로 문서 20파일 충돌이 발생했던 이력이 있어 이번에는 사전 확인과
사후 검증을 모두 수행했다.

- 사전: merge-base `8bb8f277d` 이후 **main 독자 변경 0파일** — 충돌 없음을 확인하고 PR 생성.
- merge 방식: **merge commit**(`--admin`). 부모 2개(`60911e822`, `ace187d52`) 확인.
- 사후: main 트리 == devel 트리 (`15bd90e10` 동일) — 내용 무변경 증명.
- 사후: `devel` 이 `main` 의 조상 — **ancestry 보존 확인**.

`--admin` 은 main 브랜치 보호의 `required_reviews: 1` 을 우회하기 위해 작업지시자 승인 뒤
사용했다. CI 는 우회하지 않았다(SUCCESS 40 확인 후 merge).

## 5. 확장 배포 산출물

빌드 기준: main = `v0.8.1` 태그 트리, `npm ci` 선행.

| 파일 | 크기 | 검증 |
|---|---|---|
| `rhwp-chrome/rhwp-chrome-0.8.1.zip` | 30.0MB | manifest 0.8.1 |
| `rhwp-chrome/rhwp-edge-0.8.1.zip` | 30.0MB | chrome 과 md5 동일(`0e483204…`) |
| `rhwp-firefox/rhwp-firefox-0.8.1.zip` | 30.0MB | web-ext lint errors 0 |
| `rhwp-firefox/rhwp-source-0.8.1-amo.zip` | 26.9MB | 200MB 제한 충족 |

위생 검증:

- dist 에 `.env`·token·pem 없음.
- source zip 금지 경로 7종 전부 미포함 — `samples/`, `pdf-large/`, `output/`, `target/`,
  `node_modules/`, 확장 `dist/`, `rhwp-studio/public/samples/`.
- source zip 필수 파일 6종 포함 — `Cargo.toml`, `Cargo.lock`, `LICENSE`,
  `THIRD_PARTY_LICENSES.md`, 확장 manifest, `src/`.

제출 문서 4종은 `mydocs/feedback/` 에 있다 — `chrome-0.8.1_{kor,eng}.md`,
`edge-0.8.1_reviewer_notes.md`, `firefox-0.8.1_amo_notes.md`. 권한·외부 endpoint 변경이
없음을 각 문서에 명시했다.

## 6. 미해결

### #3412 — studio E2E issue-2214 실패

`hwp run 1 after-56-sync` 체크포인트에서 누적 `wasmFlush` 가 기대값 0 대신 2. 2회 재현했다.
#2214 가 고정한 "페이지 로컬 리페인트 중 WASM flush 없음" 계약 위반이다.

**회귀 여부는 미확정이다.** v0.8.0(`main`) 기준 대조를 수행하지 않았다. 이번 릴리즈 범위의
studio 소스 변경은 #3387·#3348 두 건뿐이고 둘 다 flush 횟수와 직접 연관이 보이지 않으나,
대조 없이 단정하지 않았다. 작업지시자 판단으로 별도 이슈로 분리하고 릴리즈를 계속했으며,
CHANGELOG 와 릴리즈 노트의 **"알려진 문제"** 절에 회귀 여부 미확정 사실과 함께 명시했다.

부수 관찰로 `--mode=headless` 가 WSL2 에서 브라우저 기동 자체에 실패하는 문제(`/sys/devices/
system/cpu/cpu0/cpufreq/` 부재)도 #3412 에 기록했다. 같은 headless 인 다른 스위트는 통과하므로
이 스위트만 다른 실행 경로를 탄다.

### 스토어 수동 업로드 — 완료

2026-07-26 메인테이너가 **3개 스토어 모두 제출 완료**했다(Chrome Web Store, Microsoft Edge
Add-ons, Firefox AMO). AMO 는 확장 zip 과 source zip 을 함께 올렸다. 이후 각 스토어 심사
결과는 릴리즈 범위 밖이다.

## 7. 운영 기록

| 단계 | 산출물 |
|---|---|
| 수행계획서 | `mydocs/plans/task_m100_3401.md` |
| 1단계 보고서 | `mydocs/working/task_m100_3401_stage1.md` |
| 2단계 보고서 | `mydocs/working/task_m100_3401_stage2.md` |
| 최종 보고서 | 이 문서 |

관련 PR: [#3392](https://github.com/edwardkim/rhwp/pull/3392)(`53ed38a40`),
[#3393](https://github.com/edwardkim/rhwp/pull/3393)(`ce2156dad`),
[#3400](https://github.com/edwardkim/rhwp/pull/3400)(`91bd61758`),
[#3417](https://github.com/edwardkim/rhwp/pull/3417)(`52c3bb493`),
[#3424](https://github.com/edwardkim/rhwp/pull/3424)(`ace187d52`),
[#3418](https://github.com/edwardkim/rhwp/pull/3418)(`1dbf024fd`).
