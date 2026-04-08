<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { getBootstrap } from "@/composables/bootstrap";
import { ROUTE_PATHS } from "@/router/routes";

const router = useRouter();

onMounted(async () => {
  const bootstrap = await getBootstrap();
  const onboardingDone = Boolean(bootstrap?.confirmedDataRoot);
  const target = onboardingDone ? ROUTE_PATHS.projects : ROUTE_PATHS.onboarding;
  void router.replace(target);
});
</script>

<template>
  <section class="boot">
    <h1>초기화 중...</h1>
    <p>앱 시작 상태를 확인하고 있습니다.</p>
  </section>
</template>

<style scoped>
.boot {
  height: 100vh;
  display: grid;
  place-content: center;
  gap: 10px;
  text-align: center;
}
</style>
