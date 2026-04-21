<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  trackedChanges: number;
  untrackedFiles: number;
}>();

const { t } = useI18n();

/** 그룹·툴팁·접근성용 요약 (기존 statusDirty* 문구 재사용) */
const groupSummary = computed(() => {
  const m = props.trackedChanges;
  const n = props.untrackedFiles;
  if (m > 0 && n > 0) return t("workspace.statusDirtyBoth", { m, n });
  if (m > 0) return t("workspace.statusDirtyTrackedOnly", { m });
  if (n > 0) return t("workspace.statusDirtyUntrackedOnly", { n });
  return "";
});
</script>

<template>
  <span
    class="status-badges"
    role="group"
    :aria-label="groupSummary"
    :title="groupSummary"
  >
    <span class="status-prefix" aria-hidden="true">{{ $t("workspace.statusHasChangesLabel") }}</span>
    <span
      v-if="trackedChanges > 0"
      class="status-badge status-badge--tracked"
      :title="$t('workspace.statusBadgeTrackedTitle')"
    >
      M {{ trackedChanges }}
    </span>
    <span
      v-if="untrackedFiles > 0"
      class="status-badge status-badge--untracked"
      :title="$t('workspace.statusBadgeUntrackedTitle')"
    >
      U {{ untrackedFiles }}
    </span>
  </span>
</template>

<style scoped>
.status-badges {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 5px;
}

.status-prefix {
  font-size: 0.64rem;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: var(--folder-status-changes-prefix-fg);
  white-space: nowrap;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  border: 1px solid transparent;
  border-radius: 999px;
  padding: 0 7px;
  min-height: 20px;
  font-size: 0.68rem;
  line-height: 1;
  font-weight: 600;
}

.status-badge--tracked {
  color: var(--folder-status-tracked-fg);
  border-color: var(--folder-status-tracked-border);
  background: var(--folder-status-tracked-bg);
}

.status-badge--untracked {
  color: var(--folder-status-untracked-fg);
  border-color: var(--folder-status-untracked-border);
  background: var(--folder-status-untracked-bg);
}
</style>
