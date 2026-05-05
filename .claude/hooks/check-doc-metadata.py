import sys
import json

try:
    data = json.load(sys.stdin)
    tool_input = data.get('tool_input', {})
    path = tool_input.get('file_path', '')
    content = tool_input.get('content', '')

    norm = path.replace('\\', '/')

    if path.endswith('.md') and '/docs/' in norm:
        if not content.startswith('---'):
            msg = (
                "BLOCKED: docs/*.md files require YAML frontmatter.\n"
                "Required fields: doc_id, title, status, owner, created_at, product_area\n"
                "See: docs/spec/04_METADATA_STANDARD.md\n"
                "File: " + path
            )
            print(msg, file=sys.stderr)
            sys.exit(1)
except Exception:
    pass

sys.exit(0)
