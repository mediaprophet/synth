import copy, hashlib, json, subprocess, sys, tempfile, unittest
from pathlib import Path
from production_core_artifacts import (
    DETERMINISM_BOUNDARY, INCOMPLETE_CORE_CHECKLIST, LOWERHEX_HISTORY, attributed_glb,
    capability_record, check_affine, classify_capability, documented_commands,
    inspect_whole_surface_bound, sha256_lowerhex,
)
from admitted_geometry_probe import discover_worktrees, inspect_admitted_geometry, invoke_admitted_geometry

class IndependentAdmissionTests(unittest.TestCase):
    def test_oracle_rejects_nonfinite_sources_and_outputs(self):
        for bad in [float('nan'),float('inf'),-float('inf'),2.0**200,True]:
            with self.subTest(value=bad):
                with self.assertRaises(ValueError):check_affine([[bad,2,3]],[[2.25,4.25,6.25]])
                with self.assertRaises(ValueError):check_affine([[1,2,3]],[[bad,4.25,6.25]])

    def test_oracle_detects_corruption_and_count_mismatch(self):
        self.assertEqual(check_affine([[1,2,3]],[[2.25,4.25,6.25]])['compared'],3)
        for output in [[[2.25,4.25,6.5]],[],[[2.25,4.25]]]:
            with self.assertRaises(ValueError):check_affine([[1,2,3]],output)

    def fixture(self):
        vertices=[[0.,0.,0.],[.5,0.,0.],[.5,1.,0.],[.5,0.,0.],[1.,0.,0.],[.5,1.,0.]]
        attrs=[dict(uv=[0.,0.],normal=[0.,0.,1.],subpart_tag=101 if i<3 else 202) for i in range(6)]
        return vertices,[[0,1,2],[3,4,5]],attrs

    def test_glb_refuses_invalid_attributes_and_mixed_domains(self):
        vertices,triangles,attrs=self.fixture()
        self.assertTrue(attributed_glb(vertices,triangles,attrs).startswith(b'glTF'))
        for key,bad in [('uv',[float('nan'),0.]),('uv',[0.]),('normal',[0.,float('inf'),1.]),('normal',[0.,1.]),('uv',[1e7,0.])]:
            changed=copy.deepcopy(attrs);changed[0][key]=bad
            with self.assertRaises(ValueError):attributed_glb(vertices,triangles,changed)
        with self.assertRaises(ValueError):attributed_glb(vertices,[[0,1,3],[3,4,5]],attrs)
        with self.assertRaises(ValueError):attributed_glb(vertices,[[False,1,2],[3,4,5]],attrs)

class AdmittedPathTests(unittest.TestCase):
    def test_capability_states_are_distinct(self):
        self.assertEqual(classify_capability(),'absent')
        self.assertEqual(classify_capability(present=True),'present')
        self.assertEqual(classify_capability(present=True,is_callable=True),'callable')
        self.assertEqual(classify_capability(present=True,is_callable=True,executed=True),'executed')
        self.assertEqual(classify_capability(present=True,is_callable=True,executed=True,qualified=True),'qualified')
        with self.assertRaises(ValueError):classify_capability(executed=True)
        with self.assertRaises(ValueError):classify_capability(qualified=True)
        with self.assertRaises(ValueError):classify_capability(present=True,is_callable=True,qualified=True)
        record=capability_record(present=True,is_callable=False)
        self.assertEqual(record['state'],'present')
        self.assertFalse(record['callable'])
        self.assertFalse(record['executed'])
        self.assertFalse(record['qualified'])

    def test_sha256_lowerhex_matches_hashlib_and_records_history(self):
        payload=b'format!("{:x}", Sha256::digest(bytes))'
        self.assertEqual(sha256_lowerhex(payload),hashlib.sha256(payload).hexdigest())
        self.assertTrue(sha256_lowerhex(payload).islower())
        self.assertEqual(len(sha256_lowerhex(payload)),64)
        self.assertEqual(LOWERHEX_HISTORY['repair']['status'],'rebuilt')
        self.assertEqual(LOWERHEX_HISTORY['repair']['passing_receipt'],'tracer_run_not_qualification')
        self.assertEqual(LOWERHEX_HISTORY['previous_failure']['error'],'E0277')
        self.assertIn('not three independent GPU runs',DETERMINISM_BOUNDARY)

    def test_documented_commands_cover_geometry_probe_and_glb_smoke(self):
        commands=documented_commands(discover_worktrees())
        self.assertEqual([c['id'] for c in commands],[
            'qualiadb-production-core-geometry',
            'etalon-production-core-probe',
            'producer-glb-smoke',
        ])

    def test_inspect_absent_tree_is_not_qualified(self):
        with tempfile.TemporaryDirectory() as tmp:
            report=inspect_admitted_geometry(worktrees=discover_worktrees(tmp))
            lib=report['capabilities']['admitted_geometry_library']
            self.assertEqual(lib['state'],'absent')
            self.assertFalse(lib['qualified'])
            self.assertFalse(report['invented_passing_geometry_receipt'])
            self.assertFalse(report['gpu_identical_timing_claimed'])
            self.assertTrue(report['joined_three_geometry_builds_are_not_three_gpu_runs'])

    def test_inspect_source_without_binary_is_present_not_executed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            admission=root/'qualiaDB'/'crates'/'qualia-core-db'/'src'/'specialized_libs'/'computational_geometry'/'decimate_3_semantic'/'admission.rs'
            example=root/'qualiaDB'/'crates'/'qualia-core-db'/'examples'/'production_core_geometry.rs'
            semantic=root/'qualiaDB'/'crates'/'qualia-core-db'/'src'/'specialized_libs'/'computational_geometry'/'decimate_3_semantic.rs'
            admission.parent.mkdir(parents=True);example.parent.mkdir(parents=True)
            admission.write_text('pub fn decimate_semantic_admitted_3() {}\n',encoding='utf-8')
            semantic.write_text('pub mod admission;\n',encoding='utf-8')
            example.write_text('decimate_semantic_admitted_3(\nconst HEX: &[u8; 16] = b"0123456789abcdef";\nSha256::digest(bytes)\n',encoding='utf-8')
            report=inspect_admitted_geometry(worktrees=discover_worktrees(root))
            lib=report['capabilities']['admitted_geometry_library']
            example_cap=report['capabilities']['admitted_geometry_example']
            self.assertEqual(lib['state'],'present')
            self.assertTrue(example_cap['source_wired_to_admitted_driver'])
            self.assertTrue(example_cap['lowerhex_repair_in_source'])
            self.assertFalse(example_cap['callable'])
            self.assertFalse(example_cap['executed'])
            self.assertFalse(example_cap['qualified'])

    def test_missing_geometry_exe_invoke_does_not_invent_a_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            result=invoke_admitted_geometry(Path(tmp)/'missing.exe',Path(tmp)/'geometry-1')
            self.assertEqual(result['status'],'missing_executable')
            self.assertNotIn(result['status'],('passed','qualified'))

    def test_live_worktree_library_present_binary_callable_not_qualified(self):
        report=inspect_admitted_geometry()
        lib=report['capabilities']['admitted_geometry_library']
        example=report['capabilities']['admitted_geometry_example']
        if not (Path(report['worktrees']['qualiaDB'])/'crates'/'qualia-core-db'/'examples'/'production_core_geometry.rs').is_file():
            self.skipTest('sibling QualiaDB worktree not present')
        self.assertTrue(lib['present'])
        self.assertTrue(example['source_wired_to_admitted_driver'])
        self.assertTrue(example['lowerhex_repair_in_source'])
        exe=Path(report['worktrees']['qualiaDB'])/'target-production-core'/'debug'/'examples'/'production_core_geometry.exe'
        if exe.is_file():
            self.assertTrue(example['callable'])
            self.assertIn(example['state'],('callable','present'))
        else:
            self.assertFalse(example['callable'])
            self.assertEqual(example['state'],'present')
        self.assertFalse(example['executed'])
        self.assertFalse(example['qualified'])
        self.assertEqual(report['sha256_lowerhex_encoding']['digest'],report['sha256_lowerhex_encoding']['hashlib_hexdigest'])
        self.assertNotEqual(lib['state'],'qualified')
        example_src = Path(report['worktrees']['qualiaDB'])/'crates'/'qualia-core-db'/'examples'/'production_core_geometry.rs'
        text = example_src.read_text(encoding='utf-8')
        driver_in_source = '"--driver"' in text
        self.assertEqual(bool(report['sources']['example']['required_snippets'].get('--driver')), '--driver' in text)
        geometric = report['capabilities']['geometric_qem_driver']
        self.assertEqual(geometric['callable'], bool(driver_in_source and exe.is_file()))
        self.assertFalse(geometric['executed'])
        self.assertFalse(geometric['qualified'])
        self.assertEqual(geometric['planned_argv'], ['--driver', 'geometric'])
        self.assertEqual(geometric['driver_flag_in_example_source'], '--driver' in text)
        by_item = {row['item']: row['status'] for row in report['incomplete_core_checklist']}
        self.assertEqual(by_item['whole_surface_bound'], 'not_measured')
        self.assertEqual(by_item['collapse_provenance_qualification'], 'inspected_not_supplied')
        self.assertEqual(by_item['p01_p03_accepted'], 'open')
        self.assertNotEqual(by_item['whole_surface_bound'], 'qualified')

    def test_checklist_and_enclosing_bound_inspect_stay_honest(self):
        by_item = {row['item']: row['status'] for row in INCOMPLETE_CORE_CHECKLIST}
        self.assertEqual(by_item['whole_surface_bound'], 'not_measured')
        self.assertEqual(by_item['collapse_provenance_qualification'], 'inspected_not_supplied')
        self.assertEqual(by_item['operator_or_physical_qualification'], 'not_executed')
        self.assertEqual(by_item['p01_p03_accepted'], 'open')
        self.assertNotIn('qualified', by_item.values())
        missing = inspect_whole_surface_bound(Path('definitely-missing-geometry-receipt.json'))
        self.assertEqual(missing['status'], 'not_supplied')
        self.assertIsNone(missing['class'])
        self.assertIsNone(missing['sha256'])
        self.assertFalse(missing['qualified'])
        live = Path(__file__).resolve().parents[2]/'docs'/'production-core'/'evidence'/'admitted-geometry-run-01'/'geometry-receipt.json'
        if live.is_file():
            report = inspect_whole_surface_bound(live)
            self.assertEqual(report['status'], 'not_measured')
            self.assertNotEqual(report['class'], 'enclosing_bound')
            self.assertFalse(report['qualified'])

    def test_tracer_inspect_only_writes_honest_receipt(self):
        script=Path(__file__).resolve().parent/'run_production_core_tracer.py'
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp)/'inspect-01'
            process=subprocess.run(
                [sys.executable,str(script),'--inspect-only','--out',str(out)],
                cwd=str(script.parent),capture_output=True,text=True,encoding='utf-8',
            )
            self.assertEqual(process.returncode,0,process.stderr)
            receipt=json.loads((out/'joined-receipt.json').read_text(encoding='utf-8'))
            self.assertEqual(receipt['status'],'inspected_not_executed')
            self.assertNotIn('passed',receipt['status'])
            self.assertFalse(receipt['invented_passing_geometry_receipt'])
            self.assertFalse(receipt['gpu_identical_timing_claimed'])
            self.assertTrue(receipt['joined_three_geometry_builds_are_not_three_gpu_runs'])
            self.assertEqual(receipt['sha256_lowerhex_history']['repair']['status'],'rebuilt')
            ids=[c['id'] for c in receipt['documented_commands']]
            self.assertEqual(ids,[
                'qualiadb-production-core-geometry',
                'etalon-production-core-probe',
                'producer-glb-smoke',
            ])
            lib_state=receipt['capability_states']['admitted_geometry_library']['state']
            self.assertIn(lib_state,('present','callable'))
            self.assertFalse(receipt['capability_states']['admitted_geometry_library']['qualified'])

    def test_joined_tracer_missing_geometry_exe_fails_honestly(self):
        script=Path(__file__).resolve().parent/'run_production_core_tracer.py'
        with tempfile.TemporaryDirectory() as tmp:
            missing=Path(tmp)/'missing-geometry.exe'
            forge=Path(tmp)/'forge.exe';forge.write_bytes(b'not-a-real-exe')
            synth=Path(tmp)/'synth.exe';synth.write_bytes(b'not-a-real-exe')
            out=Path(tmp)/'joined-missing'
            process=subprocess.run(
                [sys.executable,str(script),'--geometry-exe',str(missing),'--forge-exe',str(forge),'--synth-exe',str(synth),'--out',str(out)],
                cwd=str(script.parent),capture_output=True,text=True,encoding='utf-8',
            )
            self.assertNotEqual(process.returncode,0)
            receipt=json.loads((out/'joined-receipt.json').read_text(encoding='utf-8'))
            self.assertEqual(receipt['status'],'failed')
            self.assertTrue('missing executable' in receipt['reason'] or 'LowerHex' in receipt['reason'])
            self.assertFalse(receipt['invented_passing_geometry_receipt'])
            self.assertNotEqual(receipt['status'],'passed')

if __name__=='__main__':unittest.main()
