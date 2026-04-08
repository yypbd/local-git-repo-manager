/** `git status` 상대 경로를 트리 뷰용으로 묶기 위한 노드 */
export type PathTreeNode = {
  name: string;
  fullPath: string;
  children: PathTreeNode[];
};

function sortPathTree(nodes: PathTreeNode[]): void {
  nodes.sort((a, b) => {
    const aDir = a.children.length > 0 ? 0 : 1;
    const bDir = b.children.length > 0 ? 0 : 1;
    if (aDir !== bDir) return aDir - bDir;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
  });
  for (const n of nodes) sortPathTree(n.children);
}

/** 플랫 경로 목록을 디렉터리 우선·이름순 트리로 만듭니다. */
export function buildPathTree(paths: string[]): PathTreeNode[] {
  const roots: PathTreeNode[] = [];
  const uniq = Array.from(
    new Set(paths.map((p) => p.replace(/\\/g, "/").trim()).filter(Boolean)),
  );

  for (const norm of uniq) {
    const segments = norm.split("/").filter(Boolean);
    if (segments.length === 0) continue;

    let siblings = roots;
    let prefix = "";

    for (let i = 0; i < segments.length; i++) {
      const seg = segments[i]!;
      prefix = prefix ? `${prefix}/${seg}` : seg;

      let node = siblings.find((n) => n.name === seg);
      if (!node) {
        node = { name: seg, fullPath: prefix, children: [] };
        siblings.push(node);
      }

      if (i === segments.length - 1) break;
      siblings = node.children;
    }
  }

  sortPathTree(roots);
  return roots;
}
