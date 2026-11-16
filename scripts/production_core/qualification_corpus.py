"""Deterministic analytic/adversarial corpus for production-core qualification.

Fixtures are inputs and independently specified expectations, never passing
test results. Oracles are closed-form or exact-integer statements; they are
not copied from QualiaDB, Etalon, or Hypermedia helpers.

The maximum-size case is a descriptor. Do not materialize it into the corpus.
"""
from __future__ import annotations

from pathlib import Path
import argparse
import hashlib
import json
import math
import sys

SCHEMA = 'production-core.qualification-fixture/v1'
MANIFEST_SCHEMA = 'production-core.qualification-corpus/v1'
MAX_BYTES = 4 * 1024 * 1024
PROVENANCE = (
    'Original analytic construction; no imported dataset; oracle values are '
    'closed-form or exact-integer and are not derived from QualiaDB/Etalon/'
    'Hypermedia implementation helpers'
)
LICENSE = 'Apache-2.0'

FAMILIES = (
    'analytic_surfaces',
    'locked_protected',
    'mixed_provenance',
    'uv_seam_tangents',
    'thin_near_threshold',
    'large_translated',
    'empty_impossible_degenerate',
    'known_bad',
    'identity',
    'holdout',
    'maximum_size_descriptor',
)

REQ37_EXPOSURE = {
    'analytic_surfaces': 'Analytic plane, sphere, cylinder, and scale extremes',
    'locked_protected': 'Open boundaries, sharp CAD features, protected pivots',
    'mixed_provenance': 'Mixed provenance and epistemic labels',
    'uv_seam_tangents': 'UV seams, tiled/checker textures, normal maps, tangent handedness',
    'thin_near_threshold': 'Thin shells and near-threshold / nearly coplanar features',
    'large_translated': 'Small features at large world coordinates',
    'empty_impossible_degenerate': 'Empty, impossible-ratio, degenerate, non-manifold, disconnected, ragged/exhausted work',
    'known_bad': 'Adversarial candidate that must fail relevant gates',
    'identity': 'B0 identity/no-reduction candidate that must pass relevant tests; spectral and rigid-motion oracles',
    'holdout': 'Held-out geometry, views, and poses excluded from tuning',
    'maximum_size_descriptor': 'Empty/ragged/maximum-size/exhausted worklists; descriptor only',
}

G1_SURFACE = 1e-3
G1_BRACKET = 1e-5
G1_PROTECTED = 1e-5
K1_ATOL = 1e-6
K1_RTOL = 1e-5
O1_EIGEN_REL = 0.01
PROFILE_VERTEX_CAP = 50000
PROFILE_TRIANGLE_CAP = 100000
COLD_BYTE_LIMIT = 256 * 1024 * 1024


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'),
                       allow_nan=False) + '\n').encode('utf-8')


def hash_backend():
    vendor = Path(__file__).resolve().parent / 'vendor'
    if vendor.is_dir():
        vendor_s = str(vendor)
        if vendor_s not in sys.path:
            sys.path.insert(0, vendor_s)
        try:
            from blake3 import blake3
        except ImportError:
            blake3 = None
        else:
            return ('blake3', lambda data: blake3(data).hexdigest(),
                    'blake3 used from scripts/production_core/vendor; sha256 also recorded')
    return ('sha256', lambda data: hashlib.sha256(data).hexdigest(),
            'scripts/production_core/vendor absent or blake3 import failed; sha256 fallback; blake3 optional')


def digest_bytes(data, primary_algo, primary_fn):
    item = {
        'hash_algorithm': primary_algo,
        'digest': primary_fn(data),
        'sha256': hashlib.sha256(data).hexdigest(),
    }
    if primary_algo != 'sha256':
        item[primary_algo] = item['digest']
    return item


def g1_tolerances(extent_diagonal, extra=None):
    d = float(extent_diagonal)
    item = dict(
        profile='G1',
        extent_diagonal=d,
        nonzero_reference_scale=d,
        surface_deviation_threshold=G1_SURFACE * d,
        bracket_width_limit=G1_BRACKET * d,
        protected_displacement_limit=G1_PROTECTED * d,
        forbidden_semantic_merges=0,
        near_zero_policy=(
            'Distinguish absolute and relative error. A residual is near-zero '
            'only if |r| <= 1e-15 * nonzero_reference_scale or r is an exact integer zero.'
        ),
        absolute_tolerance=G1_SURFACE * d,
        relative_tolerance=G1_SURFACE,
    )
    if extra:
        item.update(extra)
    return item


def k1_tolerances(extra=None):
    item = dict(
        profile='K1',
        atol=K1_ATOL,
        rtol=K1_RTOL,
        near_zero_policy='|r| <= atol + rtol*|expected|; nonfinite values are domain errors',
        integer_topology_exact=True,
    )
    if extra:
        item.update(extra)
    return item


def o1_tolerances(nullspace_dimension, extra=None):
    item = dict(
        profile='O1',
        relative_eigenvalue_limit=O1_EIGEN_REL,
        principal_angle_limit_radians=5.0 * math.pi / 180.0,
        nullspace_dimension=nullspace_dimension,
        first_nonzero_modes_claimed=None,
        note='Only the modes that exist on this fixture are claimed; no twenty-mode claim.',
    )
    if extra:
        item.update(extra)
    return item


def fixture(name, family, data, oracle, partition='test', seed=0, exclusions=(),
            spec_exposure=None, units='dimensionless unless declared'):
    if family not in FAMILIES:
        raise ValueError(f'unknown family {family}')
    if partition not in ('tune', 'test', 'holdout'):
        raise ValueError(f'unknown partition {partition}')
    if family == 'holdout' and partition != 'holdout':
        raise ValueError('holdout family fixtures must use the holdout partition')
    body = dict(
        schema=SCHEMA,
        id=name,
        family=family,
        family_index=FAMILIES.index(family) + 1,
        partition=partition,
        seed=int(seed),
        provenance=PROVENANCE,
        license=LICENSE,
        units=units,
        spec_exposure=spec_exposure or REQ37_EXPOSURE[family],
        data=data,
        oracle=oracle,
        exclusions=list(exclusions),
        execution_status='not_executed',
    )
    return body


def plane(scale=1.0, origin=(0.0, 0.0, 0.0), axis='z'):
    s = float(scale)
    ox, oy, oz = (float(origin[0]), float(origin[1]), float(origin[2]))
    corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
    if axis == 'z':
        vertices = [[ox + s * x, oy + s * y, oz] for x, y in corners]
        normal = [0.0, 0.0, 1.0]
        surface = f'z={oz}'
        triangles = [[0, 1, 2], [0, 2, 3]]
    elif axis == 'y':
        vertices = [[ox + s * x, oy, oz + s * y] for x, y in corners]
        normal = [0.0, 1.0, 0.0]
        surface = f'y={oy}'
        triangles = [[0, 2, 1], [0, 3, 2]]
    else:
        raise ValueError('plane axis must be y or z')
    return dict(
        vertices=vertices,
        triangles=triangles,
        exact_normal=normal,
        surface=surface,
        scale=s,
        origin=[ox, oy, oz],
        axis=axis,
    )


def two_subpart_seam():
    vertices = [
        [0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [0.5, 1.0, 0.0],
        [0.5, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0],
    ]
    return dict(
        vertices=vertices,
        triangles=[[0, 1, 2], [3, 4, 5]],
        uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        domains=[101, 101, 101, 202, 202, 202],
        subpart_tags=[101, 101, 101, 202, 202, 202],
        provenance_labels=[101, 101, 101, 202, 202, 202],
        epistemic=['measured', 'measured', 'measured', 'inferred', 'inferred', 'inferred'],
        tangent_w=[1.0, 1.0, 1.0, -1.0, -1.0, -1.0],
        locked_vertices=[],
        protected_edges=[],
        coincident_pairs=[[1, 3], [2, 5]],
        checker=dict(size=8, rule='(floor(8*u)+floor(8*v)) modulo 2'),
        normal_map_tangent=[0.0, 0.0, 1.0],
    )


def octahedron_sphere():
    return dict(
        vertices=[[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
                  [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]],
        triangles=[[0, 2, 4], [2, 1, 4], [1, 3, 4], [3, 0, 4],
                   [2, 0, 5], [1, 2, 5], [3, 1, 5], [0, 3, 5]],
    )


def open_cylinder(segments=16, radius=1.0, height=1.0, origin=(0.0, 0.0, 0.0)):
    n = int(segments)
    if n < 3:
        raise ValueError('cylinder needs at least 3 segments')
    ox, oy, oz = (float(origin[0]), float(origin[1]), float(origin[2]))
    r = float(radius)
    h = float(height)
    vertices = []
    for z in (-0.5 * h, 0.5 * h):
        for i in range(n):
            ang = 2.0 * math.pi * i / n
            vertices.append([ox + r * math.cos(ang), oy + r * math.sin(ang), oz + z])
    triangles = []
    for i in range(n):
        j = (i + 1) % n
        triangles.extend([[i, j, n + j], [i, n + j, n + i]])
    return dict(vertices=vertices, triangles=triangles, segments=n, radius=r,
                height=h, origin=[ox, oy, oz])


def normalized_cube_sphere():
    corners = []
    s = 1.0 / math.sqrt(3.0)
    for x in (-1.0, 1.0):
        for y in (-1.0, 1.0):
            for z in (-1.0, 1.0):
                corners.append([s * x, s * y, s * z])
    # 8 cube corners, triangulated faces (12 triangles), closed genus 0.
    faces = [
        [0, 1, 3], [0, 3, 2], [4, 6, 7], [4, 7, 5],
        [0, 4, 5], [0, 5, 1], [2, 3, 7], [2, 7, 6],
        [0, 2, 6], [0, 6, 4], [1, 5, 7], [1, 7, 3],
    ]
    return dict(vertices=corners, triangles=faces)


def standard_tet(sign_z=1.0):
    return dict(
        vertices=[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
                  [0.0, 0.0, float(sign_z)]],
        tets=[[0, 1, 2, 3]],
    )


def visual_capture_recipe():
    return dict(
        schema='production-core.visual-capture-recipe/v1',
        execution_status='not_executed',
        subject_fixture_ids=['identity-two-subpart-seam', 'uv-duplicated-seam'],
        views=[
            dict(name='front', camera=dict(position=[0.5, 0.5, 3.0], look=[0.5, 0.5, 0.0], up=[0.0, 1.0, 0.0])),
            dict(name='side', camera=dict(position=[3.0, 0.5, 0.5], look=[0.5, 0.5, 0.0], up=[0.0, 1.0, 0.0])),
            dict(name='three-quarter', camera=dict(position=[2.2, 1.6, 2.2], look=[0.5, 0.5, 0.0], up=[0.0, 1.0, 0.0])),
        ],
        holdout_views=[
            dict(name='holdout-low', camera=dict(position=[0.5, -3.0, 0.6], look=[0.5, 0.5, 0.0], up=[0.0, 0.0, 1.0])),
            dict(name='holdout-near-oblique', camera=dict(position=[0.15, 0.2, 0.12], look=[0.5, 0.5, 0.0], up=[0.0, 1.0, 0.0])),
        ],
        overlays=['wireframe', 'normals', 'error'],
        note=(
            'Recipe only. Screenshots are not generated by fixture publication. '
            'Holdout views must not be used to choose thresholds.'
        ),
    )


def maximum_size_descriptor(seed):
    # Closed genus-0 Euler: F = 2V - 4. V=50000 => F=99996, inside the 100000 cap.
    equator = PROFILE_VERTEX_CAP - 2
    return dict(
        kind='execution_time_bounded_generator',
        materialization='bounded_generated_input_required',
        generator='closed_genus0_unit_bipyramid',
        start_mesh='unit_bipyramid_over_regular_ngon',
        seed=int(seed),
        parameters=dict(
            vertex_count_cap=PROFILE_VERTEX_CAP,
            triangle_count_cap=PROFILE_TRIANGLE_CAP,
            exact_closed_manifold=dict(
                vertex_count=PROFILE_VERTEX_CAP,
                triangle_count=2 * PROFILE_VERTEX_CAP - 4,
                equator_vertices=equator,
                euler_characteristic=2,
            ),
            work_bound_scans=PROFILE_VERTEX_CAP + PROFILE_TRIANGLE_CAP,
        ),
        cold_byte_limit=COLD_BYTE_LIMIT,
        admitted_profile=dict(vertices=PROFILE_VERTEX_CAP, triangles=PROFILE_TRIANGLE_CAP),
        do_not_write_mesh_into_repository=True,
        quality_note='Capacity/work fixture; equatorial triangles are long and thin and are not an element-quality target.',
    )


def bounded_maximum_size_mesh(descriptor):
    """Execution-time generator. Never called by write_corpus or corpus publication."""
    if descriptor.get('materialization') != 'bounded_generated_input_required':
        raise ValueError('refusing to materialize a non-descriptor maximum-size case')
    if descriptor.get('generator') != 'closed_genus0_unit_bipyramid':
        raise ValueError(f'unsupported maximum-size generator {descriptor.get("generator")}')
    exact = descriptor['parameters']['exact_closed_manifold']
    north, south = [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]
    n = int(exact['equator_vertices'])
    work_bound = int(descriptor['parameters']['work_bound_scans'])
    if 2 + n + 2 * n > work_bound:
        raise ValueError('work bound too small to emit the closed bipyramid')
    vertices = [north, south]
    for i in range(n):
        ang = 2.0 * math.pi * i / n
        vertices.append([math.cos(ang), math.sin(ang), 0.0])
    triangles = []
    for i in range(n):
        j = 2 + i
        k = 2 + ((i + 1) % n)
        triangles.append([0, j, k])
        triangles.append([1, k, j])
    if len(vertices) != exact['vertex_count'] or len(triangles) != exact['triangle_count']:
        raise ValueError('bipyramid generator missed the Euler target')
    return dict(
        vertices=vertices,
        triangles=triangles,
        generator=descriptor['generator'],
        work_used=len(vertices) + len(triangles),
        truncated=False,
    )


def _analytic_family(add):
    for scale, split, seed in [(1.0, 'tune', 101), (1e-6, 'test', 102), (1e6, 'test', 103)]:
        mesh = plane(scale)
        d = math.sqrt(2.0) * scale
        add(f'analytic-plane-scale-{scale:g}', 'analytic_surfaces',
            dict(kind='triangle_mesh', **{k: mesh[k] for k in ('vertices', 'triangles', 'scale', 'origin', 'axis')}),
            dict(
                surface=mesh['surface'],
                reference_area=scale * scale,
                exact_normal=mesh['exact_normal'],
                expected_components=1,
                boundary_loops=1,
                vertex_count=4,
                triangle_count=2,
                derivation='closed_form_unit_square_area_scale_squared_and_axis_aligned_normal',
                not_derived_from='QualiaDB mesh_quality or Hypermedia simplify',
                tolerances=g1_tolerances(d),
                scales=dict(construction_scale=scale, extent_diagonal=d, nonzero_reference_scale=d),
                conditioning=dict(
                    world_origin_norm=0.0,
                    feature_scale=scale,
                    f32_ulp_risk='low_at_origin',
                    recommended_frame='construction_frame',
                ),
                nullspace=dict(dimension=0),
            ),
            partition=split, seed=seed,
            exclusions=['Axis-aligned unit square only; not a general CAD plane tessellation.'])
    sphere = octahedron_sphere()
    add('analytic-sphere-octahedron', 'analytic_surfaces',
        dict(kind='triangle_mesh', **sphere),
        dict(
            vertex_surface='x*x+y*y+z*z=1',
            radial_vertex_error=0.0,
            mesh_is_analytic_surface=False,
            expected_components=1,
            boundary_loops=0,
            euler_characteristic=2,
            vertex_count=6,
            triangle_count=8,
            derivation='octahedron vertices are the unit-axis points; ||v||^2=1 exactly',
            not_derived_from='QualiaDB authoring sphere tessellation',
            tolerances=g1_tolerances(2.0),
            scales=dict(radius=1.0, extent_diagonal=2.0, nonzero_reference_scale=2.0),
            conditioning=dict(world_origin_norm=0.0, feature_scale=1.0, f32_ulp_risk='low'),
            nullspace=dict(dimension=0),
        ),
        partition='tune', seed=104,
        exclusions=['Octahedron face interiors are not on the unit sphere.'])
    cyl = open_cylinder()
    add('analytic-open-cylinder', 'analytic_surfaces',
        dict(kind='triangle_mesh', **cyl),
        dict(
            vertex_surface='x*x+y*y=1',
            radial_vertex_error_bound=2.0 ** -52,
            boundary_loops=2,
            expected_components=1,
            vertex_count=32,
            triangle_count=32,
            derivation='unit circle samples at 2*pi*i/n with n=16; height 1; open side wall',
            not_derived_from='QualiaDB authoring cylinder',
            tolerances=g1_tolerances(math.sqrt(4.0 + 1.0)),
            scales=dict(radius=1.0, height=1.0, extent_diagonal=math.sqrt(5.0), nonzero_reference_scale=math.sqrt(5.0)),
            conditioning=dict(world_origin_norm=0.0, feature_scale=1.0, f32_ulp_risk='low'),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=105,
        exclusions=['Polygonal chord interiors differ from the analytic cylinder.'])


def _locked_family(add):
    mesh = plane()
    add('locked-open-boundary-pivot', 'locked_protected',
        dict(kind='triangle_mesh', vertices=mesh['vertices'], triangles=mesh['triangles'],
             locked_vertices=[0, 1, 2, 3], protected_edges=[[0, 1], [1, 2], [2, 3], [3, 0]],
             target_faces=0, target_ratio=0.0),
        dict(
            expected_status='unsatisfied_constraint',
            reason_code='UnsatisfiedConstraint',
            locked_displacement=0.0,
            target_must_not_override_locks=True,
            published_success=False,
            derivation='all vertices locked; any collapse moves a lock; target_faces=0 is impossible',
            not_derived_from='QualiaDB semantic QEM penalty weights',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(constraint_rank='full_vertex_lock', nullspace_of_allowed_collapses=0),
            nullspace=dict(dimension=0, allowed_collapses=0),
        ),
        partition='test', seed=201,
        exclusions=['Constraint-limited termination is not a successful 0-face simplification.'])
    add('locked-sharp-crease', 'locked_protected',
        dict(kind='triangle_mesh',
             vertices=[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
             triangles=[[0, 1, 2], [1, 0, 3]],
             protected_edges=[[0, 1]],
             locked_vertices=[]),
        dict(
            crease_angle_radians=math.pi / 2.0,
            protected_edge_must_survive=True,
            expected_status='constraint_limited_if_target_removes_crease',
            derivation='two unit triangles sharing edge (0,1); dihedral angle pi/2 by orthonormal planes z=0 and y=0',
            not_derived_from='QualiaDB feature-angle heuristics',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(feature_angle_radians=math.pi / 2.0, well_separated_from_crease_threshold=True),
            nullspace=dict(dimension=0),
        ),
        partition='tune', seed=202)
    add('locked-impossible-ratio', 'locked_protected',
        dict(kind='triangle_mesh', vertices=mesh['vertices'], triangles=mesh['triangles'],
             locked_vertices=[0, 1, 2, 3], target_faces=1, target_ratio=0.5,
             target_ratio_ladder=[0.75, 0.5, 0.25, 0.1]),
        dict(
            expected_status='unsatisfied_constraint',
            reason_code='UnsatisfiedConstraint',
            reachable_face_count=2,
            reachable_ratio=1.0,
            failure_to_reach_ratio_is_not_success=True,
            published_success=False,
            derivation='four locked corners on two triangles; no legal collapse exists',
            not_derived_from='weighted QEM cost as a substitute for hard locks',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0), target_ratio=0.5),
            conditioning=dict(constraint_limited=True),
            nullspace=dict(dimension=0, allowed_collapses=0),
        ),
        partition='test', seed=203,
        exclusions=['Failing to reach 50%/25%/10% while honouring locks is not equivalent to meeting that ratio.'])


def _mixed_family(add):
    seam = two_subpart_seam()
    add('mixed-provenance-domains', 'mixed_provenance',
        dict(kind='triangle_mesh',
             vertices=seam['vertices'], triangles=seam['triangles'],
             domains=seam['domains'],
             provenance_labels=seam['provenance_labels'],
             epistemic=seam['epistemic'],
             subpart_tags=seam['subpart_tags'],
             materials=seam['domains'],
             coincident_pairs=seam['coincident_pairs']),
        dict(
            forbidden_cross_domain_support=0,
            forbidden_merges=0,
            triangle_domain_must_be_uniform=True,
            coincident_pairs_are_not_welded=True,
            derivation=(
                'Two triangles meet geometrically at x=0.5 but use duplicated corners. '
                'Face 0 is entirely 101/measured; face 1 is entirely 202/inferred. '
                'Welding coincident pairs would mix provenance, epistemic, subpart and material labels.'
            ),
            not_derived_from='QualiaDB semantic penalty tables',
            tolerances=g1_tolerances(math.sqrt(2.0), extra=dict(profile='G1+T1')),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(domain_cut_edges=1, coincident_pairs=2),
            nullspace=dict(dimension=0),
        ),
        partition='tune', seed=301)


def _uv_family(add):
    seam = two_subpart_seam()
    add('uv-duplicated-seam', 'uv_seam_tangents',
        dict(kind='triangle_mesh', **seam),
        dict(
            no_cross_domain_support=True,
            preserve_corner_chart=True,
            preserve_tangent_handedness=True,
            opposite_chart_w_product=-1.0,
            coincident_pairs_are_not_welded=True,
            checker_must_not_bleed_across_seam=True,
            derivation=(
                'Two charts meet at x=0.5 with duplicated corners. Chart 101 has tangent_w=+1, '
                'chart 202 has tangent_w=-1. Bitangent = w * cross(n, t); averaging across the '
                'seam would flip a chart.'
            ),
            not_derived_from='Hypermedia compute_mikktspace output or QualiaDB uv_atlas',
            tolerances=g1_tolerances(math.sqrt(1.0 + 1.0), extra=dict(profile='G1+T1')),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(seam_vertex_multiplicity=2, chart_count=2),
            nullspace=dict(dimension=0),
        ),
        partition='tune', seed=401,
        exclusions=['Geometric QEM is not promised to preserve UV seams; this fixture tests the hard seam contract.'])


def _thin_family(add):
    base = plane()
    shell_vertices = list(base['vertices']) + [[x, y, 1e-5] for x, y, z in base['vertices']]
    shell_triangles = list(base['triangles']) + [[4, 6, 5], [4, 7, 6]]
    add('thin-double-sheet', 'thin_near_threshold',
        dict(kind='triangle_mesh', vertices=shell_vertices, triangles=shell_triangles, sheet_gap=1e-5),
        dict(
            expected_components=2,
            sheet_gap=1e-5,
            forbid_sheet_merge=True,
            gap_below_g1_threshold=True,
            derivation='two parallel unit squares separated by 1e-5 along z; G1 1e-3 D ~ 1.4e-3 so the gap is a thin feature, not geometric noise',
            not_derived_from='sampled Hausdorff estimates',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0 + 1e-10), nonzero_reference_scale=math.sqrt(2.0), feature_thickness=1e-5),
            conditioning=dict(thickness_over_diagonal=1e-5 / math.sqrt(2.0), nearly_duplicate_vertices=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=501)
    h = 2.0e-3 * math.sqrt(2.0)
    bump = plane()
    bump_vertices = [list(v) for v in bump['vertices']]
    bump_vertices.append([0.5, 0.5, h])
    bump_triangles = [[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]]
    add('thin-ridge-near-g1', 'thin_near_threshold',
        dict(kind='triangle_mesh', vertices=bump_vertices, triangles=bump_triangles, ridge_height=h),
        dict(
            ridge_height=h,
            g1_surface_threshold=G1_SURFACE * math.sqrt(2.0),
            dropping_ridge_exceeds_g1=True,
            derivation='pyramid apex height 2e-3 * D with D=sqrt(2); G1 surface threshold is 1e-3 D',
            not_derived_from='QEM cost as a surface certificate',
            tolerances=g1_tolerances(math.sqrt(2.0 + h * h)),
            scales=dict(extent_diagonal=math.sqrt(2.0 + h * h), nonzero_reference_scale=math.sqrt(2.0), feature_height=h),
            conditioning=dict(feature_to_threshold_ratio=2.0),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=502)
    add('thin-near-coplanar-predicate', 'thin_near_threshold',
        dict(kind='predicate',
             points=[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.3, 0.3, 1e-18]],
             predicate='orient3d'),
        dict(
            exact_orient_sign='uncertain_without_adaptive_filter',
            requires_adaptive_precision=True,
            must_not_commit_topology_on_approximate_sign=True,
            derivation='scaled orient3d ~ 1e-18; f32 cannot represent the height; filtered exact predicate required',
            not_derived_from='WGSL f32 cross/dot as a topology oracle',
            tolerances=k1_tolerances(extra=dict(profile='predicate-filter', atol=0.0, note='topology signs are exact; no K1 atol on the predicate outcome')),
            scales=dict(base_edge=1.0, height=1e-18, nonzero_reference_scale=1.0),
            conditioning=dict(coordinate_dynamic_range=1e18, nearly_singular=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=503,
        exclusions=['This is a predicate admission fixture, not a mesh simplification input.'])


def _translated_family(add):
    origin = (1e9, 1e9, 1e9)
    mesh = plane(scale=1e-3, origin=origin)
    add('large-origin-small-feature', 'large_translated',
        dict(kind='triangle_mesh', vertices=mesh['vertices'], triangles=mesh['triangles'],
             origin=list(origin), construction_scale=1e-3),
        dict(
            local_frame_required_for_f32=True,
            direct_f32_world_coordinate_collapse_must_not_be_accepted=True,
            derivation='feature scale 1e-3 at world 1e9; f32 ulp(1e9)=128, which exceeds the feature and G1 1e-3 D',
            not_derived_from='implementation world-space f32 QEM',
            tolerances=g1_tolerances(math.sqrt(2.0) * 1e-3, extra=dict(
                note='G1 applies in a local frame after subtracting origin; world f32 is outside the admitted arithmetic domain',
            )),
            scales=dict(
                origin_norm=math.sqrt(3.0) * 1e9,
                construction_scale=1e-3,
                extent_diagonal=math.sqrt(2.0) * 1e-3,
                nonzero_reference_scale=math.sqrt(2.0) * 1e-3,
                f32_ulp_at_origin=128.0,
            ),
            conditioning=dict(
                world_origin_norm=math.sqrt(3.0) * 1e9,
                feature_scale=1e-3,
                dynamic_range=1e9 / 1e-3,
                f32_ulp_risk='catastrophic_in_world_frame',
                recommended_frame='local_origin_1e9',
            ),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=601)


def _empty_family(add):
    add('empty-mesh', 'empty_impossible_degenerate',
        dict(kind='triangle_mesh', vertices=[], triangles=[], items=[], capacity=0),
        dict(
            expected_status='invalid_input',
            reason_code='InvalidInput',
            explicit_empty_outcome=True,
            no_dispatch=True,
            published_success=False,
            derivation='empty vertex and triangle arrays; admission must fail closed',
            not_derived_from='producer placeholder meshes',
            tolerances=k1_tolerances(),
            scales=dict(extent_diagonal=0.0, nonzero_reference_scale=None, near_zero_policy='empty has no reference scale'),
            conditioning=dict(empty=True),
            nullspace=dict(dimension=None, note='no operator is assembled on an empty mesh'),
        ),
        partition='test', seed=701)
    mesh = plane()
    add('impossible-ratio', 'empty_impossible_degenerate',
        dict(kind='triangle_mesh', vertices=mesh['vertices'], triangles=mesh['triangles'],
             target_faces=0, target_ratio=0.0, locked_vertices=[]),
        dict(
            expected_status='unsatisfied_constraint',
            reason_code='UnsatisfiedConstraint',
            published_success=False,
            derivation='two-face disk cannot reach target_faces=0 without deleting the asset; empty output is not a legal ratio',
            not_derived_from='success-shaped zero-face dumps',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0), target_ratio=0.0),
            conditioning=dict(impossible_ratio=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=702)
    verts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]
    for name, triangles, expectation, seed, note in [
        ('duplicate-face', [[0, 1, 2], [0, 1, 2]], 'reject_duplicate', 703, 'identical oriented face repeated'),
        ('degenerate-face', [[0, 0, 1]], 'reject_degenerate', 704, 'repeated vertex indices, zero area'),
        ('nonmanifold-edge', [[0, 1, 2], [1, 0, 3], [0, 1, 4]], 'reject_nonmanifold', 705, 'edge (0,1) used three times'),
        ('inconsistent-orientation', [[0, 1, 2], [0, 1, 3]], 'reject_orientation', 706, 'shared edge traversed twice in the same direction'),
    ]:
        add(name, 'empty_impossible_degenerate',
            dict(kind='triangle_mesh', vertices=verts, triangles=triangles),
            dict(
                strict_manifold_status=expectation,
                expected_status='invalid_input',
                reason_code='InvalidInput',
                published_success=False,
                derivation=note,
                not_derived_from='implicit repair paths',
                tolerances=k1_tolerances(),
                scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
                conditioning=dict(outside_admitted_topology=True),
                nullspace=dict(dimension=0),
            ),
            partition='test', seed=seed,
            exclusions=['Repair is never implicit; these inputs must be rejected unless an explicitly qualified repair path is selected.'])
    disconnected = plane()
    other = plane(origin=(3.0, 0.0, 0.0))
    disconnected_vertices = list(disconnected['vertices']) + other['vertices']
    disconnected_triangles = list(disconnected['triangles']) + [[4, 5, 6], [4, 6, 7]]
    add('disconnected-disks', 'empty_impossible_degenerate',
        dict(kind='triangle_mesh', vertices=disconnected_vertices, triangles=disconnected_triangles),
        dict(
            expected_components=2,
            boundary_loops=2,
            admitted_if_disconnected_manifold_mode=True,
            derivation='two disjoint unit squares; union-find on triangle vertex ids yields two components',
            not_derived_from='single-component simplify assumptions',
            tolerances=g1_tolerances(math.sqrt((3.0 + 1.0) ** 2 + 1.0)),
            scales=dict(extent_diagonal=math.sqrt(16.0 + 1.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(disconnected=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=707)
    add('worklist-exhausted', 'empty_impossible_degenerate',
        dict(kind='worklist', item_count=65, capacity=64, workgroup_size=64, cancelled=False),
        dict(
            expected_status='capacity_exhausted',
            reason_code='CapacityExhausted',
            published_partial_success=False,
            no_out_of_bounds=True,
            derivation='65 items into capacity 64; remainder must not be treated as success',
            not_derived_from='GPU tail loops that skip barriers',
            tolerances=k1_tolerances(extra=dict(profile='R1')),
            scales=dict(capacity=64, item_count=65),
            conditioning=dict(ragged_tail=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=708)
    add('worklist-cancelled', 'empty_impossible_degenerate',
        dict(kind='worklist', item_count=65, capacity=65, cancelled=True),
        dict(
            expected_status='cancelled',
            reason_code='Cancelled',
            published_partial_success=False,
            derivation='cooperative cancellation before commit; cancelled work is not a qualified partial mesh',
            not_derived_from='best-effort truncated output',
            tolerances=k1_tolerances(extra=dict(profile='R1')),
            scales=dict(capacity=65, item_count=65),
            conditioning=dict(cancelled=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=709)


def _known_bad_family(add):
    src = two_subpart_seam()
    add('known-bad-cross-domain-weld', 'known_bad',
        dict(
            kind='representation_candidate',
            input=src,
            candidate=dict(
                claimed_status='accepted',
                claimed_forbidden_merges=0,
                claimed_target_ratio=0.5,
                vertices=[[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [0.5, 1.0, 0.0], [1.0, 0.0, 0.0]],
                triangles=[[0, 1, 2], [1, 3, 2]],
                domains=[101, 101, 101, 202],
                uvs=[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [1.0, 0.0]],
                tangent_w=[1.0, 1.0, 1.0, 1.0],
                collapsed_vertex_pairs=[[1, 3], [2, 5]],
            ),
        ),
        dict(
            must_fail=True,
            pass_is_corpus_defect=True,
            failing_tests=[
                'forbidden_semantic_merges',
                'no_cross_domain_support',
                'preserve_corner_chart',
                'preserve_tangent_handedness',
                'coincident_pairs_are_not_welded',
            ],
            mixed_face_domains=True,
            welded_opposite_tangent_w=True,
            derivation='candidate welds coincident seam corners from domains 101 and 202 and copies w=+1 onto the former w=-1 chart',
            not_derived_from='implementation simplify output',
            tolerances=g1_tolerances(math.sqrt(2.0), extra=dict(profile='G1+T1')),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(adversarial_weld=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=801)
    identity_src = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    poisoned = [[2.0 * x + 0.25 for x in p] for p in identity_src]
    poisoned[0][0] += 1.0
    add('known-bad-poisoned-affine', 'known_bad',
        dict(
            kind='affine_candidate',
            kernel=dict(scale=2.0, bias=0.25, profile='K1'),
            input=dict(vertices=identity_src),
            candidate=dict(claimed_status='accepted', vertices=poisoned),
        ),
        dict(
            must_fail=True,
            pass_is_corpus_defect=True,
            failing_tests=['independent_affine_oracle', 'k1_abs_rel_tolerance'],
            fused_closed_form='2*x+0.25',
            derivation='first output component is fused 2*x+0.25 then deliberately incremented by 1',
            not_derived_from='Etalon CPU f32 mul_add oracle',
            tolerances=k1_tolerances(),
            scales=dict(nonzero_reference_scale=2.25),
            conditioning=dict(well_conditioned=True, error_is_gross=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=802)
    inv = standard_tet(sign_z=-1.0)
    add('known-bad-tetra-inversion', 'known_bad',
        dict(kind='volume_candidate', input=inv, candidate=dict(claimed_status='accepted', **inv)),
        dict(
            must_fail=True,
            pass_is_corpus_defect=True,
            failing_tests=['accept_when_inversion_forbidden'],
            signed_volume=-1.0 / 6.0,
            signed_volume_numerator=-1,
            signed_volume_denominator=6,
            accept_when_inversion_forbidden=False,
            derivation='standard tet with apex z=-1; scalar triple product is -1, volume -1/6',
            not_derived_from='QualiaDB tet quality helpers',
            tolerances=dict(profile='T1+P1', volume_exact_rational=True, inversions_accepted=0),
            scales=dict(nonzero_reference_scale=1.0 / 6.0),
            conditioning=dict(inverted=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=803)
    add('known-bad-infeasible-contact', 'known_bad',
        dict(
            kind='contact_candidate',
            stiffness=1e6,
            displacement=1e-6,
            sphere_center=[0.0, 0.75, 0.0],
            sphere_radius=1.0,
            plane_normal=[0.0, 1.0, 0.0],
            plane_offset=0.0,
            candidate=dict(claimed_status='accepted', claimed_feasible=True),
        ),
        dict(
            must_fail=True,
            pass_is_corpus_defect=True,
            failing_tests=['contact_feasibility', 'authoritative_full_model_required'],
            spring_force=1.0,
            spring_energy=5e-7,
            penetration=0.25,
            feasible_contact=False,
            authoritative_full_model=True,
            derivation=(
                'linear spring k*x with k=1e6 and x=1e-6 gives force 1 and energy 0.5*k*x^2=5e-7; '
                'sphere of radius 1 centred at y=0.75 against plane y=0 has penetration 0.25'
            ),
            not_derived_from='Etalon XPBD kernels as a general contact solver',
            tolerances=dict(profile='P1', force_exact=1.0, energy_exact=5e-7),
            scales=dict(nonzero_reference_scale=1.0, stiffness=1e6, displacement=1e-6),
            conditioning=dict(high_stiffness=True, timestep_sensitive=True),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=804,
        exclusions=['Analytic admission fixture, not a general contact simulation. Core excludes a new general contact solver.'])


def _identity_family(add):
    src = two_subpart_seam()
    add('identity-two-subpart-seam', 'identity',
        dict(
            kind='representation_candidate',
            baseline='B0_identity_no_reduction',
            input=src,
            candidate=dict(claimed_status='accepted', **src),
        ),
        dict(
            must_pass=True,
            fail_is_corpus_defect=True,
            passing_tests=[
                'identity_geometry',
                'zero_forbidden_merges',
                'locked_displacement_zero',
                'uv_seam_preserved',
                'preserve_tangent_handedness',
                'opposite_chart_w_product',
            ],
            locked_displacement=0.0,
            forbidden_merges=0,
            opposite_chart_w_product=-1.0,
            derivation='candidate is a byte-for-byte copy of the authored two-subpart seam mesh; B0 no-reduction reference',
            not_derived_from='implementation simplify identity path',
            tolerances=g1_tolerances(math.sqrt(2.0), extra=dict(profile='G1+T1')),
            scales=dict(extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0), target_ratio=1.0),
            conditioning=dict(well_conditioned=True, reduction=0),
            nullspace=dict(dimension=0),
        ),
        partition='tune', seed=901)
    src_v = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    fused = [[2.0 * x + 0.25 for x in p] for p in src_v]
    add('identity-affine-k1', 'identity',
        dict(
            kind='affine_candidate',
            kernel=dict(scale=2.0, bias=0.25, profile='K1'),
            input=dict(vertices=src_v),
            candidate=dict(claimed_status='accepted', vertices=fused),
        ),
        dict(
            must_pass=True,
            fail_is_corpus_defect=True,
            passing_tests=['independent_affine_oracle', 'staged_vs_fused_affine'],
            fused_closed_form='2*x+0.25',
            staged_closed_form='(x+x)+0.25',
            derivation='generator stores fused 2*x+0.25; harness must recompute with a staged route and f32-domain K1 check',
            not_derived_from='Etalon generated AffineF32 helper',
            tolerances=k1_tolerances(),
            scales=dict(nonzero_reference_scale=2.25),
            conditioning=dict(well_conditioned=True, coordinate_range='unit_tet'),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=902)
    # 1D Dirichlet path with pinned ends 0 and 4; interior solution is the linear ramp.
    add('identity-dirichlet-path', 'identity',
        dict(
            kind='operator_candidate',
            matrix=[[2.0, -1.0, 0.0], [-1.0, 2.0, -1.0], [0.0, -1.0, 2.0]],
            rhs=[0.0, 0.0, 4.0],
            mass=[1.0, 1.0, 1.0],
            pinned_endpoints=[0.0, 4.0],
            candidate=dict(claimed_status='accepted', solution=[1.0, 2.0, 3.0]),
        ),
        dict(
            must_pass=True,
            fail_is_corpus_defect=True,
            passing_tests=['true_residual_zero', 'closed_form_eigenvalues', 'nullspace_dimension'],
            solution=[1.0, 2.0, 3.0],
            true_residual=[0.0, 0.0, 0.0],
            nullspace_dimension=0,
            eigenvalues=[2.0 - 2.0 * math.cos(k * math.pi / 4.0) for k in (1, 2, 3)],
            eigenvalue_closed_form='2-2*cos(k*pi/4) for k=1,2,3',
            characteristic_polynomial_roots=['2-sqrt(2)', '2', '2+sqrt(2)'],
            mode_count=3,
            derivation=(
                'Linear ramp on a 3-node Dirichlet path matches Ax=b with b=[0,0,4]. '
                'Eigenvalues of the SPD tridiagonal are the standard 1D Dirichlet spectrum, '
                'not an assembled production cotangent Laplacian.'
            ),
            not_derived_from='QualiaDB ddg_operators or Jacobi iterate-change residual',
            tolerances=o1_tolerances(0, extra=dict(first_nonzero_modes_claimed=3, residual_exact=True)),
            scales=dict(nonzero_reference_scale=4.0),
            conditioning=dict(spd=True, condition_number=(2.0 + math.sqrt(2.0)) / (2.0 - math.sqrt(2.0))),
            nullspace=dict(dimension=0, pinned=True),
        ),
        partition='test', seed=903,
        exclusions=['Only three nonzero modes exist; no twenty-mode O1 claim.'])
    add('identity-constant-neumann-mode', 'identity',
        dict(
            kind='operator_candidate',
            matrix=[[1.0, -1.0, 0.0], [-1.0, 2.0, -1.0], [0.0, -1.0, 1.0]],
            field=[1.0, 1.0, 1.0],
            candidate=dict(claimed_status='accepted', operator_result=[0.0, 0.0, 0.0]),
        ),
        dict(
            must_pass=True,
            fail_is_corpus_defect=True,
            passing_tests=['nullspace_vector', 'operator_annihilates_constants'],
            operator_result=[0.0, 0.0, 0.0],
            nullspace_dimension=1,
            nullspace_vector=[1.0, 1.0, 1.0],
            derivation='Neumann 1D Laplacian rows sum to zero; constants are the exact 1-dimensional nullspace',
            not_derived_from='QualiaDB DDG assembly',
            tolerances=o1_tolerances(1, extra=dict(first_nonzero_modes_claimed=2)),
            scales=dict(nonzero_reference_scale=1.0),
            conditioning=dict(singular=True, declared_nullspace=1),
            nullspace=dict(dimension=1, vector=[1.0, 1.0, 1.0]),
        ),
        partition='test', seed=904)
    tet = standard_tet(sign_z=1.0)
    rotation = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]
    translation = [2.0, 3.0, 4.0]
    rest = [0.25, 0.25, 0.25]
    deformed = [
        rotation[0][0] * rest[0] + rotation[0][1] * rest[1] + rotation[0][2] * rest[2] + translation[0],
        rotation[1][0] * rest[0] + rotation[1][1] * rest[1] + rotation[1][2] * rest[2] + translation[1],
        rotation[2][0] * rest[0] + rotation[2][1] * rest[1] + rotation[2][2] * rest[2] + translation[2],
    ]
    add('identity-rigid-tetra', 'identity',
        dict(
            kind='transfer_candidate',
            vertices=tet['vertices'],
            tets=tet['tets'],
            barycentric=[0.25, 0.25, 0.25, 0.25],
            rotation=rotation,
            translation=translation,
            skin_weights=[0.25, 0.75],
            skin_joints=[0, 1],
            candidate=dict(claimed_status='accepted', rest_point=rest, deformed_point=deformed),
        ),
        dict(
            must_pass=True,
            fail_is_corpus_defect=True,
            passing_tests=['rigid_motion_transfer', 'quantized_weight_sum', 'positive_volume'],
            rest_point=rest,
            deformed_point=deformed,
            volume=1.0 / 6.0,
            volume_numerator=1,
            volume_denominator=6,
            quantized_weight_sum=65535,
            inversions=0,
            derivation=(
                'Barycentre of the standard tet is (1,1,1)/4. Rotation is +90 deg about z, '
                'then translation (2,3,4). u16 skin weights round(0.25*65535)+round(0.75*65535)=16384+49151=65535.'
            ),
            not_derived_from='QualiaDB dual_quaternion_skinning or FEMC adapters',
            tolerances=dict(profile='T1', decoded_barycentric_sum=65535, rigid_motion_exact=True),
            scales=dict(nonzero_reference_scale=1.0 / 6.0),
            conditioning=dict(rotation_orthonormal=True, volume_sign=1),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=905)


def _holdout_family(add):
    cube = normalized_cube_sphere()
    add('holdout-normalized-cube-sphere', 'holdout',
        dict(kind='triangle_mesh', **cube),
        dict(
            vertex_surface='x*x+y*y+z*z=1',
            radial_vertex_error=0.0,
            mesh_is_analytic_surface=False,
            expected_components=1,
            boundary_loops=0,
            euler_characteristic=2,
            vertex_count=8,
            triangle_count=12,
            derivation='cube corners normalised by 1/sqrt(3); distinct tessellation from the octahedron tune fixture',
            not_derived_from='analytic-sphere-octahedron',
            tolerances=g1_tolerances(2.0 / math.sqrt(3.0)),
            scales=dict(radius=1.0, extent_diagonal=2.0 / math.sqrt(3.0), nonzero_reference_scale=2.0 / math.sqrt(3.0)),
            conditioning=dict(holdout=True, tessellation='normalized_cube'),
            nullspace=dict(dimension=0),
            holdout_policy='must_not_be_used_for_tuning_or_threshold_search',
        ),
        partition='holdout', seed=1001,
        exclusions=['Face interiors are not on the unit sphere. Holdout tessellation must not tune octahedron thresholds.'])
    rotated = plane(axis='y')
    add('holdout-rotated-plane', 'holdout',
        dict(kind='triangle_mesh', vertices=rotated['vertices'], triangles=rotated['triangles'],
             axis=rotated['axis'], scale=1.0, origin=rotated['origin']),
        dict(
            surface=rotated['surface'],
            reference_area=1.0,
            exact_normal=rotated['exact_normal'],
            expected_components=1,
            boundary_loops=1,
            derivation='unit square in the y=0 plane; holdout against z=0 axis-aligned cheats',
            not_derived_from='analytic-plane-scale-1',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(construction_scale=1.0, extent_diagonal=math.sqrt(2.0), nonzero_reference_scale=math.sqrt(2.0)),
            conditioning=dict(holdout=True, axis='y'),
            nullspace=dict(dimension=0),
            holdout_policy='must_not_be_used_for_tuning_or_threshold_search',
        ),
        partition='holdout', seed=1002)
    add('holdout-camera-near-cut', 'holdout',
        dict(
            kind='view_selection',
            camera=dict(near=0.1, position=[0.0, 0.0, 0.0], look=[0.0, 0.0, 1.0]),
            aabb_min=[-0.1, -0.1, 0.05],
            aabb_max=[0.1, 0.1, 0.2],
            hierarchy={'root': ['left', 'right']},
            resident=['root', 'left'],
            requested_cut=['left', 'right'],
            previous_occlusion_visible=False,
        ),
        dict(
            intersects_near_plane=True,
            allowed_complete_cut=['root'],
            forbidden_partial_cut=['left'],
            camera_cut_invalidates_old_occlusion=True,
            derivation=(
                'AABB z in [0.05,0.2] crosses near=0.1. A complete root cut is legal; a left-only '
                'cut while right is requested is a missing-page crack. Stale occlusion from a prior camera is invalid.'
            ),
            not_derived_from='Etalon culling occupancy as a completeness proof',
            tolerances=k1_tolerances(extra=dict(profile='view-selection', integer_topology_exact=True)),
            scales=dict(near=0.1, aabb_z_min=0.05, aabb_z_max=0.2),
            conditioning=dict(holdout=True, near_plane_clip=True),
            nullspace=dict(dimension=0),
            holdout_policy='must_not_be_used_for_tuning_or_threshold_search',
        ),
        partition='holdout', seed=1003)
    hold_shell = plane(origin=(50.0, -20.0, 8.0))
    gap = 2e-4
    hv = list(hold_shell['vertices']) + [[x, y, z + gap] for x, y, z in hold_shell['vertices']]
    ht = list(hold_shell['triangles']) + [[4, 6, 5], [4, 7, 6]]
    add('holdout-translated-thin-shell', 'holdout',
        dict(kind='triangle_mesh', vertices=hv, triangles=ht, sheet_gap=gap, origin=[50.0, -20.0, 8.0]),
        dict(
            expected_components=2,
            sheet_gap=gap,
            forbid_sheet_merge=True,
            derivation='holdout thin shell at a moderate translation not used in the tune/test thin or large-origin fixtures',
            not_derived_from='thin-double-sheet or large-origin-small-feature',
            tolerances=g1_tolerances(math.sqrt(2.0)),
            scales=dict(extent_diagonal=math.sqrt(2.0 + gap * gap), nonzero_reference_scale=math.sqrt(2.0), feature_thickness=gap),
            conditioning=dict(holdout=True, translation_norm=math.sqrt(50.0 ** 2 + 20.0 ** 2 + 8.0 ** 2)),
            nullspace=dict(dimension=0),
            holdout_policy='must_not_be_used_for_tuning_or_threshold_search',
        ),
        partition='holdout', seed=1004)


def _maximum_family(add):
    seed = 1101
    add('maximum-size-descriptor', 'maximum_size_descriptor',
        dict(**maximum_size_descriptor(seed)),
        dict(
            admit_only_if_full_working_set_fits=True,
            materialize_into_repository=False,
            euler_triangle_count=2 * PROFILE_VERTEX_CAP - 4,
            vertex_count_cap=PROFILE_VERTEX_CAP,
            triangle_count_cap=PROFILE_TRIANGLE_CAP,
            derivation='closed unit bipyramid over a 49998-gon: V=n+2=50000, F=2n=99996, chi=2; generated only at execution',
            not_derived_from='a checked-in dense mesh',
            tolerances=k1_tolerances(extra=dict(profile='R1', cold_byte_limit=COLD_BYTE_LIMIT)),
            scales=dict(vertex_count_cap=PROFILE_VERTEX_CAP, triangle_count_cap=PROFILE_TRIANGLE_CAP, cold_byte_limit=COLD_BYTE_LIMIT),
            conditioning=dict(execution_time_generator_required=True, repository_bytes='descriptor_only'),
            nullspace=dict(dimension=0),
        ),
        partition='test', seed=seed,
        exclusions=[
            'Descriptor requires a bounded generator at execution; not a materialized maximum mesh.',
            'Do not count publication of this JSON file as an executed maximum-size test.',
        ])


def corpus():
    cases = []

    def add(name, family, data, oracle, **kwargs):
        cases.append(fixture(name, family, data, oracle, **kwargs))

    _analytic_family(add)
    _locked_family(add)
    _mixed_family(add)
    _uv_family(add)
    _thin_family(add)
    _translated_family(add)
    _empty_family(add)
    _known_bad_family(add)
    _identity_family(add)
    _holdout_family(add)
    _maximum_family(add)
    validate_cases(cases)
    return cases


def validate_cases(cases):
    required = ('schema', 'id', 'family', 'partition', 'seed', 'provenance',
                'license', 'data', 'oracle', 'exclusions', 'execution_status')
    seen = set()
    families_present = set()
    partitions = set()
    for case in cases:
        missing = [key for key in required if key not in case]
        if missing:
            raise ValueError(f'{case.get("id")}: missing {missing}')
        if case['id'] in seen:
            raise ValueError(f'duplicate fixture id {case["id"]}')
        seen.add(case['id'])
        if case['schema'] != SCHEMA:
            raise ValueError(f'{case["id"]}: wrong schema')
        if case['execution_status'] != 'not_executed':
            raise ValueError(f'{case["id"]}: fixtures are never passing test results')
        if case['family'] not in FAMILIES:
            raise ValueError(f'{case["id"]}: unknown family')
        families_present.add(case['family'])
        partitions.add(case['partition'])
        oracle = case['oracle']
        for key in ('derivation', 'not_derived_from', 'tolerances', 'scales', 'conditioning', 'nullspace'):
            if key not in oracle:
                raise ValueError(f'{case["id"]}: oracle missing {key}')
        if case['family'] == 'known_bad' and oracle.get('must_fail') is not True:
            raise ValueError(f'{case["id"]}: known-bad must set must_fail')
        if case['family'] == 'identity' and oracle.get('must_pass') is not True:
            raise ValueError(f'{case["id"]}: identity must set must_pass')
        if case['family'] == 'maximum_size_descriptor':
            if case['data'].get('materialization') != 'bounded_generated_input_required':
                raise ValueError('maximum-size fixture must be a descriptor')
            if case['data'].get('do_not_write_mesh_into_repository') is not True:
                raise ValueError('maximum-size fixture must forbid repository materialization')
        if case['family'] == 'holdout' and case['partition'] != 'holdout':
            raise ValueError(f'{case["id"]}: holdout family requires holdout partition')
    missing_families = [name for name in FAMILIES if name not in families_present]
    if missing_families:
        raise ValueError(f'missing families {missing_families}')
    if partitions != {'tune', 'test', 'holdout'}:
        raise ValueError(f'expected tune/test/holdout partitions, got {partitions}')
    canonical_bytes = sum(len(canonical(case)) for case in cases)
    if canonical_bytes > MAX_BYTES:
        raise ValueError('corpus exceeds 4 MiB before publication')


def write_corpus(out: Path, byte_budget=MAX_BYTES, replace_existing=False):
    if not 0 <= byte_budget <= MAX_BYTES:
        raise ValueError('corpus byte budget must be within 0..4 MiB')
    out = Path(out)
    fixtures = corpus()
    algo, primary_fn, algo_note = hash_backend()
    artifacts = []
    entries = []
    for case in fixtures:
        data = canonical(case)
        digest = digest_bytes(data, algo, primary_fn)
        name = case['id'] + '.json'
        artifacts.append((name, data))
        entries.append(dict(
            file=name,
            id=case['id'],
            family=case['family'],
            family_index=case['family_index'],
            partition=case['partition'],
            seed=case['seed'],
            **digest,
        ))
    family_counts = {name: 0 for name in FAMILIES}
    for case in fixtures:
        family_counts[case['family']] += 1
    manifest = dict(
        schema=MANIFEST_SCHEMA,
        family_count=len(FAMILIES),
        families=[{
            'id': name,
            'index': index + 1,
            'fixture_count': family_counts[name],
            'spec_exposure': REQ37_EXPOSURE[name],
        } for index, name in enumerate(FAMILIES)],
        partitions=['tune', 'test', 'holdout'],
        hash_algorithm=algo,
        hash_algorithm_note=algo_note,
        canonicalization='UTF-8 JSON, lexical object keys, compact separators, trailing newline; not P02 contract canonicalization',
        fixtures=entries,
        qualification_status='not_executed',
        production_implementation_compared=False,
        maximum_size_case='descriptor_only_requires_execution_generator',
        oracle_harness='scripts/production_core/oracle_harness.py',
        visual_capture_recipe=visual_capture_recipe(),
        baseline_runner=dict(
            command='python scripts/production_core/oracle_harness.py --corpus docs/production-core/corpus',
            role='independent fixture publication and analytic oracle check',
            production_comparison='not_executed',
            visual_capture='recipe_only',
        ),
        notes=[
            'Fixtures are inputs and independently specified expectations, never passing test results.',
            'Oracles differ in derivation from QualiaDB/Etalon/Hypermedia implementation helpers.',
            'Known-bad candidates must fail; identity candidates must pass relevant tests.',
            'Holdout fixtures and holdout views must not be used for tuning or threshold search.',
            'There is no train split: these are analytic qualification cases, not a learned-search corpus.',
        ],
    )
    manifest_bytes = canonical(manifest)
    artifacts.append(('manifest.json', manifest_bytes))
    total = sum(len(data) for _, data in artifacts)
    if total > byte_budget:
        raise ValueError('corpus output byte budget exhausted before publication')
    if out.exists():
        if not replace_existing:
            raise FileExistsError(f'corpus directory already exists: {out}')
        if not out.is_dir():
            raise NotADirectoryError(str(out))
        for child in out.iterdir():
            if child.is_file() and child.suffix == '.json':
                child.unlink()
            else:
                raise ValueError(f'refusing to replace non-json corpus entry {child}')
    else:
        out.mkdir(parents=True, exist_ok=False)
    for name, data in artifacts:
        with (out / name).open('xb') as handle:
            handle.write(data)
    return dict(
        manifest=manifest,
        fixture_count=len(fixtures),
        hash_algorithm=algo,
        total_bytes=total,
        out=str(out),
    )


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--byte-budget', type=int, default=MAX_BYTES)
    parser.add_argument('--replace-existing', action='store_true')
    args = parser.parse_args()
    result = write_corpus(args.out, args.byte_budget, replace_existing=args.replace_existing)
    print(json.dumps(dict(
        fixtures=result['fixture_count'],
        families=len(FAMILIES),
        hash_algorithm=result['hash_algorithm'],
        total_bytes=result['total_bytes'],
        status='generated_unexecuted',
        qualification_status='not_executed',
    ), indent=2, sort_keys=True))
