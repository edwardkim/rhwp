"""[#4080] cache-generation-sweep.yml 의 정리 판정 계약 테스트.

스윕 로직은 checkout 금지 안전 경계 때문에 workflow YAML 안에 인라인되어 있다.
따라서 `test_ci_impact_workflow.py` 의 aggregate shell 과 같은 방식으로, YAML 에서
github-script 본문을 추출해 node 스텁 위에서 실행하고 판정만 단언한다.
"""

from __future__ import annotations

import json
import re
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2] / ".github/workflows/cache-generation-sweep.yml"
)
SCRIPT_MARKER = "          script: |\n"

HARNESS = """
const fixture = %(fixture)s;

const result = {
  deleted: [],
  info: [],
  warnings: [],
  failed: null,
  summary: null,
  calls: [],
};

for (const [key, value] of Object.entries(fixture.env)) {
  process.env[key] = value;
}

const context = { repo: { owner: 'edwardkim', repo: 'rhwp' } };

const core = {
  info: (m) => result.info.push(String(m)),
  warning: (m) => result.warnings.push(String(m)),
  setFailed: (m) => { result.failed = String(m); },
  summary: {
    addHeading() { return this; },
    addTable(rows) { result.summary = rows; return this; },
    async write() { return this; },
  },
};

const listPulls = Symbol('pulls.list');
const listBranches = Symbol('repos.listBranches');
const listTags = Symbol('repos.listTags');
const listCaches = Symbol('actions.getActionsCacheList');

const github = {
  rest: {
    pulls: { list: listPulls },
    repos: { listBranches: listBranches, listTags: listTags },
    actions: {
      getActionsCacheList: listCaches,
      deleteActionsCacheById: async ({ cache_id: id }) => {
        if (fixture.deleteFails && fixture.deleteFails.includes(id)) {
          throw new Error(`stub delete failure ${id}`);
        }
        result.deleted.push(id);
      },
    },
  },
  paginate: async (fn, params) => {
    if (fn === listPulls) {
      result.calls.push('pulls');
      // 실제 API 처럼 state 를 존중한다. `state: 'open'` 이 아니면 닫힌 PR 도 섞여
      // 돌아오므로, 보호 기준을 열림에서 전체로 넓히는 회귀가 테스트에 잡힌다.
      const state = params && params.state;
      if (state !== 'open') return [...fixture.openPrs, ...(fixture.closedPrs || [])];
      return fixture.openPrs;
    }
    if (fn === listBranches) {
      result.calls.push('branches');
      if (fixture.branchesThrow) throw new Error('stub listBranches failure');
      return fixture.branches;
    }
    if (fn === listTags) { result.calls.push('tags'); return fixture.tags || []; }
    if (fn === listCaches) { result.calls.push('caches'); return fixture.caches; }
    throw new Error('unexpected paginate target');
  },
};

(async () => {
%(script)s
})().then(
  () => console.log(JSON.stringify(result)),
  (error) => {
    result.threw = String(error && error.message);
    console.log(JSON.stringify(result));
  },
);
"""


def cache(cid, key, ref, created, mb=100):
    return {
        "id": cid,
        "key": key,
        "ref": ref,
        "created_at": created,
        "size_in_bytes": mb * 1024**2,
    }


class CacheSweepWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        body = workflow.split(SCRIPT_MARKER, maxsplit=1)[1]
        cls.script = textwrap.indent(textwrap.dedent(body), "  ")

    def run_sweep(self, **fixture):
        payload = {
            "env": {
                "DRY_RUN": "false",
                "KEEP_GENERATIONS": "2",
                "SWEEP_ORPHAN_REFS": "true",
                "LIMIT_BYTES": "10000000000",
                "WARN_PERCENT": "80",
                "FAIL_PERCENT": "95",
            },
            "openPrs": [],
            "closedPrs": [],
            "branches": [{"name": "devel"}],
            "tags": [],
            "caches": [],
            "deleteFails": [],
            "branchesThrow": False,
        }
        payload.update(fixture)
        payload["env"] = {**payload["env"], **fixture.get("env", {})}

        source = HARNESS % {
            "fixture": json.dumps(payload),
            "script": self.script,
        }
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "harness.mjs"
            path.write_text(source, encoding="utf-8")
            proc = subprocess.run(
                ["node", str(path)], capture_output=True, text=True, check=False
            )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        out = json.loads(proc.stdout.strip().splitlines()[-1])
        self.assertIsNone(out.get("threw"), out.get("threw"))
        return out

    # --- 고아 ref 정리 (#4080 원인 2) ---

    def test_deletes_cache_of_deleted_branch_regardless_of_generation(self):
        out = self.run_sweep(
            branches=[{"name": "devel"}],
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/deleted-branch", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/heads/deleted-branch", "2026-08-04T00:00:00Z"),
            ],
        )
        # 세대가 keep=2 이내여도 ref 가 없으면 전량 삭제한다.
        self.assertEqual(sorted(out["deleted"]), [1, 2])

    def test_deletes_cache_of_closed_pull_request(self):
        out = self.run_sweep(
            openPrs=[{"number": 10}],
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/pull/10/merge", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/pull/99/merge", "2026-08-05T00:00:00Z"),
            ],
        )
        self.assertEqual(out["deleted"], [2], "열린 PR 은 보호하고 닫힌 PR 만 지운다")

    def test_open_state_not_merge_state_decides_protection(self):
        """merge·단순 close·체리픽 후 close 를 구분하지 않는다.

        열려 있지 않은 PR 의 `refs/pull/<n>/merge` 로는 더 이상 CI 가 돌지 않고 그
        캐시를 다른 ref 가 읽지도 못한다. 실제로 #3779·#3775 는 내용이 통합 PR #3801 로
        반영되고 CLOSED 된 경우인데, 캐시는 merge 된 #3919 와 똑같이 고아여야 한다.
        """
        out = self.run_sweep(
            openPrs=[{"number": 40}],
            closedPrs=[{"number": 41}, {"number": 42}, {"number": 43}],
            caches=[
                # 열려 있음 — 보호
                cache(1, "grp-aaaaaaaa", "refs/pull/40/merge", "2026-08-05T00:00:00Z"),
                # merge 후 닫힘
                cache(2, "grp-bbbbbbbb", "refs/pull/41/merge", "2026-08-05T00:00:00Z"),
                # 체리픽·통합 PR 로 반영되고 merge 없이 닫힘
                cache(3, "grp-cccccccc", "refs/pull/42/merge", "2026-08-05T00:00:00Z"),
                # 그냥 닫힘
                cache(4, "grp-dddddddd", "refs/pull/43/merge", "2026-08-05T00:00:00Z"),
            ],
        )
        self.assertEqual(sorted(out["deleted"]), [2, 3, 4])

    def test_protects_both_merge_and_head_refs_of_open_pull_requests(self):
        """열린 PR 은 `/merge` 뿐 아니라 `/head` 캐시도 보호한다.

        이 저장소의 캐시 ref 는 실측상 `/merge` 뿐이지만(2026-08-06), 고아 정리가
        생기면서 `/head` 캐시는 세대 상한이 아니라 전량 삭제 대상이 됐다. 가정이
        깨지는 날 조용히 열린 PR 의 캐시가 사라지지 않도록 계약으로 고정한다.
        """
        out = self.run_sweep(
            openPrs=[{"number": 7}],
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/pull/7/merge", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/pull/7/head", "2026-08-05T00:00:00Z"),
                cache(3, "grp-cccccccc", "refs/pull/8/head", "2026-08-05T00:00:00Z"),
            ],
        )
        self.assertEqual(out["deleted"], [3], "닫힌 PR 의 /head 만 지운다")

    def test_keeps_cache_on_existing_tag_ref(self):
        out = self.run_sweep(
            tags=[{"name": "v1.2.3"}],
            caches=[cache(1, "grp-aaaaaaaa", "refs/tags/v1.2.3", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["deleted"], [])

    def test_reads_caches_before_refs_to_avoid_race(self):
        # 캐시는 자기 ref 보다 먼저 생길 수 없다. 캐시를 먼저 읽어야 조회 사이에 열린
        # PR·브랜치의 캐시를 고아로 오인하지 않는다.
        out = self.run_sweep(
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["calls"][0], "caches", out["calls"])
        self.assertIn("pulls", out["calls"])
        self.assertLess(out["calls"].index("caches"), out["calls"].index("pulls"))
        self.assertLess(out["calls"].index("caches"), out["calls"].index("branches"))

    # --- fail-closed 가드 ---

    def test_skips_orphan_sweep_when_branch_list_is_empty(self):
        out = self.run_sweep(
            branches=[],
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["deleted"], [], "목록을 못 믿으면 아무것도 지우지 않는다")
        self.assertTrue(any("건너뛴다" in w for w in out["warnings"]), out["warnings"])

    def test_skips_orphan_sweep_when_ref_lookup_fails(self):
        out = self.run_sweep(
            branchesThrow=True,
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["deleted"], [])
        self.assertTrue(any("조회 실패" in w for w in out["warnings"]), out["warnings"])

    def test_orphan_sweep_can_be_disabled(self):
        out = self.run_sweep(
            env={"SWEEP_ORPHAN_REFS": "false"},
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["deleted"], [])

    # --- 세대 상한 (#3684 기존 계약) ---

    def test_keeps_latest_generations_per_ref_and_group(self):
        out = self.run_sweep(
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/heads/devel", "2026-08-04T00:00:00Z"),
                cache(3, "grp-cccccccc", "refs/heads/devel", "2026-08-03T00:00:00Z"),
            ],
        )
        self.assertEqual(out["deleted"], [3], "최신 2세대를 남기고 가장 오래된 것만 지운다")

    def test_generation_limit_is_per_ref(self):
        out = self.run_sweep(
            branches=[{"name": "devel"}, {"name": "main"}],
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/heads/devel", "2026-08-04T00:00:00Z"),
                cache(3, "grp-cccccccc", "refs/heads/main", "2026-08-03T00:00:00Z"),
            ],
        )
        self.assertEqual(out["deleted"], [], "ref 가 다르면 서로의 세대를 잠식하지 않는다")

    def test_dry_run_deletes_nothing(self):
        out = self.run_sweep(
            env={"DRY_RUN": "true"},
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z"),
                cache(2, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z"),
                cache(3, "grp-bbbbbbbb", "refs/heads/devel", "2026-08-04T00:00:00Z"),
                cache(4, "grp-cccccccc", "refs/heads/devel", "2026-08-03T00:00:00Z"),
            ],
        )
        self.assertEqual(out["deleted"], [])
        self.assertTrue(any("(예정)" in line for line in out["info"]), out["info"])

    def test_delete_failure_is_a_warning_not_a_crash(self):
        out = self.run_sweep(
            deleteFails=[1],
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z")],
        )
        self.assertEqual(out["deleted"], [])
        self.assertTrue(any("삭제 실패" in w for w in out["warnings"]), out["warnings"])

    # --- 한도 경보 (#4080 제안 3) ---

    def test_fails_when_post_sweep_total_exceeds_fail_threshold(self):
        out = self.run_sweep(
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z", mb=9800)],
        )
        self.assertIsNotNone(out["failed"], "한도의 95% 초과는 실패로 드러낸다")

    def test_warns_when_post_sweep_total_exceeds_warn_threshold(self):
        out = self.run_sweep(
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z", mb=8600)],
        )
        self.assertIsNone(out["failed"])
        self.assertTrue(
            any("경고 임계" in w for w in out["warnings"]), out["warnings"]
        )

    def test_quiet_when_total_is_below_thresholds(self):
        out = self.run_sweep(
            caches=[cache(1, "grp-aaaaaaaa", "refs/heads/devel", "2026-08-05T00:00:00Z", mb=1000)],
        )
        self.assertIsNone(out["failed"])
        self.assertEqual([w for w in out["warnings"] if "임계" in w], [])

    def test_threshold_uses_post_sweep_total_not_pre_sweep(self):
        # 정리 전에는 한도를 넘지만 고아를 지우고 나면 임계 아래로 내려간다.
        out = self.run_sweep(
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z", mb=8000),
                cache(2, "grp-bbbbbbbb", "refs/heads/devel", "2026-08-05T00:00:00Z", mb=1000),
            ],
        )
        self.assertEqual(out["deleted"], [1])
        self.assertIsNone(out["failed"])
        self.assertEqual([w for w in out["warnings"] if "임계" in w], [])

    # --- summary 계약 ---

    def test_summary_reports_orphan_and_generation_separately(self):
        out = self.run_sweep(
            caches=[
                cache(1, "grp-aaaaaaaa", "refs/heads/gone", "2026-08-05T00:00:00Z"),
                cache(2, "grp-bbbbbbbb", "refs/heads/devel", "2026-08-05T00:00:00Z"),
                cache(3, "grp-cccccccc", "refs/heads/devel", "2026-08-04T00:00:00Z"),
                cache(4, "grp-dddddddd", "refs/heads/devel", "2026-08-03T00:00:00Z"),
            ],
        )
        labels = [row[0] for row in out["summary"] if isinstance(row[0], str)]
        for expected in ["고아 ref", "구 세대", "한도 대비", "고아 ref 정리"]:
            self.assertIn(expected, labels)


class BooleanInputExpressionTests(unittest.TestCase):
    """[#4080] boolean 입력의 YAML 표현식 형태를 단언한다.

    JS 하네스는 env 를 직접 주입하므로 YAML 표현식은 그 검증 범위 밖이다. 실제로
    `SWEEP_ORPHAN_REFS` 가 `A && inputs.x || 'true'` 형태여서 false 를 넣어도 켜졌는데,
    `test_orphan_sweep_can_be_disabled` 는 통과했다. 스위치가 죽은 것을 테스트가
    못 잡은 것이다.

    GitHub Actions 표현식에서 `A && B` 는 A 가 truthy 면 B, 아니면 A 를 준다.
    `X || Y` 는 X 가 truthy 면 X, 아니면 Y 다. 따라서 기본값이 true 인 boolean 입력에
    `A && inputs.x || 'true'` 를 쓰면 `true && false` → `false` → `|| 'true'` → `'true'`
    가 되어 끄려던 값이 켠 값이 된다.

    안전한 형태는 fallback 없이 쓰는 것이다. 표현식의 falsy 결과가 그대로 'false' 로
    렌더되므로 기본값이 저절로 맞는다.
    """

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def _boolean_inputs(self) -> list[str]:
        """`type: boolean` 으로 선언된 workflow_dispatch 입력 이름."""
        block = self.workflow.split("  workflow_dispatch:", maxsplit=1)[1]
        block = block.split("\npermissions:", maxsplit=1)[0]
        names = []
        for match in re.finditer(
            r"(?m)^      (?P<name>[a-z0-9_]+):\n(?P<body>(?:^        .*\n)+)", block
        ):
            if "type: boolean" in match.group("body"):
                names.append(match.group("name"))
        return names

    def _env_expression(self, env_name: str) -> str:
        match = re.search(
            rf"(?m)^\s*{re.escape(env_name)}:\s*(\$\{{\{{.*?\}}\}})\s*$", self.workflow
        )
        self.assertIsNotNone(match, f"{env_name} env 를 찾지 못했다")
        return match.group(1) if match else ""

    def test_boolean_inputs_are_declared(self):
        self.assertEqual(
            sorted(self._boolean_inputs()), ["dry_run", "sweep_orphan_refs"]
        )

    def test_boolean_input_expressions_have_no_literal_fallback(self):
        """`|| '<literal>'` fallback 이 붙으면 false 가 되살아난다."""
        for input_name in self._boolean_inputs():
            expression = self._env_expression(input_name.upper())
            self.assertIn(f"inputs.{input_name}", expression)
            self.assertNotRegex(
                expression,
                r"\|\|\s*'(?:true|false)'",
                f"{input_name}: boolean 입력에 리터럴 fallback 을 쓰면 dispatch 에서 "
                f"false 가 무시된다. `event == 'workflow_dispatch' && inputs.x` 또는 "
                f"`event != 'workflow_dispatch' || inputs.x` 형태로 쓴다. 표현식: {expression}",
            )

    def test_sweep_orphan_refs_defaults_on_for_cron_and_can_be_turned_off(self):
        expression = self._env_expression("SWEEP_ORPHAN_REFS")
        # cron 기본 true 이므로 `!=` 형태여야 한다.
        self.assertIn("github.event_name != 'workflow_dispatch'", expression)
        self.assertIn("|| inputs.sweep_orphan_refs", expression)

    def test_dry_run_defaults_off_for_cron(self):
        expression = self._env_expression("DRY_RUN")
        # cron 기본 false 이므로 `==` 형태여야 한다.
        self.assertIn("github.event_name == 'workflow_dispatch'", expression)
        self.assertIn("&& inputs.dry_run", expression)


class LimitUnitTests(unittest.TestCase):
    """[#4080] 한도를 바이트로 명시하고 십진 해석을 고정한다.

    GitHub 문서는 캐시 한도를 "10 GB" 라고만 쓰고 십진(10^9)인지 이진(2^30)인지
    밝히지 않는다. 둘의 차이는 7.4% 라 임계 발화 시점이 달라진다. 2026-08-06 실측
    10,241,001,878 B 는 십진으로 102.4%, 이진으로 95.4% 로 읽힌다 — 한쪽은 한도 초과이고
    다른 쪽은 아니다.

    쿼터 가드는 늦게 우는 것보다 일찍 우는 편이 안전하므로 보수적인 십진을 쓴다.
    `LIMIT_GB` 처럼 단위가 코드에 숨는 형태로 되돌아가지 않게 계약으로 고정한다.
    """

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")

    def test_limit_is_declared_in_bytes(self):
        self.assertIn('LIMIT_BYTES: "10000000000"', self.workflow)
        self.assertNotIn("LIMIT_GB", self.workflow)

    def test_script_reads_the_byte_limit_without_unit_multiplication(self):
        match = re.search(r"const limitBytes = (.+);", self.workflow)
        self.assertIsNotNone(match)
        expression = match.group(1) if match else ""
        self.assertIn("LIMIT_BYTES", expression)
        self.assertNotIn("1024", expression, "한도에 이진 배수를 다시 곱하면 안 된다")

    def test_summary_states_the_raw_byte_limit(self):
        # 표시는 GiB 로 유지하되(#3684 이후 시계열과 대조 가능), 한도의 원시 바이트를
        # 함께 남겨 어떤 해석을 썼는지 로그만 보고 알 수 있게 한다.
        self.assertIn("limitBytes} B", self.workflow)


if __name__ == "__main__":
    unittest.main()
