/** 저장소 루트 — `repoPath`와 `repo_path`를 함께 넘깁니다. */
export function repoPathArgs(repoPath: string) {
  return { repoPath, repo_path: repoPath };
}

/** `git_archive_export`용 출력 부모 폴더. */
export function outputParentDirArgs(outputParentDir: string) {
  return { outputParentDir, output_parent_dir: outputParentDir };
}
