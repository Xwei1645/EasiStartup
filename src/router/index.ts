import { createRouter, createWebHistory } from "vue-router";
import Settings from "../views/Settings.vue";
import StartupEditor from "../views/StartupEditor.vue";

const routes = [
  {
    path: "/",
    redirect: "/startup-editor",
  },
  {
    path: "/startup-editor",
    name: "StartupEditor",
    component: StartupEditor,
  },
  {
    path: "/settings",
    name: "Settings",
    component: Settings,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;