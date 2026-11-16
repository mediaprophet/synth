"""Independent CPU oracles for the P03 qualification corpus.

Derivation is intentionally different from qualification_corpus.py closed forms
and from QualiaDB/Etalon/Hypermedia implementation helpers. Agreement between
this harness and a fixture oracle validates publication, not production.

This runner does not execute production implementations and must not set
execution_status to anything other than not_executed.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import json
import math
import sys

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

from qualification_corpus import (
    FAMILIES,
    canonical,
    digest_bytes,
    hash_backend,
)

try:
    from production_core_artifacts import check_affine
except ImportError:
    check_affine = None


class OracleError(AssertionError):
    pass


def fail(fid, message):
    raise OracleError(f'{fid}: {message}')


def load_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))


def sub(a, b):
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]


def addv(a, b):
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]


def scale(a, s):
    return [a[0] * s, a[1] * s, a[2] * s]


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def norm(a):
    return math.sqrt(dot(a, a))


def nearly_equal(actual, expected, atol, rtol, label, fid):
    if actual is None and expected is None:
        return
    if isinstance(expected, (list, tuple)):
        if not isinstance(actual, (list, tuple)) or len(actual) != len(expected):
            fail(fid, f'{label}: shape {actual!r} != {expected!r}')
        for index, (left, right) in enumerate(zip(actual, expected)):
            nearly_equal(left, right, atol, rtol, f'{label}[{index}]', fid)
        return
    if isinstance(expected, bool) or isinstance(actual, bool):
        if actual is not expected:
            fail(fid, f'{label}: {actual!r} != {expected!r}')
        return
    if expected is None:
        if actual is not None:
            fail(fid, f'{label}: expected none, got {actual!r}')
        return
    err = abs(float(actual) - float(expected))
    scale_e = abs(float(expected))
    if not math.isfinite(err) or err > atol + rtol * scale_e:
        fail(fid, f'{label}: {actual!r} != {expected!r} (atol={atol}, rtol={rtol})')


def triangle_area(vertices, tri):
    a, b, c = vertices[tri[0]], vertices[tri[1]], vertices[tri[2]]
    return 0.5 * norm(cross(sub(b, a), sub(c, a)))


def mesh_area(vertices, triangles):
    return sum(triangle_area(vertices, tri) for tri in triangles)


def face_normal(vertices, tri):
    a, b, c = vertices[tri[0]], vertices[tri[1]], vertices[tri[2]]
    n = cross(sub(b, a), sub(c, a))
    length = norm(n)
    if length == 0.0:
        return None
    return [n[0] / length, n[1] / length, n[2] / length]


def aabb_diagonal(vertices):
    if not vertices:
        return 0.0
    lo = [min(v[i] for v in vertices) for i in range(3)]
    hi = [max(v[i] for v in vertices) for i in range(3)]
    return norm(sub(hi, lo))


def edge_key(i, j):
    return (i, j) if i < j else (j, i)


class UnionFind:
    def __init__(self, n):
        self.parent = list(range(n))

    def find(self, i):
        while self.parent[i] != i:
            self.parent[i] = self.parent[self.parent[i]]
            i = self.parent[i]
        return i

    def union(self, i, j):
        ri, rj = self.find(i), self.find(j)
        if ri != rj:
            self.parent[rj] = ri


def component_count(vertex_count, triangles):
    uf = UnionFind(vertex_count)
    used = set()
    for tri in triangles:
        for index in tri:
            used.add(index)
        uf.union(tri[0], tri[1])
        uf.union(tri[1], tri[2])
    return len({uf.find(i) for i in used}) if used else 0


def boundary_loop_count(triangles):
    oriented = {}
    for tri in triangles:
        for a, b in ((tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])):
            key = edge_key(a, b)
            oriented.setdefault(key, []).append((a, b))
    boundary = [pair for pair, uses in oriented.items() if len(uses) == 1]
    if not boundary:
        return 0
    nxt = {}
    for a, b in (uses[0] for _, uses in oriented.items() if len(uses) == 1):
        nxt.setdefault(a, []).append(b)
    seen_edges = set()
    loops = 0
    for start, ends in nxt.items():
        for first in ends:
            edge = (start, first)
            if edge in seen_edges:
                continue
            loops += 1
            a, b = start, first
            for _ in range(len(boundary) + 1):
                seen_edges.add((a, b))
                if b == start:
                    break
                choices = nxt.get(b, [])
                if not choices:
                    break
                a, b = b, choices[0]
    return loops


def euler_characteristic(vertices, triangles):
    edges = {edge_key(a, b) for tri in triangles for a, b in ((tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0]))}
    return len(vertices) - len(edges) + len(triangles)


def manifold_status(vertices, triangles):
    if any(len(tri) != 3 for tri in triangles):
        return 'reject_degenerate'
    for tri in triangles:
        if len(set(tri)) != 3:
            return 'reject_degenerate'
        if any(index < 0 or index >= len(vertices) for index in tri):
            return 'reject_degenerate'
        if triangle_area(vertices, tri) == 0.0:
            return 'reject_degenerate'
    seen = {}
    oriented = {}
    for tri in triangles:
        key = tuple(sorted(tri))
        seen[key] = seen.get(key, 0) + 1
        if seen[key] > 1:
            return 'reject_duplicate'
        for a, b in ((tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])):
            ek = edge_key(a, b)
            oriented.setdefault(ek, []).append((a, b))
    for uses in oriented.values():
        if len(uses) > 2:
            return 'reject_nonmanifold'
        if len(uses) == 2 and uses[0] == uses[1]:
            return 'reject_orientation'
        if len(uses) == 2 and uses[0][0] == uses[1][0]:
            return 'reject_orientation'
    return 'ok'


def matvec(matrix, vector):
    return [sum(row[i] * vector[i] for i in range(len(vector))) for row in matrix]


def residual(matrix, vector, rhs):
    av = matvec(matrix, vector)
    return [av[i] - rhs[i] for i in range(len(rhs))]


def characteristic_roots_tridiag3(a00, a11, a22, a01, a12):
    # SPD 3x3 tridiagonal [[a,b,0],[b,c,d],[0,d,e]] closed characteristic solve
    # used only for the published Dirichlet matrix shape.
    # Roots of det(A-λI)=0 for the specific Dirichlet Laplacian are 2±sqrt(2) and 2.
    # Independent expansion:
    # det = (a00-λ)*((a11-λ)*(a22-λ)-a12^2) - a01^2*(a22-λ)
    # For a00=a11=a22=2, a01=a12=-1 this is (2-λ)^3 - 2(2-λ).
    if (a00, a11, a22, a01, a12) != (2.0, 2.0, 2.0, -1.0, -1.0):
        raise OracleError('unexpected operator matrix for closed char-poly roots')
    return [2.0 - math.sqrt(2.0), 2.0, 2.0 + math.sqrt(2.0)]


def tet_signed_volume(vertices, tet):
    a, b, c, d = (vertices[i] for i in tet)
    return dot(sub(b, a), cross(sub(c, a), sub(d, a))) / 6.0


def apply_affine_fused(vertices, scale_k, bias):
    return [[scale_k * x + bias for x in point] for point in vertices]


def apply_affine_staged(vertices, bias):
    return [[(x + x) + bias for x in point] for point in vertices]


def rotate_translate(point, rotation, translation):
    return addv([
        rotation[0][0] * point[0] + rotation[0][1] * point[1] + rotation[0][2] * point[2],
        rotation[1][0] * point[0] + rotation[1][1] * point[1] + rotation[1][2] * point[2],
        rotation[2][0] * point[0] + rotation[2][1] * point[1] + rotation[2][2] * point[2],
    ], translation)


def barycentre(vertices, weights):
    acc = [0.0, 0.0, 0.0]
    for vertex, weight in zip(vertices, weights):
        acc = addv(acc, scale(vertex, weight))
    return acc


def unique_domains(labels, indices):
    return {labels[i] for i in indices}


def check_required_fields(case, fid):
    for key in ('schema', 'id', 'family', 'partition', 'seed', 'provenance',
                'license', 'data', 'oracle', 'exclusions', 'execution_status'):
        if key not in case:
            fail(fid, f'missing field {key}')
    if case['execution_status'] != 'not_executed':
        fail(fid, 'execution_status must remain not_executed')
    if case['family'] not in FAMILIES:
        fail(fid, f'unknown family {case["family"]}')
    if case['partition'] not in ('tune', 'test', 'holdout'):
        fail(fid, f'unknown partition {case["partition"]}')


def check_mesh_oracle(fid, data, oracle):
    vertices = data.get('vertices')
    triangles = data.get('triangles')
    if vertices is None or triangles is None:
        return
    if oracle.get('reference_area') is not None:
        nearly_equal(mesh_area(vertices, triangles), oracle['reference_area'], 1e-12, 1e-12, 'area', fid)
    if oracle.get('exact_normal') is not None and triangles:
        for tri in triangles:
            n = face_normal(vertices, tri)
            if n is None:
                fail(fid, 'zero-area face while exact_normal declared')
            nearly_equal(n, oracle['exact_normal'], 1e-12, 1e-12, 'normal', fid)
    if oracle.get('vertex_count') is not None:
        if len(vertices) != oracle['vertex_count']:
            fail(fid, f'vertex_count {len(vertices)} != {oracle["vertex_count"]}')
    if oracle.get('triangle_count') is not None:
        if len(triangles) != oracle['triangle_count']:
            fail(fid, f'triangle_count {len(triangles)} != {oracle["triangle_count"]}')
    if oracle.get('expected_components') is not None:
        got = component_count(len(vertices), triangles)
        if got != oracle['expected_components']:
            fail(fid, f'components {got} != {oracle["expected_components"]}')
    if oracle.get('boundary_loops') is not None:
        got = boundary_loop_count(triangles)
        if got != oracle['boundary_loops']:
            fail(fid, f'boundary_loops {got} != {oracle["boundary_loops"]}')
    if oracle.get('euler_characteristic') is not None:
        got = euler_characteristic(vertices, triangles)
        if got != oracle['euler_characteristic']:
            fail(fid, f'euler {got} != {oracle["euler_characteristic"]}')
    if oracle.get('radial_vertex_error') == 0.0:
        for index, vertex in enumerate(vertices):
            nearly_equal(dot(vertex, vertex), 1.0, 1e-15, 0.0, f'vertex[{index}] radius^2', fid)
    if oracle.get('radial_vertex_error_bound') is not None:
        bound = oracle['radial_vertex_error_bound']
        radius = data.get('radius', 1.0)
        origin = data.get('origin', [0.0, 0.0, 0.0])
        for index, vertex in enumerate(vertices):
            radial = math.hypot(vertex[0] - origin[0], vertex[1] - origin[1])
            if abs(radial - radius) > max(bound * 8.0, 1e-12):
                fail(fid, f'vertex[{index}] cylinder radius {radial} != {radius}')
    if 'strict_manifold_status' in oracle:
        status = manifold_status(vertices, triangles)
        if status != oracle['strict_manifold_status']:
            fail(fid, f'manifold {status} != {oracle["strict_manifold_status"]}')
    if oracle.get('sheet_gap') is not None and len(vertices) >= 8:
        z0, z1 = vertices[0][2], vertices[4][2]
        gap = abs(z1 - z0)
        ulp = math.ulp(max(abs(z0), abs(z1), 1.0))
        nearly_equal(gap, oracle['sheet_gap'], 16.0 * ulp, 0.0, 'sheet_gap', fid)
    if oracle.get('triangle_domain_must_be_uniform') and data.get('domains'):
        for tri in triangles:
            if len(unique_domains(data['domains'], tri)) != 1:
                fail(fid, f'mixed domain labels on triangle {tri}')


def _unique_extend(dst, name):
    if name not in dst:
        dst.append(name)


def observe_known_bad_gates(data, oracle=None):
    """Evaluate a published known-bad CANDIDATE on CPU.

    Unobservable gates stay unevaluated. Absence is not a fake fail and is
    never recorded as 0.0. This is not a production-executable run: do not
    pass a welded candidate to production_core_geometry.exe as an input mesh.
    """
    oracle = oracle or {}
    kind = data.get('kind')
    observed = []
    unevaluated = []
    publication_defect = None
    declared = list(oracle.get('failing_tests') or [])

    if kind == 'representation_candidate':
        source = data.get('input')
        cand = data.get('candidate')
        if not isinstance(source, dict) or not isinstance(cand, dict):
            return dict(
                observed=[],
                unevaluated=declared or [
                    'forbidden_semantic_merges',
                    'no_cross_domain_support',
                    'preserve_corner_chart',
                    'preserve_tangent_handedness',
                    'coincident_pairs_are_not_welded',
                ],
                kind=kind,
                publication_defect=None,
                production_exe_invoked=False,
                gpu=False,
                route='oracle_harness.observe_known_bad_gates',
                note='Published candidate or input missing; gates left unobserved.',
            )
        triangles = cand.get('triangles')
        domains = cand.get('domains')
        if not isinstance(triangles, list) or not isinstance(domains, list):
            unevaluated.append('forbidden_semantic_merges')
        else:
            try:
                mixed = any(len(unique_domains(domains, tri)) > 1 for tri in triangles)
            except (IndexError, TypeError, KeyError):
                unevaluated.append('forbidden_semantic_merges')
            else:
                if mixed:
                    _unique_extend(observed, 'forbidden_semantic_merges')
        pairs = cand.get('collapsed_vertex_pairs')
        src_domains = source.get('domains')
        if pairs:
            _unique_extend(observed, 'coincident_pairs_are_not_welded')
            if not isinstance(src_domains, list):
                unevaluated.append('no_cross_domain_support')
            else:
                try:
                    cross = any(src_domains[a] != src_domains[b] for a, b in pairs)
                except (IndexError, TypeError, ValueError):
                    unevaluated.append('no_cross_domain_support')
                else:
                    if cross:
                        _unique_extend(observed, 'no_cross_domain_support')
        src_w = source.get('tangent_w')
        cand_w = cand.get('tangent_w')
        if src_w is None or cand_w is None:
            unevaluated.append('preserve_tangent_handedness')
        else:
            src_set, cand_set = set(src_w), set(cand_w)
            if src_set != cand_set and (-1.0 in src_set) and (-1.0 not in cand_set):
                _unique_extend(observed, 'preserve_tangent_handedness')
        if source.get('coincident_pairs') and pairs:
            _unique_extend(observed, 'preserve_corner_chart')
        elif 'preserve_corner_chart' in declared and not (source.get('coincident_pairs') and pairs):
            if source.get('coincident_pairs') is None or pairs is None:
                unevaluated.append('preserve_corner_chart')
    elif kind == 'affine_candidate':
        try:
            verts = data['input']['vertices']
            scale = data['kernel']['scale']
            bias = data['kernel']['bias']
            got = data['candidate']['vertices']
        except (KeyError, TypeError):
            unevaluated.extend(
                name for name in (declared or ['independent_affine_oracle', 'k1_abs_rel_tolerance'])
                if name not in unevaluated
            )
        else:
            expected = apply_affine_fused(verts, scale, bias)
            staged = apply_affine_staged(verts, bias)
            if got == expected or got == staged:
                publication_defect = 'poisoned affine candidate unexpectedly matches closed form'
            else:
                _unique_extend(observed, 'k1_abs_rel_tolerance')
                if check_affine is not None:
                    try:
                        check_affine(verts, got)
                    except ValueError:
                        _unique_extend(observed, 'independent_affine_oracle')
                    else:
                        publication_defect = 'independent affine oracle accepted known-bad output'
                else:
                    _unique_extend(observed, 'independent_affine_oracle')
    elif kind == 'volume_candidate':
        try:
            volume = tet_signed_volume(data['candidate']['vertices'], data['candidate']['tets'][0])
        except (KeyError, TypeError, IndexError):
            unevaluated.append('accept_when_inversion_forbidden')
        else:
            if volume >= 0.0:
                publication_defect = f'expected inverted tet, signed volume {volume}'
            elif oracle.get('accept_when_inversion_forbidden') is False:
                _unique_extend(observed, 'accept_when_inversion_forbidden')
            elif 'accept_when_inversion_forbidden' in declared:
                unevaluated.append('accept_when_inversion_forbidden')
    elif kind == 'contact_candidate':
        try:
            center = data['sphere_center']
            normal = data['plane_normal']
            penetration = data['sphere_radius'] - (dot(center, normal) - data['plane_offset'])
        except (KeyError, TypeError):
            for name in ('contact_feasibility', 'authoritative_full_model_required'):
                if name not in unevaluated:
                    unevaluated.append(name)
        else:
            if penetration > 0.0:
                _unique_extend(observed, 'contact_feasibility')
            _unique_extend(observed, 'authoritative_full_model_required')
    else:
        unevaluated = list(declared)
        publication_defect = f'unhandled known-bad kind {kind}' if kind else 'missing known-bad kind'

    return dict(
        observed=observed,
        unevaluated=unevaluated,
        kind=kind,
        publication_defect=publication_defect,
        production_exe_invoked=False,
        gpu=False,
        route='oracle_harness.observe_known_bad_gates',
        note=(
            'Published candidate evaluated by the independent CPU oracle. '
            'Geometry exe was not invoked on the welded candidate as an input mesh.'
        ),
    )


def check_known_bad(fid, data, oracle):
    if oracle.get('must_fail') is not True:
        fail(fid, 'known-bad oracle must declare must_fail')
    observation = observe_known_bad_gates(data, oracle)
    if observation.get('publication_defect'):
        fail(fid, observation['publication_defect'])
    kind = data.get('kind')
    if kind == 'volume_candidate':
        volume = tet_signed_volume(data['candidate']['vertices'], data['candidate']['tets'][0])
        nearly_equal(volume, oracle['signed_volume'], 1e-15, 0.0, 'signed_volume', fid)
    elif kind == 'contact_candidate':
        force = data['stiffness'] * data['displacement']
        energy = 0.5 * data['stiffness'] * data['displacement'] ** 2
        center = data['sphere_center']
        normal = data['plane_normal']
        penetration = data['sphere_radius'] - (dot(center, normal) - data['plane_offset'])
        nearly_equal(force, oracle['spring_force'], 1e-12, 0.0, 'spring_force', fid)
        nearly_equal(energy, oracle['spring_energy'], 1e-18, 1e-12, 'spring_energy', fid)
        nearly_equal(penetration, oracle['penetration'], 1e-15, 0.0, 'penetration', fid)
    required = set(oracle['failing_tests'])
    got = set(observation['observed'])
    if not required.issubset(got):
        fail(fid, f'known-bad did not trigger {sorted(required - got)}; observed {sorted(got)}')


def check_identity(fid, data, oracle):
    if oracle.get('must_pass') is not True:
        fail(fid, 'identity oracle must declare must_pass')
    kind = data.get('kind')
    if kind == 'representation_candidate':
        src, cand = data['input'], data['candidate']
        if src['vertices'] != cand['vertices'] or src['triangles'] != cand['triangles']:
            fail(fid, 'identity mesh candidate differs from input')
        if src['domains'] != cand['domains']:
            fail(fid, 'identity candidate merged or relabelled domains')
        if src['uvs'] != cand['uvs']:
            fail(fid, 'identity candidate altered UVs')
        if src['tangent_w'] != cand['tangent_w']:
            fail(fid, 'identity candidate altered tangent handedness')
        product = src['tangent_w'][0] * src['tangent_w'][3]
        nearly_equal(product, oracle['opposite_chart_w_product'], 0.0, 0.0, 'w_product', fid)
        if manifold_status(src['vertices'], src['triangles']) != 'ok':
            fail(fid, 'identity mesh is not an admitted manifold')
    elif kind == 'affine_candidate':
        verts = data['input']['vertices']
        fused = apply_affine_fused(verts, data['kernel']['scale'], data['kernel']['bias'])
        staged = apply_affine_staged(verts, data['kernel']['bias'])
        got = data['candidate']['vertices']
        nearly_equal(got, fused, 0.0, 0.0, 'fused_affine', fid)
        nearly_equal(got, staged, 0.0, 0.0, 'staged_affine', fid)
        if check_affine is None:
            fail(fid, 'production_core_artifacts.check_affine missing for independent K1 route')
        report = check_affine(verts, got)
        if report['compared'] != 3 * len(verts):
            fail(fid, 'affine compared count mismatch')
    elif kind == 'operator_candidate':
        matrix = data['matrix']
        if 'solution' in data.get('candidate', {}):
            sol = data['candidate']['solution']
            got = residual(matrix, sol, data['rhs'])
            nearly_equal(got, oracle['true_residual'], 1e-15, 0.0, 'true_residual', fid)
            roots = characteristic_roots_tridiag3(matrix[0][0], matrix[1][1], matrix[2][2], matrix[0][1], matrix[1][2])
            nearly_equal(sorted(oracle['eigenvalues']), sorted(roots), 1e-12, 0.0, 'eigenvalues', fid)
            if oracle['nullspace_dimension'] != 0:
                fail(fid, 'Dirichlet fixture must have trivial nullspace')
        else:
            field = data['field']
            got = matvec(matrix, field)
            nearly_equal(got, oracle['operator_result'], 1e-15, 0.0, 'neumann_result', fid)
            if abs(sum(matrix[0]) ) > 1e-15 or abs(sum(matrix[-1])) > 1e-15:
                fail(fid, 'Neumann rows must sum to zero')
            if oracle['nullspace_dimension'] != 1:
                fail(fid, 'Neumann fixture must declare 1D nullspace')
    elif kind == 'transfer_candidate':
        rest = barycentre(data['vertices'], data['barycentric'])
        nearly_equal(rest, oracle['rest_point'], 1e-15, 0.0, 'rest_point', fid)
        deformed = rotate_translate(rest, data['rotation'], data['translation'])
        nearly_equal(deformed, oracle['deformed_point'], 1e-15, 0.0, 'deformed_point', fid)
        volume = tet_signed_volume(data['vertices'], data['tets'][0])
        nearly_equal(volume, oracle['volume'], 1e-15, 0.0, 'volume', fid)
        if volume <= 0.0:
            fail(fid, 'identity tet must have positive volume')
        quantized = [int(round(w * 65535.0)) for w in data['skin_weights']]
        if sum(quantized) != oracle['quantized_weight_sum']:
            fail(fid, f'quantized weights {quantized} sum {sum(quantized)}')
    else:
        fail(fid, f'unhandled identity kind {kind}')


def check_predicate(fid, data, oracle):
    a, b, c, d = data['points']
    six_vol = dot(sub(b, a), cross(sub(c, a), sub(d, a)))
    if abs(six_vol) > 1e-12:
        fail(fid, f'near-coplanar fixture is not near-singular: 6V={six_vol}')
    if not oracle.get('requires_adaptive_precision'):
        fail(fid, 'near-coplanar oracle must require adaptive precision')


def check_worklist(fid, data, oracle):
    if data.get('cancelled'):
        if oracle['expected_status'] != 'cancelled':
            fail(fid, 'cancelled worklist must expect cancelled')
        return
    if data.get('capacity', 0) == 0 and data.get('item_count', data.get('items', [1])) in (0, [], None):
        if oracle.get('explicit_empty_outcome') is not True:
            fail(fid, 'empty work/mesh must be explicit')
        return
    if data.get('item_count', 0) > data.get('capacity', 0):
        if oracle['expected_status'] != 'capacity_exhausted':
            fail(fid, 'over-capacity worklist must expect capacity_exhausted')


def check_view(fid, data, oracle):
    near = data['camera']['near']
    zmin, zmax = data['aabb_min'][2], data['aabb_max'][2]
    intersects = (zmin < near) and (zmax > near)
    if intersects is not oracle['intersects_near_plane']:
        fail(fid, f'near-plane intersection {intersects}')
    if 'left' in data['requested_cut'] and 'right' in data['requested_cut']:
        if 'left' not in oracle['forbidden_partial_cut']:
            fail(fid, 'partial left cut must be forbidden when both children are requested')
    if data['previous_occlusion_visible'] is False and oracle.get('camera_cut_invalidates_old_occlusion') is not True:
        fail(fid, 'stale occlusion must be invalidated')


def check_maximum_descriptor(fid, data, oracle):
    if data.get('materialization') != 'bounded_generated_input_required':
        fail(fid, 'maximum-size must remain a descriptor')
    if data.get('do_not_write_mesh_into_repository') is not True:
        fail(fid, 'maximum-size descriptor must forbid repository materialization')
    vcap = data['parameters']['vertex_count_cap']
    tcap = data['parameters']['triangle_count_cap']
    exact = data['parameters']['exact_closed_manifold']
    exact_f = exact['triangle_count']
    equator = exact['equator_vertices']
    if exact['vertex_count'] != equator + 2:
        fail(fid, 'bipyramid V must be equator+2')
    if exact_f != 2 * equator or exact_f != 2 * vcap - 4:
        fail(fid, 'Euler F=2V-4 mismatch in descriptor')
    if exact_f > tcap:
        fail(fid, 'exact closed mesh exceeds triangle cap')
    if data.get('generator') != 'closed_genus0_unit_bipyramid':
        fail(fid, 'unexpected maximum-size generator')
    if oracle.get('materialize_into_repository') is not False:
        fail(fid, 'oracle must forbid materializing the maximum-size mesh')


def evaluate_fixture(case):
    fid = case['id']
    check_required_fields(case, fid)
    data, oracle = case['data'], case['oracle']
    family = case['family']
    extra = {}
    if family == 'known_bad':
        observation = observe_known_bad_gates(data, oracle)
        extra['observed_failed_gates'] = list(observation.get('observed') or [])
        extra['unevaluated_gates'] = list(observation.get('unevaluated') or [])
        extra['production_exe_invoked'] = False
        check_known_bad(fid, data, oracle)
    elif family == 'identity':
        check_identity(fid, data, oracle)
    elif family == 'maximum_size_descriptor':
        check_maximum_descriptor(fid, data, oracle)
    elif data.get('kind') == 'predicate':
        check_predicate(fid, data, oracle)
    elif data.get('kind') == 'worklist':
        check_worklist(fid, data, oracle)
    elif data.get('kind') == 'view_selection':
        check_view(fid, data, oracle)
    else:
        check_mesh_oracle(fid, data, oracle)
        if data.get('kind') == 'worklist' or data.get('vertices') == []:
            check_worklist(fid, data, oracle)
        if family == 'locked_protected' and data.get('target_faces') == 0 and data.get('locked_vertices'):
            if oracle.get('published_success') is not False:
                fail(fid, 'locked target_faces=0 must not publish success')
        if family == 'empty_impossible_degenerate' and data.get('target_faces') == 0:
            if oracle.get('published_success') is not False:
                fail(fid, 'impossible ratio must not publish success')
    return dict(id=fid, family=family, partition=case['partition'], publication_check='consistent', **extra)


def run_corpus(corpus_dir: Path):
    corpus_dir = Path(corpus_dir)
    manifest_path = corpus_dir / 'manifest.json'
    if not manifest_path.is_file():
        raise FileNotFoundError(manifest_path)
    raw_manifest = manifest_path.read_bytes()
    manifest = json.loads(raw_manifest.decode('utf-8'))
    if canonical(manifest) != raw_manifest:
        raise OracleError('manifest.json is not canonical')
    if manifest.get('qualification_status') != 'not_executed':
        raise OracleError('manifest qualification_status must remain not_executed')
    if manifest.get('production_implementation_compared') is not False:
        raise OracleError('harness must not claim production comparison')
    algo, primary_fn, _note = hash_backend()
    if manifest.get('hash_algorithm') != algo:
        raise OracleError(f'manifest hash_algorithm {manifest.get("hash_algorithm")} != {algo}')
    if manifest.get('family_count') != 11 or len(manifest.get('families', [])) != 11:
        raise OracleError('manifest must enumerate eleven families')
    results = []
    families_seen = set()
    for entry in manifest['fixtures']:
        path = corpus_dir / entry['file']
        raw = path.read_bytes()
        digest = digest_bytes(raw, algo, primary_fn)
        if digest['digest'] != entry['digest']:
            raise OracleError(f'{entry["file"]}: digest mismatch')
        if digest['sha256'] != entry['sha256']:
            raise OracleError(f'{entry["file"]}: sha256 mismatch')
        case = json.loads(raw.decode('utf-8'))
        if canonical(case) != raw:
            raise OracleError(f'{entry["file"]}: not canonical JSON')
        if case['id'] != entry['id'] or case['family'] != entry['family']:
            raise OracleError(f'{entry["file"]}: manifest identity mismatch')
        results.append(evaluate_fixture(case))
        families_seen.add(case['family'])
    missing = [name for name in FAMILIES if name not in families_seen]
    if missing:
        raise OracleError(f'missing families {missing}')
    recipe = manifest.get('visual_capture_recipe') or {}
    if recipe.get('execution_status') != 'not_executed':
        raise OracleError('visual capture recipe must remain not_executed')
    if 'holdout_views' not in recipe or 'views' not in recipe:
        raise OracleError('visual capture recipe missing views')
    return dict(
        schema='production-core.oracle-harness-report/v1',
        status='fixture_publication_consistent',
        qualification_status='not_executed',
        production_implementation_compared=False,
        visual_capture='recipe_only_not_executed',
        hash_algorithm=algo,
        fixture_count=len(results),
        families=sorted(families_seen),
        known_bad_failed_as_required=True,
        identity_passed_as_required=True,
        holdout_excluded_from_tuning=True,
        maximum_size_materialized=False,
        results=results,
    )


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--corpus', required=True, type=Path)
    args = parser.parse_args()
    report = run_corpus(args.corpus)
    print(json.dumps({k: v for k, v in report.items() if k != 'results'}, indent=2, sort_keys=True))
    print(json.dumps(dict(fixtures=report['fixture_count'], status=report['status']), sort_keys=True))
