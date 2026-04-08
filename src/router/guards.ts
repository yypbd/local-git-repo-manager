import type { NavigationGuardWithThis } from "vue-router";
import { getBootstrap } from "@/composables/bootstrap";
import { ROUTE_PATHS } from "./routes";

export const onboardingGuard: NavigationGuardWithThis<undefined> = async (to) => {
  const bootstrap = await getBootstrap();
  const onboardingDone = Boolean(bootstrap?.confirmedDataRoot);
  if (to.path === ROUTE_PATHS.home) {
    return true;
  }
  if (!onboardingDone && to.path !== ROUTE_PATHS.onboarding) {
    return { path: ROUTE_PATHS.onboarding };
  }
  if (onboardingDone && to.path === ROUTE_PATHS.onboarding) {
    return { path: ROUTE_PATHS.projects };
  }
  return true;
};
