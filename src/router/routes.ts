export const ROUTE_PATHS = {
  home: "/",
  onboarding: "/onboarding",
  projects: "/projects",
  projectDetail: "/projects/:id",
  settings: "/settings",
  notFound: "/:pathMatch(.*)*",
} as const;
