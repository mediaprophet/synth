/**
 * tools/rights_fiduciary/index.js — Rights, Fiduciary Governance & Permissions Toolbox
 * Principle: Cryptographically enforced duty-of-care, permission scopes, and sovereign DID key identity.
 */

export const rightsFiduciaryToolbox = {
  id: 'rights_fiduciary',
  title: 'Rights, Fiduciary & Sovereign Identity',
  icon: '🔒',
  domain: 'rights',
  honesty: 'partial',
  sections: [
    {
      title: 'Permission Scopes & Duty of Care',
      tools: [
        { id: 'authors_group', label: '👥 Project Authors Circle (Full Sovereign Access)', honesty: 'partial' },
        { id: 'duty_of_care', label: '🛡️ Fiduciary Duty-of-Care Gate L3', honesty: 'partial' },
        { id: 'compliance_tag', label: '🏷️ Tag #catchment-compliance', honesty: 'partial' }
      ]
    },
    {
      title: 'DID Credentials & Merkle Audit',
      tools: [
        { id: 'did_sign', label: '✍️ Sign with did:qualia:timothy_charles_holborn', honesty: 'partial' },
        { id: 'audit_log', label: '📜 Export Tamper-Proof Merkle Audit Certificate', honesty: 'partial' }
      ]
    }
  ]
};
