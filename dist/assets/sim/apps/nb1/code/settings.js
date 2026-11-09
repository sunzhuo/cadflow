const HEX_COLOR_RE = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i;
const CANVAS_BACKGROUND_VALUES = new Set(["#141428", "#ffffff"]);
const DEFAULT_RANDOM_SEED_EXPRESSION = "Date.now() % 10000";

export const SCENE_ID = "nb1";
export const SCENE_LABEL = "nb1";

export const CANVAS_BACKGROUND_OPTIONS = Object.freeze([
  { label: "深色", value: "#141428" },
  { label: "白色", value: "#ffffff" },
]);

export const RENDERER_BACKEND_OPTIONS = Object.freeze([
  { label: "WebGPU", value: "webgpu" },
  { label: "WebGL", value: "webgl" },
]);

export const DEFAULT_UI_SETTINGS = Object.freeze({
  rendererBackend: "webgpu",
  randomSeed: DEFAULT_RANDOM_SEED_EXPRESSION,
  backgroundColor: "#141428",
  simulationDuration: -1,
  defaultSpeed: 5,
});

export const DEFAULT_WAREHOUSE_SETTINGS = Object.freeze({
  straightSpeed: 10,
  turnSpeed: 2,
  diagonalSpeed: 2,
  maxAccel: 8,
  taskMean: 5,
  igvCount: 32,
  handlingDuration: 1,
  showLockZone: true,
});

export const DEFAULT_SCENE_SETTINGS = DEFAULT_WAREHOUSE_SETTINGS;

export const SETTINGS_SCHEMA = Object.freeze([
  {
    title: "显示设置",
    description: "显示引擎会在页面刷新后生效，画布背景会立即更新。",
    fields: [
      {
        scope: "ui",
        key: "rendererBackend",
        label: "显示引擎",
        type: "select",
        options: RENDERER_BACKEND_OPTIONS,
      },
      {
        scope: "ui",
        key: "backgroundColor",
        label: "画布背景",
        type: "select",
        options: CANVAS_BACKGROUND_OPTIONS,
      },
    ],
  },
  {
    title: "仿真设置",
    description: "随机数种子会在启动时动态解析，仿真参数会在下一次启动仿真时生效。",
    fields: [
      {
        scope: "ui",
        key: "randomSeed",
        label: "随机数种子",
        type: "text",
        placeholder: DEFAULT_RANDOM_SEED_EXPRESSION,
      },
      {
        scope: "ui",
        key: "simulationDuration",
        label: "仿真时长",
        type: "number",
        min: -1,
        max: 1000000000,
        step: 1,
        unit: "秒",
        hint: "-1 为无穷",
      },
    ],
  },
  {
    title: "场景参数",
    description: "对应 nb1 场景设置中的核心参数，修改后会在下一次启动仿真时生效。",
    fields: [
      {
        scope: "scene",
        key: "straightSpeed",
        label: "直行速度",
        type: "number",
        min: 0.1,
        max: 50,
        step: 0.1,
        unit: "米/秒",
      },
      {
        scope: "scene",
        key: "turnSpeed",
        label: "转弯速度",
        type: "number",
        min: 0.1,
        max: 20,
        step: 0.1,
        unit: "米/秒",
      },
      {
        scope: "scene",
        key: "diagonalSpeed",
        label: "斜行速度",
        type: "number",
        min: 0.1,
        max: 20,
        step: 0.1,
        unit: "米/秒",
      },
      {
        scope: "scene",
        key: "maxAccel",
        label: "最大加减速",
        type: "number",
        min: 0.1,
        max: 50,
        step: 0.1,
        unit: "米/秒²",
      },
      {
        scope: "scene",
        key: "taskMean",
        label: "任务平均间隔",
        type: "number",
        min: 0.1,
        max: 120,
        step: 0.1,
        unit: "秒",
      },
      {
        scope: "scene",
        key: "igvCount",
        label: "初始 IGV 数量",
        type: "number",
        min: 1,
        max: 256,
        step: 1,
      },
      {
        scope: "scene",
        key: "handlingDuration",
        label: "装卸时长",
        type: "number",
        min: 0,
        max: 60,
        step: 0.1,
        unit: "秒",
      },
      {
        scope: "scene",
        key: "showLockZone",
        label: "显示锁闭区",
        type: "checkbox",
        hint: "关闭后不再绘制 IGV 周围的锁闭区",
      },
    ],
  },
]);

function clampNumber(value, fallback, { min = -Infinity, max = Infinity, round = false } = {}) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }

  const bounded = Math.min(max, Math.max(min, parsed));
  return round ? Math.round(bounded) : bounded;
}

function normalizeHexColor(value, fallback = DEFAULT_UI_SETTINGS.backgroundColor) {
  if (typeof value !== "string") {
    return fallback;
  }

  const trimmed = value.trim();
  if (!HEX_COLOR_RE.test(trimmed)) {
    return fallback;
  }

  if (trimmed.length === 4) {
    const [, red, green, blue] = trimmed;
    const expanded = `#${red}${red}${green}${green}${blue}${blue}`.toLowerCase();
    return CANVAS_BACKGROUND_VALUES.has(expanded) ? expanded : fallback;
  }

  const normalized = trimmed.toLowerCase();
  return CANVAS_BACKGROUND_VALUES.has(normalized) ? normalized : fallback;
}

function normalizeSeedExpression(value, fallback = DEFAULT_UI_SETTINGS.randomSeed) {
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : fallback;
  }

  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }

  return String(Math.trunc(parsed));
}

function normalizeSimulationDuration(value, fallback = DEFAULT_UI_SETTINGS.simulationDuration) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }

  if (parsed === -1) {
    return -1;
  }

  return parsed >= 1 ? parsed : fallback;
}

function normalizeRendererBackend(value, fallback = DEFAULT_UI_SETTINGS.rendererBackend) {
  if (typeof value !== "string") {
    return fallback;
  }

  const normalized = value.trim().toLowerCase();
  return normalized === "webgl" || normalized === "webgpu" ? normalized : fallback;
}

function normalizeBoolean(value, fallback = true) {
  if (typeof value === "boolean") {
    return value;
  }

  if (typeof value === "number") {
    if (value === 1) {
      return true;
    }
    if (value === 0) {
      return false;
    }
  }

  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (normalized === "true" || normalized === "1" || normalized === "yes" || normalized === "on") {
      return true;
    }
    if (normalized === "false" || normalized === "0" || normalized === "no" || normalized === "off") {
      return false;
    }
  }

  return fallback;
}

export function normalizeUiSettings(input = {}) {
  return {
    rendererBackend: normalizeRendererBackend(input.rendererBackend ?? input.displayEngine),
    randomSeed: normalizeSeedExpression(input.randomSeed),
    backgroundColor: normalizeHexColor(input.backgroundColor),
    simulationDuration: normalizeSimulationDuration(input.simulationDuration),
  };
}

export function normalizeWarehouseSettings(input = {}) {
  return {
    straightSpeed: clampNumber(input.straightSpeed, DEFAULT_WAREHOUSE_SETTINGS.straightSpeed, { min: 0.1, max: 100 }),
    turnSpeed: clampNumber(input.turnSpeed, DEFAULT_WAREHOUSE_SETTINGS.turnSpeed, { min: 0.1, max: 30 }),
    diagonalSpeed: clampNumber(input.diagonalSpeed, DEFAULT_WAREHOUSE_SETTINGS.diagonalSpeed, { min: 0.1, max: 30 }),
    maxAccel: clampNumber(input.maxAccel, DEFAULT_WAREHOUSE_SETTINGS.maxAccel, { min: 0.1, max: 100 }),
    taskMean: clampNumber(input.taskMean, DEFAULT_WAREHOUSE_SETTINGS.taskMean, { min: 0.1, max: 1e6 }),
    igvCount: clampNumber(input.igvCount, DEFAULT_WAREHOUSE_SETTINGS.igvCount, { min: 1, max: 256, round: true }),
    handlingDuration: clampNumber(input.handlingDuration, DEFAULT_WAREHOUSE_SETTINGS.handlingDuration, { min: 0, max: 60 }),
    showLockZone: normalizeBoolean(input.showLockZone, DEFAULT_WAREHOUSE_SETTINGS.showLockZone),
  };
}

export const normalizeSceneSettings = normalizeWarehouseSettings;

export function normalizeStoredSettings(input = {}) {
  return {
    ui: normalizeUiSettings(input.ui),
    scene: normalizeWarehouseSettings(input.scene),
  };
}