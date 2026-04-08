import { createRouter, createWebHashHistory } from "vue-router";
import { onboardingGuard } from "./guards";
import { ROUTE_PATHS } from "./routes";

import BootGatePage from "@/pages/BootGatePage.vue";
import OnboardingPage from "@/pages/OnboardingPage.vue";
import ProjectsWorkspacePage from "@/pages/ProjectsWorkspacePage.vue";
import NotFoundPage from "@/pages/NotFoundPage.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: ROUTE_PATHS.home, name: "home", component: BootGatePage },
    { path: ROUTE_PATHS.onboarding, name: "onboarding", component: OnboardingPage },
    { path: ROUTE_PATHS.projects, name: "projects", component: ProjectsWorkspacePage },
    { path: ROUTE_PATHS.projectDetail, name: "project-detail", component: ProjectsWorkspacePage },
    { path: ROUTE_PATHS.settings, redirect: { path: ROUTE_PATHS.projects } },
    { path: ROUTE_PATHS.notFound, name: "not-found", component: NotFoundPage },
  ],
});

router.beforeEach(onboardingGuard);

export default router;
