# #7473 standard release 검증과 배포 산출물 재사용 — 2026-10-23

Issue: [#7473](https://github.com/edwardkim/rhwp/issues/7473).
[PR #7474](https://github.com/edwardkim/rhwp/pull/7474) · [opt-in PR #7488](https://github.com/edwardkim/rhwp/pull/7488)

## 2026-10-23 최신 devel 검증과 수용 판단

기준 devel `605ccfd92f2f12f18e14182548d729f9929fda00`을 이력 보존 merge로 통합한
code head `df39fb5d7a566ba6a49c2f6a04965616d2c2af73`의 검증을 완료했다.
제품 코드·Cargo 입력·Studio·VS Code 확장·기존 fixture는 이 devel과 같다.
#7488 head `3bfe08e24a55f16f7408405ead9550ffab925dad`는 이미 포함되어 있다.
아래 결과는 코드 후보의 결과이며, 이 보고서를 추가한 문서 후행 head의 CI는 별도로 확인한다.

**두 PR의 수용을 권고한다.** #7488은 명시적 opt-in 최적화이고 #7474는 standard release와
실제 패키지의 검증 신뢰도를 높이는 변경이다. 추가 실행 비용만을 이유로 더 보류할 근거는 약하다.
GitHub 승인·병합·실제 배포·이슈 종료는 수행하지 않았다. 문서 후행 head의 CI 성공 뒤 #7474를
Ready로 전환하고 메인테이너의 최종 수용 판단을 기다리는 절차다.

### 최신 실행과 산출물

[Full CI](https://github.com/edwardkim/rhwp/actions/runs/39885844384),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39885844317),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/39885844282),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/39885844394), CI Impact Policy가 성공했다.
수동 실행을 포함한 code head check는 35 success·12 skipped이며, 별도 commit status도 성공했다.
로컬 focused Python 46개, workflow 계약 192개, Node 계약 102개, actionlint·공백 검사가 통과했다.
중복되는 검사 묶음은 합산하지 않는다. 기존 12개 입력의 bytes와 code commit blob도 일치했다.

| 경로 | WASM wrapper wall | 해당 빌드 job 전체 | 결과 |
| --- | ---: | ---: | --- |
| [PR Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39885844291) | 717.900초 | 1,207초 | Canvas 3/3, direct PDF 3/3, readiness 8/8 |
| [Pages verify-only](https://github.com/edwardkim/rhwp/actions/runs/39885887626) | 677.745초 | 1,086초 | 렌더·dist 해시·artifact 성공, Deploy skipped |
| [Package verify-only](https://github.com/edwardkim/rhwp/actions/runs/39885906844) | 682.242초 | WASM job 1,072초 | WASM·VSIX·집계 성공, 외부 게시 4채널 skipped |

모두 EPYC 9V74, 논리 CPU 4개, Ubuntu 24.04 image `20261012.334.1`이지만 서로 다른 VM의 n=1이다.
Rust 1.93.1, wasm-pack 0.15.0이며 세 경로 모두 release/standard, rlib+cdylib, manifest LTO와
실제 wasm-opt 실행을 확인했다. 이 표의 차이를 최적화 개선율로 해석하지 않는다.

PR 실제 checkout은 GitHub merge ref `ab0049304398b423bc812abdbb0078379ae33e52`이고
Pages/package는 위 code head다. 세 경로의 JS/WASM 해시가 각각 같았으며 실제 브라우저 응답
22건(18+2+2)이 각 manifest와 일치했다. artifact ZIP digest, 내려받은 wasm-pkg, 최종 VSIX
내부 JS/WASM을 재검산했다. Pages dist는 job의 업로드 직전 검사 성공으로 확인했다.
Pages 전체 artifact를 로컬에 내려받아 재검산한 것은 아니다.

- JS: 481,233 bytes, `da00f4937f352731a2d24a1ba4b54629009aa5edc9c929815aed2cd069cca3bc`.
- WASM: 20,488,912 bytes, `51d6a242d1c076d5fd72e8881dcc1dc8133a47e0c0e596e007b2da1c6312a28f`.
- package 집계는 accepted=true, verdict=completed, mode=verify이며 네 외부 채널은 verify-only다.
- `publish=false`에서 생략되는 npm 패키징은 로컬 `npm pack --offline --ignore-scripts`로 보완했다.
  lifecycle 빌드가 없는 `@rhwp/core`·`@rhwp/editor` 0.8.7 tarball을 생성했다. core는 이 run의
  wasm-pkg와 기존 prepare-npm.sh를 사용했고 내부 JS/WASM이 같았다. editor 포함 파일도 원본과 같았다.

### 비용과 한계

배포 전 검증 action은 Pages 222초, package 208초였다. WASM을 다시 빌드하지 않지만 Native PDF
참조 CLI·브라우저 준비와 렌더 검사 비용은 추가된다. 이번 Render Diff는 Build & Test보다
425초 먼저 끝나 이 실행의 PR 전체 완료 시점을 늦추지 않았다. runner 사용량은 증가한다.

최근 7일의 정상 첫 실행 167쌍에 이전 standard 실측을 대입한 비용 모델은 추가 runner 시간 약 6.5%,
대기 증가 중앙값 0초·P90 약 110초였다. 이는 과거 실행에 대한 추정이며 최신 코드의 대조 실험이나
향후 대기시간 보장은 아니다. 세 최신 실행 모두 cache miss였다. Pages는 cache 저장에 성공했고
package는 같은 키의 선행 저장으로 예약이 거부됐다. 이전 예산 초과 오류와 구분한다.
PR Render Diff는 restore-only이며 이 branch의 저장 성공을 다른 PR의 warm cache 보장으로 확대하지 않는다.

기존 report-only PDF 경고 4개는 각 경로에 남는다. 대표 문서·쪽 범위 검사이며 전체 문서 정확도,
runtime 성능, 모든 출력의 동등성이나 registry 게시 성공을 검증했다고 주장하지 않는다.
#7488 Mac 측정은 아래 역사적 기록이며 이번에 opt-in 성능을 다시 측정한 것은 아니다.

현재 devel에서 #7488 → #7474를 merge·squash 방식으로 각각 로컬 시뮬레이션했고 최종 tree는
모두 검증한 code tree `a03114506d543c95f95c3ed3a1d3fd50115c9182`와 같았다. 실제 ref는 변경하지 않았다.
향후 base/head 변경이나 문서 추가 뒤에는 해당 최종 상태를 다시 확인한다.

[수치·해시·fixture 요약](assets/issue-7473/release-standard-20261023.json) ·
[최신 결과 코멘트](https://github.com/edwardkim/rhwp/pull/7474#issuecomment-6355793528) ·
[검토 기록](../pr/archives/pr_7474_review.md)

## 이전 기록 — 2026-10-20

아래의 SHA·시간·cache 오류와 당시 대기 조건은 역사적 기록이며 현재 판정은 위 10월 23일 결과를 따른다.

[PR #7474](https://github.com/edwardkim/rhwp/pull/7474) · [opt-in PR #7488](https://github.com/edwardkim/rhwp/pull/7488)

## 2026-10-20 구현과 검증

기준 devel `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`, #7488 head
`3bfe08e24a55f16f7408405ead9550ffab925dad`, #7474 code head
`1ae9b510a5af0164de45191670e5e895e97270b3`에서 역할을 분리했다.

- #7488은 환경변수로 명시할 때만 cdylib를 사용한다. 기본 경로·전역 release 설정은 유지한다.
- #7474 PR Render Diff와 수동 auto는 standard release로 검사한다. Frontend dev 빌드와 기존
  trusted impact 분류는 유지한다. 관련 PR의 release 비용을 없앴다고 주장하지 않는다.
- Pages/npm은 이미 필요한 standard WASM 빌드를 계측 wrapper로 실행하고, 그 패키지를 Canvas/direct PDF로
  확인한 뒤 업로드한다. 검사 action은 WASM을 재빌드하지 않으며 Native PDF 비교 CLI만 준비한다.
- dirty/stale/dev/cdylib 빌드·누락 소비·해시 불일치·파일 교체는 실패한다. Pages dist도 업로드 직전에 대조한다.
  npm/VSIX는 기존 wasm-pkg download 경로를 사용하며 다른 PR run의 artifact를 신뢰하지 않는다.
- 기존 게시 trigger·권한·exact release source guard를 유지한다. 이 검증은 실제 배포 없이 실행했다.

### 실제 실행 결과

| 경로 | CPU | WASM wrapper wall | 전체 해당 job | 결과 |
| --- | --- | ---: | ---: | --- |
| [PR Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39380203897) | EPYC 9V74 | 689.626초 | 1134초 | Canvas 3/3, direct PDF 3/3, readiness 8/8 |
| [Pages verify-only](https://github.com/edwardkim/rhwp/actions/runs/39380248603) | Xeon Platinum 8370C | 778.382초 | 1177초 | 검사·dist hash·artifact 통과, Deploy skipped |
| [Package verify-only](https://github.com/edwardkim/rhwp/actions/runs/39380255159) | EPYC 7763 | 743.018초 | WASM job 1155초 | 검사·WASM/VSIX artifact 통과, 외부 게시 4채널 skipped |

각기 다른 VM의 n=1이며 세 시간을 개선율 계산에 사용하지 않는다. 새 배포 검증 action 자체의 실행 시간은
Pages 225초, package 239초였다. WASM을 추가로 빌드하지 않아도 Native 비교 준비·브라우저 검사 비용은 남는다.
PR Cargo cache는 miss, 두 배포 manifest의 cache 값은 unknown이다. 배포 로그의 cache miss 및
budget 초과에 따른 cache 저장 거부를 확인했으며, 새 cache가 저장됐다고 보고하지 않는다.

세 경로의 JS와 WASM은 각각 byte hash가 같았고 브라우저 실제 응답 22건(18+2+2)이 각 manifest와 일치했다.
업로드된 wasm-pkg도 내려받아 같은 verifier로 확인했고, 최종 VSIX 내부의 JS/WASM도 같은 해시였다.
Pages의 dist 검사는 run의 성공 step으로 확인했다. package 집계 verdict는 completed, 네 채널은 verify-only였다.

- JS: 479,094 bytes, `915ddc5aa7998937f009c76b71e53236a2536b2c30de1a37b9603b95e0b3ff15`.
- WASM: 19,281,907 bytes, `a7462849b7e0db027b59b81b791d13c45f1fc7669d0aed45caa1a20ca15b8508`.
- PR의 실제 checkout은 GitHub merge ref `9c14f1b3cdeab25244e6111aeab996227e4c3371`,
  배포 verify-only checkout은 code head `1ae9b510a5af0164de45191670e5e895e97270b3`이다.
- report-only PDF 경고 4개는 각 경로에 남는다. 허용치를 완화하지 않았으며 전체 문서·전체 쪽의
  독립 한컴 정확도, runtime 성능 또는 모든 PNG의 cross-run byte 동등성을 주장하지 않는다.

### CI와 재현 근거

[Full CI](https://github.com/edwardkim/rhwp/actions/runs/39380204001),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39380203913), Render Diff와 관련 PR 검사에서
code head는 33 success·5 skipped였다. #7488의 새 head는
[CI](https://github.com/edwardkim/rhwp/actions/runs/39379760031)·
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39379760037) 포함 31 success·5 skipped였다.
배포 verify-only는 PR 검사와 별도로 위 run에서 확인했다.

로컬 focused Python 46개, workflow 계약 192개, 정책·provenance Node 102개, actionlint·구문·공백 검사를
통과했다. 서로 중복되는 검사 묶음은 합산하지 않는다. 기존 12개 fixture의 실제 bytes와 code commit blob도
대조했다. 제품 Rust·조판 로직·fixture·baseline은 변경하지 않았다.

[빌드·산출물 요약](assets/issue-7473/release-standard-20261020.json)과 각 run의 artifact에 근거를 보존했다.
검토와 문서-only 후행 head의 확인은 [PR review](../pr/archives/pr_7474_review.md)를 따른다.
실제 배포·승인·병합·이슈 종료는 수행하지 않았다.

## macOS 결과의 위치

[Mac 4쌍 측정 코멘트](https://github.com/edwardkim/rhwp/pull/7474#issuecomment-6284410757)는 이전 제품 코드
`0eb4cc2`의 M1 가상 머신 warm 재빌드 비교다. 중앙값 595.698→527.310초(11.48% 절감),
4문서 85쪽 SVG/RenderTree·PNG 9개 동등성을 확인했다. 전체 run은 timeout cancelled였으나 완료된 측정·
검증 증거를 별도로 재검산했다. 이 결과는 #7488 opt-in의 근거이며 새 head의 cold 시간이나 순수 LTO 효과가 아니다.

## 이전 기록 — 2026-10-18 PDF 추가 후 비교

아래는 이전 코드와 정책의 역사적 측정이다. 당시의 cdylib 기본값·head·검토 상태는 현재 구현에 적용하지 않는다.


[PR #7474](https://github.com/edwardkim/rhwp/pull/7474) · [의존 PR #7488](https://github.com/edwardkim/rhwp/pull/7488)

### 결론

PDF import가 기본 feature인 최신 devel을 통합하고 #7488의 cdylib opt-in을 #7474에 적용했다.
dev·기존 release·cdylib release 각 3회, 총 9회가 성공했다. PNG 43개는 모든 실행에서 byte 단위로
같았고 실제 WASM 응답 162건도 각각 빌드 manifest와 일치했다.

동일 CPU별 WASM wrapper 시간은 cdylib 적용 후 **9V74 약 1.5%, 7763 약 6.7% 감소**했다.
그러나 CPU별 표본이 1~2회이고 서로 다른 VM이므로 확정 개선율이나 유의한 성능 개선으로 주장하지 않는다.
전체 job 감소는 각각 약 6초·41초다. dev 대비 release 검증 비용은 여전히 전체 job 기준 약 8분 증가한다.
#7488의 과거 warm n=1 결과인 9.48%를 이번 cold CI에 적용해서는 안 된다.

코드 검증은 통과했다. #7474는 추가 CI 비용과 명령 단위 LTO 정책을 검토할 수 있도록 Draft를 유지한다.
#7488은 메인테이너 직접 승인 후 반영 가능한 Ready 후보이며 아직 승인·병합되지 않았다.

### 통제 조건과 해석 범위

- 고정 base: `6ed1682c3267ea786734eb2e75252916a861d1fd`.
- 고정 코드 후보: `0eb4cc2ffde257a15080d0add104247c941fcbf9`.
- #7488 원 head: `e8e57b36d4703556128dae31e37eaa9acdceee0d`. merge로 원 이력을 보존했다.
- 동일 source·fixture·lockfile, `write-images=true`, 개별 GitHub hosted VM, 논리 CPU 4개.
- Ubuntu 24.04 image `20261012.334.1`, Linux `6.17.0-1022-azure`, Rust `1.93.1`, wasm-pack `0.15.0`.
- wasm-bindgen `0.2.127`, release wasm-opt `117 -O`; 도구 executable hash는 실행 간 같다.
- Cargo cache **9/9 miss**, matched key 없음. 측정 ref는 restore-only이며 seed를 저장하지 않았다.
- 새 cache key는 profile·crate type·LTO 정책을 구분한다. 캐시 hit 성능은 이번 측정 범위 밖이다.
- CPU는 EPYC 9V74·EPYC 7763·Xeon 6973P-C로 섞였다. 조건별 통합 중앙값을 최적화 효과로 비교하지 않는다.

`dev/auto`는 standard, `release/standard`는 기존 rlib+cdylib와 Cargo manifest LTO 설정,
`release/cdylib`는 cdylib와 명령 단위 `profile.release.lto=false`를 사용한다.
release 두 경로 모두 wasm-opt를 실제 수행하지만 **배포 빌드와 플래그까지 동일하다는 뜻은 아니다**.
PR 경로는 release/cdylib로 고정하며 수동 실행에서 세 조건을 재현할 수 있다.

wall은 helper 내부 빌드 시간, step은 metadata 수집 등을 포함하는 Actions 시간이다.
전체 job은 Canvas visual diff 시작부터 끝까지로 큐 대기·preflight는 제외한다.
Cargo·bindgen·opt 시간은 로그 수신 경계 근사이며 독점 CPU 시간으로 해석하지 않는다.
VM 부하·네트워크는 통제하지 못했다. PDF 기능만의 비용을 분리한 ablation 실험은 아니다.

### 실행별 결과

단위: 초. 모든 run은 attempt 1, 동일 코드 후보에서 성공했다.

| 조건·반복 | CPU | wrapper wall | WASM step | Cargo 근사 | opt/마무리 근사 | 전체 job |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| [dev 1](https://github.com/edwardkim/rhwp/actions/runs/39073001510) | 7763 | 149.144 | 160 | 134.617 | — | 627 |
| [release standard 1](https://github.com/edwardkim/rhwp/actions/runs/39072999119) | 9V74 | 599.171 | 609 | 423.772 | 167.863 | 1031 |
| [release cdylib 1](https://github.com/edwardkim/rhwp/actions/runs/39073003282) | 7763 | 662.695 | 674 | 484.543 | 169.738 | 1113 |
| [dev 2](https://github.com/edwardkim/rhwp/actions/runs/39073302072) | 9V74 | 121.112 | 130 | 107.922 | — | 542 |
| [release standard 2](https://github.com/edwardkim/rhwp/actions/runs/39073307122) | 9V74 | 597.802 | 607 | 423.602 | 166.039 | 1012 |
| [release cdylib 2](https://github.com/edwardkim/rhwp/actions/runs/39073312286) | 6973P-C | 702.087 | 712 | 457.709 | 235.571 | 1164 |
| [dev 3](https://github.com/edwardkim/rhwp/actions/runs/39073316898) | 7763 | 150.605 | 160 | 136.630 | — | 615 |
| [release standard 3](https://github.com/edwardkim/rhwp/actions/runs/39073320923) | 7763 | 710.405 | 722 | 526.245 | 174.667 | 1154 |
| [release cdylib 3](https://github.com/edwardkim/rhwp/actions/runs/39073325398) | 9V74 | 589.384 | 603 | 416.285 | 164.341 | 1016 |

### CPU별 비교

동일 모델 안에서도 대응 VM 비교가 아니며 표본이 작다. 단위: 초, 각 셀은 중앙값(표본 수).

| CPU | dev wall | standard wall | cdylib wall | standard→cdylib | 전체 job standard→cdylib |
| --- | ---: | ---: | ---: | ---: | ---: |
| EPYC 9V74 | 121.112 (1) | 598.487 (2) | 589.384 (1) | -9.103 / -1.52% | 1021.5→1016 (-5.5) |
| EPYC 7763 | 149.875 (2) | 710.405 (1) | 662.695 (1) | -47.710 / -6.72% | 1154→1113 (-41) |
| Xeon 6973P-C | 미측정 | 미측정 | 702.087 (1) | 비교 불가 | 비교 불가 |

dev→cdylib의 전체 job 차이는 9V74 542→1016초(+474), 7763 621→1113초(+492)다.
CPU 혼합 상태의 실제 관측 중앙값은 dev 615초, standard 1031초, cdylib 1113초이나,
**이 세 숫자로 cdylib가 느려졌다고 결론 내릴 수 없다**. 특히 Intel 표본은 cdylib에만 있다.
release opt/마무리는 약 164~236초이며 이번 wrapper 변경으로 제거되지 않는다.
추가 최적화 판단에는 같은 runner에서 순서를 교차한 반복 비교와 실제 cache hit 검증이 필요하다.

### 출력·검증 결과

- 9회 모두 Canvas 3/3, 직접 PDF gate 3/3, CanvasKit readiness 8/8 통과.
- 실제 앱에서 관측한 WASM 응답 각 18건, 총 162건이 해당 실행 manifest와 일치했다.
- 각 실행 PNG 43개(시각/PDF 27 + readiness 16)의 파일 집합과 SHA-256이 모두 같았다.
- 각 빌드 조건 안에서는 3회 WASM·JS bytes/hash가 재현됐다. standard/cdylib release의 JS는 같다.
- 두 release WASM 바이너리는 다르다. PNG 동등성을 바이너리 동등성으로 확대하지 않는다.
- report-only PDF 경고는 기존과 같은 4개다. 직접 PDF gate와 별개이며 기준값을 완화하지 않았다.
- 대표 KTX 1쪽의 지도·시간/요금표와 table-border-style의 표·세로 글줄을 직접 판독했다.
  KTX 요금표 머리글의 기존 겹침은 남아 있다. 출력 보존 검증이며 한컴 정합·기존 시각 결함 해결은 주장하지 않는다.
- 별도 한컴 기준 PDF나 전쪽 silhouette TSV를 새로 생성하지 않았다. 조판 알고리즘 수정은 비해당이다.

| 빌드 | WASM bytes | WASM SHA-256 |
| --- | ---: | --- |
| dev | 55707545 | `00989694eda270f5c579543a58a43fe23da5b6dd799b2840088512551dc1fdba` |
| release standard | 17809310 | `53d52a44ea3691fa3c774cdc749f96cce0d057d7ae596c8e45f5c4387a674087` |
| release cdylib | 17756530 | `39cc9e7587fabb776e80e5f30825a4694ebea2bba3745ba679b0b17b34df3caf` |

대표 이미지는 cdylib 1 실행의 원본 PNG다. 나머지 8회와 해시가 같다.

![KTX 1쪽 — 기존 요금표 머리글 겹침 보존](../pr/assets/issue_7473/20261018/ktx-p01-layer.png)

![CanvasKit table-border-style](../pr/assets/issue_7473/20261018/table-border-style.png)

### 코드 검증과 리뷰

정확한 후보 `0eb4cc2ff`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39072973013),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39072973012),
[PR Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39072973025),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/39072973002),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/39072973042)가 성공했다.
check 집계는 34 success·5 skipped, 실패·미완료 0건이다. 별도 PR Render Diff 실행은 위 9회 통계에 넣지 않았다.

로컬에서는 wrapper/render/workflow/Docker focused 28개, Python workflow 전체 192개,
Node CI 정책·WASM 소비 검사 102개를 통과했다. actionlint·shell/Python/Node 구문·변경 링크·공백도 검사했다.
로컬 Rust toolchain이 없어 로컬 전체 Rust 회귀는 실행하지 않았다. 실제 Rust 빌드와 브라우저 검증은 위 CI 결과다.
[검토 기록](../pr/archives/pr_7474_review.md) · [통합과 되돌리기](../pr/archives/pr_7474_review_impl.md)

### 이전 측정과 증적 보존

2026-09-28 후보의 dev/release WASM step 중앙값은 122/421초, 전체 job은 500/760초였다.
현재 standard release step은 609초 중앙값이다. 비용 증가 자체는 관측했지만 코드·PDF feature·runner image·CPU가
함께 바뀌었으므로 증가량 전부를 PDF에 귀속하지 않는다.
[이전 보고서](https://github.com/edwardkim/rhwp/blob/9d05998e09e4698391fe44f543abdd66ddc86c99/mydocs/report/task_m100_7473_report.md),
[이전 시간 원자료](assets/issue-7473/ci-results.json), [이전 출력 증거](assets/issue-7473/ci-evidence.json)는 과거 자료다.
과거 macOS readiness 7/8도 이번 Ubuntu 8/8 결과와 구분한다.

이번 9개 run의 `render-diff-artifacts`에 build log/manifest/consumption·PNG·baseline report가 있다.
artifact ZIP의 GitHub SHA-256을 다운로드 후 검증했다. 보존 기간은 14일이다.
다운로드 원본과 API 집계는 로컬 `output/pr-review/7474/`에 보관하며 원시 JSON/log를 새로 커밋하지 않는다.
위 표·WASM hash·대표 PNG는 저장소에 남긴다. 12개 입력의 blob/hash 확인은 검토 기록에 있다.
측정용 임시 ref를 정리해도 run URL과 source SHA로 실행을 식별할 수 있다.
