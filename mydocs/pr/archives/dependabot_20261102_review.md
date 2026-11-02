# Dependabot PR 9건 최종 검토 — 2026-11-02

## 최종 판정

**승인 — 원 PR #8449–#8457 9건 정상 merge 완료.** 사용자 승인에 따라 8개 원 head를 먼저 merge하고, Skia 보정 head의 새 CI·self-review를 마친 뒤 마지막으로 원 #8456을 merge했다.

| PR | 변경 | 최종 source head | merge SHA |
| --- | --- | --- | --- |
| [#8449](pr_8449_review.md) | chore(deps-dev): bump @types/chrome from 0.3.6 to 0.3.7 in /rhwp-studio | `2fa686d569a078ce9e44e05ca493e65d704784c7` | `01243e3fa72e8578b8f5a6ec33d3007d2f2a5313` |
| [#8450](pr_8450_review.md) | chore(deps): bump zip from 9.0.5 to 9.1.3 | `94fdf6fea2a7b56865e41ef3e8c8923033211c07` | `17c6082d28383a603de5c9e8389a90776e0da54c` |
| [#8451](pr_8451_review.md) | chore(deps): bump @anthropic-ai/sdk from 1.1.0 to 1.3.0 in /rhwp-studio | `b4b31c7fc834c2ed502ef1983e9fc36b341ab41f` | `e2f97b8c7d01a6d1487e41bd401fb5420c53d25b` |
| [#8452](pr_8452_review.md) | chore(deps): bump unicode-width from 0.2.2 to 0.2.3 | `5829d65eb6fd368a2557c016df4747d7b12362ca` | `1c9915beb85eb7ddaba6ec58bf4b43a1616e81a4` |
| [#8453](pr_8453_review.md) | chore(deps-dev): bump webpack from 5.112.1 to 5.112.2 in /rhwp-vscode | `dc8b52ad418594a42b52560c7b706ca0fe4f1783` | `72f8cafa505bf058a60ed35f1d4096a5700e2f0f` |
| [#8454](pr_8454_review.md) | chore(deps): bump libc from 0.2.189 to 0.2.191 | `4dd0e7eb196a1bfede636c4314fbe8fcc7bec8eb` | `0845306ef176618228746717a503f6aaf05c8f57` |
| [#8455](pr_8455_review.md) | chore(deps): bump roxmltree from 0.21.1 to 0.21.2 | `db38e6af4d65e5a8bdc7ba2697d3658a9839e505` | `db12e4a0dd006655e1b191e4fca340df30464acd` |
| [#8456](pr_8456_review.md) | chore(deps): bump skia-safe from 0.153.3 to 0.156.0 | `1fd74cf3ce37a3f6bcc20f8fb2f7fba0045f7048` | `feb092532a03f7ebba4e7a9517d25c2b1b5714b5` |
| [#8457](pr_8457_review.md) | chore(deps): bump taiki-e/install-action from 2.87.31 to 2.88.1 | `cd0ccf51995d9e985f8137d211db6077d7003a8b` | `fe0ecca2f58274832ec46a46c056e344fb20e6e9` |

## 검증과 Skia 보정

- 최소 보정 `8777e11c165636b19fe4f5db10eb0a7ee7ed06cf`: Cargo.lock의 prettyplease 0.3.0·checksum·syn 3 연결 3줄. 원 Native/Canvas 빌드 실패 E0277을 해소했고 제품 코드·기대값·CI 실행 명령은 바꾸지 않았다.
- 로컬 runtime 검증 `e58024b15e11438a0e1162a25d836d3c3c4ff7a9`와 실제 source head의 runtime·dependency tree가 같음을 확인했다. 8건 머지 뒤 actual-base sync도 전체 파일 tree를 바꾸지 않았다.
- 전체 nextest 12,369 PASS/기존 45 skip, Native lib 4,040 PASS/기존 12 ignored와 Native 집중 24 PASS, 기본 집중 31 PASS. 최소 글꼴 격리와 모든 적용 lint·workspace build·최적화 WASM을 완료했다.
- Studio 2,716 PASS/기존 2 skip, TypeScript·build·VS Code webpack compile·실제 AI 설정/패널 브라우저를 확인했다. 실제 초기화 WASM 응답 SHA도 producer와 일치했다.
- 4문서 전쪽 5쪽은 Native/fresh WASM 각 backend에서 기준/후보 픽셀이 모두 같다. 독립 Print raw 최저 Native 94.40%·WASM 98.77%, 미달·누락·2%p 초과 하락 0쪽. 직접 확대 판독과 기존 face/획 차이는 별도로 기록했다.
- npm 변경 3개와 Cargo 정확 버전 15개 감사에서 새 보고 취약점이 없었다. 전체 graph/C++ 보안 감사로 확대 해석하지 않는다. Action full SHA·권한을 유지했고 실제 원 head CI의 nextest 0.9.150 실행을 확인했다.

[정식 검증 증적](../assets/dependabot-20261102/README.md), [원격 merge·CI·self-review·duration](../assets/dependabot-20261102/remote-integration.json)에 근거를 보존한다. 원래 성공한 CI는 재사용했고 새 Skia 보정의 CI만 한 번 새 head에서 수행했다.

## 승인된 후속 처리

archive review·asset·오늘할일은 메인터너 운영 기록으로 devel에 직접 반영한다. 코드·테스트·workflow를 포함하지 않는다. 관련 closing issue는 9건 모두 없다. 댓글은 정식 초안에 실제 merge·CI·record·증적 SHA를 채우고 API로 재확인한다. 기본 devel을 동기화하고 이번 두 검토 worktree·branch·root output·미사용 review debug를 정리한다. 사용자 Vite 7700·기존 Studio WASM·다른 #8296 worktree는 보존한다.

실제 종료 상태·댓글 permalink·정리 경로와 용량은 `/home/edward/.local/state/rhwp/dependabot-cleanup-20261102.json`에 기록한다.
