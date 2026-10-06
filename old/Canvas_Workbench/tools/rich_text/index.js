/**
 * tools/rich_text/index.js — TinyMCE-Style Document & Knowledge Authoring Toolbox
 * Principle: WYSIWYG rich text formatting with native first-class semantic entity grounding,
 * NLP gazetteer sealing, Markdown source editing, and LaTeX formula support.
 */

export const richTextToolbox = {
  id: 'rich_text',
  title: 'Document & Knowledge Authoring (TinyMCE Suite)',
  icon: '📝',
  domain: 'doc',
  honesty: 'partial',
  sections: [
    {
      title: 'Typography & Block Styles',
      tools: [
        { id: 'bold', label: 'Bold (Ctrl+B)', icon: 'B', shortcut: 'Ctrl+B' },
        { id: 'italic', label: 'Italic (Ctrl+I)', icon: 'I', shortcut: 'Ctrl+I' },
        { id: 'underline', label: 'Underline (Ctrl+U)', icon: 'U', shortcut: 'Ctrl+U' },
        { id: 'code', label: 'Inline Code', icon: '</>', shortcut: 'Ctrl+`' },
        { id: 'h1', label: 'Heading 1', icon: 'H1' },
        { id: 'h2', label: 'Heading 2', icon: 'H2' },
        { id: 'quote', label: 'Blockquote', icon: '❝' },
        { id: 'list', label: 'Bullet List', icon: '•' }
      ]
    },
    {
      title: 'Semantic Context & Ontological Grounding',
      tools: [
        { id: 'entity', label: '🏷️ Tag Ontological Entity', honesty: 'partial' },
        { id: 'qualia', label: '🧠 Tag Subjective Qualia', honesty: 'partial' },
        { id: 'objective', label: '🔬 Tag Objective Fact', honesty: 'partial' },
        { id: 'normative', label: '⚖️ Tag Normative Statute', honesty: 'partial' },
        { id: 'latex', label: '$$ LaTeX Math Formula', honesty: 'partial' },
        { id: 'gazetteer', label: '🔍 Gazetteer NLP Analysis (qualia_core_db::nlp)', honesty: 'partial' }
      ]
    }
  ]
};
