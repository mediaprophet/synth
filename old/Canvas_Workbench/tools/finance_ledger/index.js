/**
 * tools/finance_ledger/index.js — Financial Economics & Double-Entry Ledger Toolbox
 * Structure:
 * Toolbox -> Toolchains (Socket-Sets) -> Tools (Instruments)
 */

export const financeLedgerToolbox = {
  id: 'finance_ledger',
  title: 'Financial Economics & Double-Entry Ledger',
  icon: '💰',
  color: 'var(--strata-fin)',
  description: 'Double-entry linked accounting quads, milestone escrow pools, capital flows, and statutory tax governance.',
  
  toolchains: [
    {
      id: 'tc_ledger_ops',
      title: 'Double-Entry Quad Operations',
      icon: '📊',
      badge: '3 Instruments',
      tools: [
        { id: 'post_quad_tx', label: '➕ Post Quad Entry', icon: '➕', honesty: 'live' },
        { id: 'reconcile_accounts', label: '⚖️ Reconcile Accounts', icon: '⚖️', honesty: 'live' },
        { id: 'audit_digest', label: '🛡️ Generate Audit Digest', icon: '🛡️', honesty: 'live' }
      ]
    },
    {
      id: 'tc_escrow_pools',
      title: 'Milestone Escrows & Capital Pools',
      icon: '🔐',
      badge: '3 Instruments',
      tools: [
        { id: 'create_escrow', label: '🔒 Lock Milestone Escrow', icon: '🔒', honesty: 'live' },
        { id: 'multisig_release', label: '✍️ Multi-Sig Escrow Release', icon: '✍️', honesty: 'live' },
        { id: 'disburse_tranche', label: '💸 Disburse Tranche', icon: '💸', honesty: 'present' }
      ]
    },
    {
      id: 'tc_fiscal_compliance',
      title: 'Fiscal Rules & Gas Settlement',
      icon: '📜',
      badge: '3 Instruments',
      tools: [
        { id: 'shacl_fiscal_verify', label: '✅ SHACL Fiscal Verification', icon: '✅', honesty: 'partial' },
        { id: 'gas_settlement', label: '⚡ Settle Compute Gas', icon: '⚡', honesty: 'live' },
        { id: 'export_rdf_ledger', label: '📑 Export RDF 1.2 Ledger', icon: '📑', honesty: 'live' }
      ]
    }
  ]
};
