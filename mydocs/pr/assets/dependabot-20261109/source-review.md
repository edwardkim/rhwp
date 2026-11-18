# source·원 PR CI 검토

기준 devel `024d610633ede166ebcbb915f9c90e7a9d0649d8`. 원 merge source `d93c03a4391a5ec22eb7688ff890a58836f41884`, CodeQL 주석 보정·검증 source `2320351d2325a41d900e91f4e07f559d4a924a7d`, 하네스·고지 보정 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`. 제품 구현·manifest·lockfile·workflow는 `2320351d2325a41d900e91f4e07f559d4a924a7d`와 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b` 사이에서 동일하다. 실제 차이는 고지 Markdown·계획/조사 Markdown과 기존 extension smoke의 대기 순서뿐이다.

원 CI 확인 시각: `2026-11-09T05:23:33.131052+00:00`. 원 head 19개가 현재 branch의 조상임을 확인했다. 2026-11-09 최종 preflight에서도 원 head 변경 없음·Ready OPEN 19건·closing issue 0건·본인 edwardkim 열린 PR 0건·원격 devel 동일을 확인했다.

| 원 PR | 원 head | 보존 merge commit | reviewer 확인 시각 UTC | 원 CI |
| --- | --- | --- | --- | --- |
| [#8811](https://github.com/edwardkim/rhwp/pull/8811) | 73f6b8c1f431dc36de0d4b17ab5ce2bf423be5ca | 0c2c6397e932bce5f06b6bf24bbb7c7606b89b64 | edwardkim / 2026-11-09T04:02:18.561006+00:00 | 성공 |
| [#8812](https://github.com/edwardkim/rhwp/pull/8812) | bb3d70c892bfffb5ee59d7570328ee86186ec5e4 | b59ac1414841fdd76b61760418614eeea0f6b561 | edwardkim / 2026-11-09T04:02:20.397312+00:00 | 성공 |
| [#8813](https://github.com/edwardkim/rhwp/pull/8813) | 5a98e6d56974481e831eb232f5c20631b48da087 | 4049cfa9c2ec10c9b7337c80efcf907f983e53b8 | edwardkim / 2026-11-09T04:02:22.220793+00:00 | 성공 |
| [#8814](https://github.com/edwardkim/rhwp/pull/8814) | efd1a65979cfe42ced6649f7118d1d65dbecaad6 | 400ea25cca32792375a685b0eccb0012b459168d | edwardkim / 2026-11-09T04:02:24.162151+00:00 | Build & Test:failure, Native Skia tests:cancelled |
| [#8815](https://github.com/edwardkim/rhwp/pull/8815) | 384d609211f9caa0408d4912c1a1e8f23c872a07 | 313f49d5f40d16ee3c8b663c612c6ac59bc5d3ea | edwardkim / 2026-11-09T04:02:26.008050+00:00 | 성공 |
| [#8816](https://github.com/edwardkim/rhwp/pull/8816) | 5d8b73a5f75482af8f1ee770f5ca698fcca1d00b | 36f9c1792474965fa95ed31085c5d65bacc86897 | edwardkim / 2026-11-09T04:02:28.088426+00:00 | 성공 |
| [#8817](https://github.com/edwardkim/rhwp/pull/8817) | 2fb2264a8242539611a424e53c722651166301f6 | 770c451883741244af9970ec8357853ba62efaa3 | edwardkim / 2026-11-09T04:02:29.838186+00:00 | Build & Test:failure, Native Skia tests:cancelled |
| [#8818](https://github.com/edwardkim/rhwp/pull/8818) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | f7ce6eed4f369ec62535780d30fa3b43a478626c | edwardkim / 2026-11-09T04:02:31.541621+00:00 | Analyze (actions):failure, Analyze (javascript-typescript):failure, Analyze (python):failure, Analyze (rust):failure, Build & Test:failure, Lint (fmt, clippy, WASM check):failure |
| [#8819](https://github.com/edwardkim/rhwp/pull/8819) | 526ef018fa216d42808a24b6f522ac6bad4bfdbd | 8056d896604fece5e8bc9f3baa5b2ce3614df4e2 | edwardkim / 2026-11-09T04:02:33.549077+00:00 | Build & Test:failure, Native Skia tests:cancelled |
| [#8820](https://github.com/edwardkim/rhwp/pull/8820) | c607ea643619fced34132144e07d43d4f9637c6e | b13408d5e8b8193b7116c3bb8cf1ae4e3e01a881 | edwardkim / 2026-11-09T04:02:35.330381+00:00 | Analyze (actions):failure, Analyze (javascript-typescript):failure, Analyze (python):failure, Analyze (rust):failure, Build & Test:failure, Lint (fmt, clippy, WASM check):failure, Native Skia tests:cancelled |
| [#8821](https://github.com/edwardkim/rhwp/pull/8821) | 6bf1774453978c45d954c5cd68a7977ae6f293fb | ae09f2cf62bf7bdfd175f4b6e8a12181ee65f060 | edwardkim / 2026-11-09T04:02:37.564582+00:00 | 성공 |
| [#8822](https://github.com/edwardkim/rhwp/pull/8822) | 4153bd5a09960fa69bf45e15d092b5b6861c985d | 9f76018a531e0a59cceb7ee7f3d13196e335e7d2 | edwardkim / 2026-11-09T04:02:39.499122+00:00 | 성공 |
| [#8823](https://github.com/edwardkim/rhwp/pull/8823) | 0de0b5e3ede6df2d257aaa62308521a25ea3aced | f7234a903a996e6ded278fd26d1f94f63656f875 | edwardkim / 2026-11-09T04:02:41.717881+00:00 | 성공 |
| [#8824](https://github.com/edwardkim/rhwp/pull/8824) | 7f26c047348a920fd2397a8b20c4edfaa0c8dd46 | f71a04e1fa8c542f427a838ae47a70a1fa32f298 | edwardkim / 2026-11-09T04:02:43.671453+00:00 | 성공 |
| [#8825](https://github.com/edwardkim/rhwp/pull/8825) | a705ee565a3985b560db58167e70b5678285ddd6 | d1505860b34f9d5080115849c0e1eb45116d1c7a | edwardkim / 2026-11-09T04:02:45.600847+00:00 | 성공 |
| [#8826](https://github.com/edwardkim/rhwp/pull/8826) | a55a1ed94a3c26b480418d3898c8dbd20b22ee6b | 242b01e9a19fb276586c12cbfad813bf0b120ef1 | edwardkim / 2026-11-09T04:02:47.282001+00:00 | 성공 |
| [#8827](https://github.com/edwardkim/rhwp/pull/8827) | bb4981791b116b393e3f432c14cdf261304f7a69 | 0674a97247797a3f0e1ce5b4612fb6fcd6ff320d | edwardkim / 2026-11-09T04:02:49.253518+00:00 | 성공 |
| [#8828](https://github.com/edwardkim/rhwp/pull/8828) | d65c82058e7caf6bc8e330785328563d88513475 | f95da5ec6079414bfe8688bab06dbe84e9b8e38d | edwardkim / 2026-11-09T04:02:51.126989+00:00 | 성공 |
| [#8829](https://github.com/edwardkim/rhwp/pull/8829) | 24ed2188056425b95b7c70b255c2e7d0ee4a447b | d93c03a4391a5ec22eb7688ff890a58836f41884 | edwardkim / 2026-11-09T04:02:53.022430+00:00 | 성공 |

## 실패 원인과 처리

14건은 원 CI 성공이다. 나머지 5건은 CodeQL 두 PR의 버전 불일치 또는 아래 Native 4건의 시간 상한이다. 중복 #8820을 두 번 세지 않는다.

| 원 PR | job | head | 확인 |
| --- | --- | --- | --- |
| 8814 | [Native](https://github.com/edwardkim/rhwp/actions/runs/42553019985/job/128952650225) | efd1a65979cfe42ced6649f7118d1d65dbecaad6 | 30m0s 상한, 성공한 테스트 뒤 후속 컴파일 중 취소. assertion·컴파일 오류 없음 |
| 8817 | [Native](https://github.com/edwardkim/rhwp/actions/runs/42553033036/job/128952707153) | 2fb2264a8242539611a424e53c722651166301f6 | 30m0s 상한, 성공한 테스트 뒤 후속 컴파일 중 취소. assertion·컴파일 오류 없음 |
| 8819 | [Native](https://github.com/edwardkim/rhwp/actions/runs/42553041597/job/128954514434) | 526ef018fa216d42808a24b6f522ac6bad4bfdbd | 30m0s 상한, 성공한 테스트 뒤 후속 컴파일 중 취소. assertion·컴파일 오류 없음 |
| 8820 | [Native](https://github.com/edwardkim/rhwp/actions/runs/42553045011/job/128957241150) | c607ea643619fced34132144e07d43d4f9637c6e | 30m0s 상한, 성공한 테스트 뒤 후속 컴파일 중 취소. assertion·컴파일 오류 없음 |

| 원 PR | 실제 job | source | 원인 |
| --- | --- | --- | --- |
| 8820 | [Analyze (javascript-typescript)](https://github.com/edwardkim/rhwp/actions/runs/42553045065/job/128963797157) | c607ea643619fced34132144e07d43d4f9637c6e | CodeQL init/analyze 버전 불일치 |
| 8820 | [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/42553045065/job/128963797208) | c607ea643619fced34132144e07d43d4f9637c6e | CodeQL init/analyze 버전 불일치 |
| 8820 | [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/42553045011/job/128957241182) | c607ea643619fced34132144e07d43d4f9637c6e | workflow 계약의 CodeQL 버전 불일치 |
| 8820 | [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/42553045011/job/128957241150) | c607ea643619fced34132144e07d43d4f9637c6e | 30분 제한 / 실제 로그 대조 |
| 8819 | [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/42553041597/job/128954514434) | 526ef018fa216d42808a24b6f522ac6bad4bfdbd | 30분 제한 / 실제 로그 대조 |
| 8818 | [Analyze (javascript-typescript)](https://github.com/edwardkim/rhwp/actions/runs/42553035480/job/128955553463) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | CodeQL init/analyze 버전 불일치 |
| 8818 | [Analyze (python)](https://github.com/edwardkim/rhwp/actions/runs/42553035480/job/128955553502) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | CodeQL init/analyze 버전 불일치 |
| 8818 | [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/42553035480/job/128955553411) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | CodeQL init/analyze 버전 불일치 |
| 8818 | [Analyze (actions)](https://github.com/edwardkim/rhwp/actions/runs/42553035480/job/128955553483) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | CodeQL init/analyze 버전 불일치 |
| 8818 | [Lint (fmt, clippy, WASM check)](https://github.com/edwardkim/rhwp/actions/runs/42553035396/job/128954717454) | 9b4ead9100f821bbcfe2331a3bf2916b96b575fb | workflow 계약의 CodeQL 버전 불일치 |
| 8817 | [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/42553033036/job/128952707153) | 2fb2264a8242539611a424e53c722651166301f6 | 30분 제한 / 실제 로그 대조 |
| 8814 | [Native Skia tests](https://github.com/edwardkim/rhwp/actions/runs/42553019985/job/128952650225) | efd1a65979cfe42ced6649f7118d1d65dbecaad6 | 30분 제한 / 실제 로그 대조 |
| 8820 | [Analyze (python)](https://github.com/edwardkim/rhwp/actions/runs/42553045065/job/128963797237) | c607ea643619fced34132144e07d43d4f9637c6e | CodeQL init/analyze 버전 불일치 |
| 8820 | [Analyze (actions)](https://github.com/edwardkim/rhwp/actions/runs/42553045065/job/128963797172) | c607ea643619fced34132144e07d43d4f9637c6e | CodeQL init/analyze 버전 불일치 |

#8818은 analyze만 4.39.3, #8820은 init만 4.39.3이었다. 각각 네 언어 Analyze와 lint의 버전 계약 실패를 확인했다. #7851·#7854 원 해결과 현재 codeql-action grouping이 이미 있음을 추적했다. 나뉘어 생성된 원인 자체는 미확정이며 Dependabot 설정을 임의로 수정하지 않았다. 둘을 함께 적용해 같은 full SHA로 맞추고 주석 5곳만 정렬했다.

원 실패·취소를 지우거나 rerun하지 않았다. 새 통합 exact head의 필수 CI·self-review·최신 base merge-tree 확인이 별도로 필요하다. [원격 처리안](remote-action-plan.md)을 따른다.
