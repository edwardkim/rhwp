## 다음 커밋이 PR #5707에 더하는 것

새 스킬은 감으로 넣지 않는다. `rhwp-skill-author` 스킬이 `"새 스킬 만들어"` 경로를 잡고, 끝난 뒤 **3-pass 게이트를 명령마다 세 번** 통과해야 한다.

```bash
python tools/skill_router/gate_new_skill.py
python -m unittest tools.skill_router.test_route
cargo test --test regression_suite_015 skills_have_valid_frontmatter -- --nocapture
```

카탈로그 스킬마다 전용 프로브 3개가 필수다. 트리거/슬러그 생성기 fallback은 없다. 프로브가 다른 스킬을 고르면 게이트가 실패한다 (`selected != catalog skill`). 현재 **27 스킬 × 3 = 81 프로브**, 선택 스킬이 카탈로그 id와 일치한다.

CI는 cargo/rust 전체를 기다리지 않는다. `.github/workflows/skill-router-gate.yml` 이 스킬·라우터 경로 PR에 Python만 돌린다.

- `gate_new_skill.py` ×3
- `test_route` / `test_skill_router_gate` / `test_catalog_routes` / `test_author_skill`
- `check_catalog_sync.py` — `.agents/skills` ≡ `catalog.json` ≡ `intents.py` ≡ `graph.py` (27/27)

로컬은 `precommit_skill_gate.py` + `install_git_hook.py` 가 같은 3-pass를 커밋 직전에 돌린다. `audit_skill_commands.py` 는 `target/release/rhwp.exe` v0.8.4 실명령 집합(207 known)과 SKILL.md 참조(189, unknown 0)를 대조한다.

증적은 터미널이 아니다. `rhwp export-svg --profile print` → Edge headless PNG. 카탈로그 27 스킬 전부 `skills/<id>/page.png` 가 있다.

새 CAP 번호는 만들지 않았다. `rhwp-skill-author` 는 기존 #5706 묶음이다.
