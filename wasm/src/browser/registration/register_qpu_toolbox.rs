//! Part of poet browser toolbox registration — QPU quantum computing & oracle tools.

use super::*;

fn qpu_tool(
    id: &'static str,
    label: &'static str,
    scope: &'static str,
    action: ActionType,
    description: &'static str,
) -> Box<dyn crate::tool_chest::core::tool::Tool> {
    Box::new(SimpleTool::new(
        ToolMetadata {
            id: id.into(),
            label: label.into(),
            icon: "quantum".into(),
            kind: ToolKind::RunAction,
            capability_scope: Some(scope.into()),
            ontology_prefix: "qpu".into(),
            description: description.into(),
        },
        action,
    ))
}

fn qpu_place_container(
    id: &'static str,
    label: &'static str,
    description: &'static str,
) -> Box<dyn crate::tool_chest::core::tool::Tool> {
    Box::new(SimpleTool::new(
        ToolMetadata {
            id: id.into(),
            label: label.into(),
            icon: "quantum".into(),
            kind: ToolKind::PlaceContainer,
            capability_scope: Some(SCOPE_PLACE.into()),
            ontology_prefix: "qpu".into(),
            description: description.into(),
        },
        ActionType::Query,
    ))
}

pub(super) fn register_qpu_toolbox(reg: &mut Registry) {
    let execution_tools: Vec<Box<dyn crate::tool_chest::core::tool::Tool>> = vec![
        qpu_tool(
            "qpu:circuit_execute",
            "Execute Circuit",
            "Qpu.execute_circuit",
            ActionType::Invoke,
            "Execute parameterized quantum circuit or OpenQASM on calibrated QPU or statevector simulator.",
        ),
        qpu_tool(
            "qpu:statevector_to_quins",
            "Amplitudes to NQuins",
            "Qpu.statevector_to_quins",
            ActionType::Invoke,
            "Project complex amplitude vector into 48-byte deterministic NQuin topological pointers with ECC XOR parity.",
        ),
        qpu_tool(
            "qpu:bell_state_verifier",
            "Verify Bell State",
            "Qpu.bell_state_verifier",
            ActionType::Invoke,
            "Generate and verify 2-qubit entangled Bell state (|Φ+⟩) with quantum state tomography.",
        ),
        qpu_tool(
            "qpu:bloch_sphere_project",
            "Project Bloch Sphere",
            "Qpu.bloch_sphere_project",
            ActionType::Query,
            "Project single-qubit density matrix into 3D Stokes vector (Sx, Sy, Sz) coordinates on the Bloch sphere.",
        ),
    ];

    let dqc_scheduler_tools: Vec<Box<dyn crate::tool_chest::core::tool::Tool>> = vec![
        qpu_tool(
            "qpu:dqc_scheduler",
            "QGroup Batch Scheduler",
            "Qpu.schedule_dqc",
            ActionType::Invoke,
            "Sort and cluster VQE and multi-tenant job queues using the QGroup heuristic (circuit depth & shot count similarity) to eliminate pipeline stalls.",
        ),
        qpu_tool(
            "qpu:dqc_cluster_queue",
            "DQC Cluster Queue",
            "Qpu.cluster_queue",
            ActionType::Query,
            "Inspect and balance distributed quantum execution queues across active P2P DQC swarm nodes.",
        ),
        qpu_tool(
            "qpu:noise_mitigate",
            "Mitigate Noise",
            "Qpu.noise_mitigate",
            ActionType::Invoke,
            "Apply Zero-Noise Extrapolation (ZNE Richardson) and TREX readout mitigation to expectation measurements.",
        ),
    ];

    let chemistry_vqe_tools: Vec<Box<dyn crate::tool_chest::core::tool::Tool>> = vec![
        qpu_tool(
            "qpu:vqe_energy_estimator",
            "VQE Energy Estimator",
            "Qpu.vqe_energy_estimator",
            ActionType::Invoke,
            "Estimate molecular ground-state electronic energy for second-quantized Hamiltonians via Variational Quantum Eigensolver.",
        ),
        qpu_tool(
            "qpu:uccsd_ansatz",
            "Generate UCCSD Ansatz",
            "Qpu.uccsd_ansatz",
            ActionType::Invoke,
            "Generate Unitary Coupled-Cluster Singles and Doubles (UCCSD) parameterized excitation ansatz circuits.",
        ),
        qpu_tool(
            "qpu:pauli_group_decompose",
            "Decompose Pauli Group",
            "Qpu.pauli_group_decompose",
            ActionType::Query,
            "Partition molecular electronic Hamiltonian into qubit-wise commuting Pauli operator groups for parallel shot measurement.",
        ),
    ];

    let container_tools: Vec<Box<dyn crate::tool_chest::core::tool::Tool>> = vec![
        qpu_place_container(
            "qpu:place_circuit_composer",
            "+ Circuit Composer",
            "Place interactive visual quantum circuit composer on the Poet manifold.",
        ),
        qpu_place_container(
            "qpu:place_bloch_multiview",
            "+ Bloch Multiview",
            "Place multi-qubit Bloch sphere and density matrix visualizer on the Poet manifold.",
        ),
        qpu_place_container(
            "qpu:place_qpu_monitor",
            "+ QPU Monitor",
            "Place real-time QPU telemetry, gate fidelity and calibration monitor on the Poet manifold.",
        ),
    ];

    reg.register_toolbox(Toolbox::new(
        ToolboxMetadata {
            id: "qpu".into(),
            label: "Quantum Processing Unit (QPU)".into(),
            icon: "quantum".into(),
            ontology_prefix: "qpu".into(),
            description: "Hardware-accelerated quantum simulation, DQC distributed scheduling, VQE molecular Hamiltonians, and topological state projection.".into(),
            enabled_by_default: true,
            family: "qpu".into(),
        },
        vec![
            ToolChain::new(
                ToolChainMetadata {
                    id: "qpu:execution".into(),
                    label: "Quantum Execution & Simulation".into(),
                    icon: "quantum".into(),
                    description: "Execute quantum circuits, verify entanglement, and project amplitudes to topological NQuins.".into(),
                },
                execution_tools,
            ),
            ToolChain::new(
                ToolChainMetadata {
                    id: "qpu:dqc_scheduler".into(),
                    label: "DQC Cloud & QGroup Scheduler".into(),
                    icon: "cluster".into(),
                    description: "Cluster multi-tenant quantum execution queues using depth/shot heuristics and zero-noise extrapolation.".into(),
                },
                dqc_scheduler_tools,
            ),
            ToolChain::new(
                ToolChainMetadata {
                    id: "qpu:chemistry_vqe".into(),
                    label: "Quantum Chemistry & VQE".into(),
                    icon: "chemistry".into(),
                    description: "Estimate molecular ground states via VQE, synthesize UCCSD ansatzes, and decompose Hamiltonian Pauli strings.".into(),
                },
                chemistry_vqe_tools,
            ),
            ToolChain::new(
                ToolChainMetadata {
                    id: "qpu:containers".into(),
                    label: "Quantum Canvas Containers".into(),
                    icon: "container".into(),
                    description: "Embed visual circuit composers, Bloch sphere multiviews, and live QPU monitors directly onto the manifold.".into(),
                },
                container_tools,
            ),
        ],
    ));
}
