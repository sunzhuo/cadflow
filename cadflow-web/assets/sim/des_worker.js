/**
 * des_worker.js —— CADFlow 离散事件仿真（DES）Web Worker 胶水宿主
 *
 * 职责：
 *  - 作为薄壳宿主运行在独立 Web Worker 中，避免阻塞 CADFlow UI 与文档编辑主线程
 *  - 动态 import() 加载业务场景脚本（如 apps/nb1/code/scene.js 或 DXF 提取的代码）
 *  - 维护由仿真时钟映射到真实时间的宏任务时间片（12ms 时间片控制让出主线程）
 *  - 通过 Comlink 暴露 RPC 控制接口（start, pause, resume, stop, setSpeed）
 *  - 通过 MessagePort 零拷贝转发紧凑二进制帧到渲染端
 */

import * as Comlink from "https://unpkg.com/comlink@4.4.2/dist/esm/comlink.mjs";
import { MessageBus, Initiator } from "./sim_sdk.js";

const DEFAULT_RUN_DURATION = Number.POSITIVE_INFINITY;
const DEFAULT_SEED_EXPRESSION = "Date.now() % 10000";
const CLOCK_EPSILON = 1e-9;
const CONTINUE_DELAY_MS = 0;
const UNLIMITED_REALTIME_BUDGET_MS = 12;
const STATUS_EMIT_INTERVAL_MS = 80;
const TARGET_RENDER_FPS = 60;

// ══════════════════════════════════════════════
//  确定性随机数与伪随机发生器包装（兼容 d3-random 闭包调用模式）
// ══════════════════════════════════════════════

function createSeededRng(seed) {
  let s = Number(seed) >>> 0;
  // LCG 算法保证确定性与跨平台一致
  function lcg() {
    s = (Math.imul(1664525, s) + 1013904223) >>> 0;
    return s / 4294967296;
  }

  return {
    uniform(min = 0, max = 1) {
      return () => min + lcg() * (max - min);
    },
    exponential(lambda = 1) {
      return () => -Math.log(1 - lcg()) / lambda;
    },
    normal(mean = 0, stdDev = 1) {
      return () => {
        let u1 = 0, u2 = 0;
        while (u1 === 0) u1 = lcg();
        while (u2 === 0) u2 = lcg();
        const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2.0 * Math.PI * u2);
        return mean + z0 * stdDev;
      };
    },
  };
}

// ══════════════════════════════════════════════
//  二进制帧编码器（严格对齐 12B Header + 24B/Entity 协议）
// ══════════════════════════════════════════════

const HEADER_BYTES = 12;
const ENTITY_BYTES = 24;
const textEncoder = new TextEncoder();

function encodeFrame(simTime, entities, { auxiliary = null } = {}) {
  const count = entities.length;
  let auxBytes = null;
  if (auxiliary && auxiliary.length > 0) {
    auxBytes = textEncoder.encode(JSON.stringify({ auxiliary }));
  }

  const buf = new ArrayBuffer(HEADER_BYTES + count * ENTITY_BYTES + (auxBytes?.length ?? 0));
  const dv = new DataView(buf);
  let off = 0;

  dv.setFloat64(off, simTime, true); off += 8;
  dv.setUint32(off, count, true);    off += 4;

  for (const e of entities) {
    dv.setUint32(off, e.id, true);    off += 4;
    dv.setUint8(off, e.type);         off += 1;
    dv.setUint8(off, e.state);        off += 1;
    off += 2; // 2 字节 padding
    dv.setFloat32(off, e.x, true);    off += 4;
    dv.setFloat32(off, e.y, true);    off += 4;
    dv.setFloat32(off, e.z, true);    off += 4;
    dv.setFloat32(off, e.rotY, true); off += 4;
  }

  if (auxBytes?.length) {
    new Uint8Array(buf, off).set(auxBytes);
  }

  return buf;
}

function collectAuxiliaryFrameData(agents) {
  const auxiliary = [];
  for (const agent of agents.values()) {
    const renderData = agent.getAuxiliaryRenderData?.();
    if (!renderData) continue;
    const items = Array.isArray(renderData) ? renderData : [renderData];
    for (const item of items) {
      if (!item || typeof item !== "object") continue;
      const ownerId = Number.isInteger(item.ownerId) ? item.ownerId : agent.id;
      auxiliary.push({ ...item, ownerId });
    }
  }
  return auxiliary;
}

// ══════════════════════════════════════════════
//  DES 引擎桥接层
// ══════════════════════════════════════════════

class DESEngineHost {
  constructor() {
    this.clock = 0;
    this.events = []; // 优先队列 [ { time, type, data, id } ]
    this.agents = new Map();
    this.planners = new Map();
    this.initiators = [];
    this.bus = new MessageBus();
    this.stats = { eventsProcessed: 0, framesPublished: 0, lastFrameRealTime: null, lastFrameSimTime: null };
    this._nextId = 1;
    this._speed = 1;
    this._duration = DEFAULT_RUN_DURATION;
    this._framePort = null;
    this._restoring = false;
    this._seed = 12345;
    this._resetRandomStreams(this._seed);
  }

  nextId() {
    return this._nextId++;
  }

  attachFramePort(port) {
    if (this._framePort && this._framePort !== port) {
      this._framePort.onmessage = null;
    }
    this._framePort = port;
    this._framePort.start?.();
  }

  emitStatus(text) {
    self.postMessage({ type: "status", text });
  }

  emitState(state) {
    self.postMessage({
      type: "simulation-state",
      state,
      clock: this.clock,
      duration: this._duration,
      seed: this._seed,
    });
  }

  isUnlimitedSpeed(value = this._speed) {
    return value === Number.POSITIVE_INFINITY || !Number.isFinite(value) || value < 0;
  }

  formatClock(value) {
    return Number.isFinite(value) ? `${value.toFixed(1)}s` : "∞";
  }

  setRunSpeed(speed) {
    this._speed = speed;
  }

  initializeRun(params) {
    for (const init of this.initiators) {
      init.onInit(this, params);
    }
  }

  reset({ speed = this._speed, duration = this._duration, seed = this._seed, sceneSettings = {} } = {}) {
    this._restoring = false;
    this._speed = speed;
    this._duration = duration;
    this._resetRandomStreams(seed);
    this._sceneSettings = sceneSettings ?? {};
    this.clock = 0;
    this.events.length = 0;
    this.agents.clear();
    this.planners.clear();
    this.initiators.length = 0;
    this.bus = new MessageBus();
    this.stats = { eventsProcessed: 0, framesPublished: 0, lastFrameRealTime: null, lastFrameSimTime: null };
    this._nextId = 1;
  }

  _resetRandomStreams(seed = this._seed) {
    this._seed = normalizeRunSeed(seed);
    this.rng = createSeededRng(this._seed);
  }

  boundTargetClock(targetClock) {
    if (Number.isFinite(this._duration)) {
      return Math.min(targetClock, this._duration);
    }
    return targetClock;
  }

  peekNextEvent() {
    return this.events.length > 0 ? this.events[0] : null;
  }

  peekNextEventTime() {
    return this.peekNextEvent()?.time ?? Number.POSITIVE_INFINITY;
  }

  hasDueEvents(clock = this.clock) {
    const nextEvent = this.peekNextEvent();
    return !!nextEvent && nextEvent.time <= clock + CLOCK_EPSILON;
  }

  scheduleEvent(delay, type, data = {}) {
    const parsedDelay = Number(delay);
    if (!Number.isFinite(parsedDelay) || parsedDelay < 0) return false;
    const time = this.clock + parsedDelay;
    const evt = { time, type, data, id: this.nextId() };

    // 二分查找插入，维持时间升序队列
    let low = 0;
    let high = this.events.length;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if (this.events[mid].time <= time) low = mid + 1;
      else high = mid;
    }
    this.events.splice(low, 0, evt);
    return true;
  }

  advanceTo(targetClock) {
    const boundedTargetClock = this.boundTargetClock(targetClock);
    if (boundedTargetClock + CLOCK_EPSILON < this.clock) return this.clock;

    const simDt = Math.max(0, boundedTargetClock - this.clock);
    this.clock = boundedTargetClock;
    this.bus.flush();

    for (const planner of this.planners.values()) {
      planner.update(simDt, this);
    }
    for (const agent of this.agents.values()) {
      agent.update(simDt, this);
    }
    return this.clock;
  }

  dispatchDueEvents() {
    let dispatched = 0;
    while (this.hasDueEvents()) {
      const evt = this.events.shift();
      this.clock = Math.max(this.clock, evt.time);
      this._dispatchEvent(evt);
      this.stats.eventsProcessed++;
      dispatched++;
    }
    this.bus.flush();
    return dispatched;
  }

  _dispatchEvent(evt) {
    for (const planner of this.planners.values()) {
      if (typeof planner.isSubscribed === "function" && planner.isSubscribed(evt.type)) {
        planner.onEvent(evt, this);
      }
    }
  }

  addAgent(agent) {
    this.agents.set(agent.id, agent);
    this.bus.register(agent);
    return agent;
  }

  removeAgent(id) {
    this.agents.delete(id);
    this.bus.unregister(id);
  }

  addPlanner(planner) {
    planner.bus = this.bus;
    this.planners.set(planner.id, planner);
    this.bus.register(planner);
    if (planner instanceof Initiator) {
      this.initiators.push(planner);
    }
    return planner;
  }

  removePlanner(id) {
    this.planners.delete(id);
    this.bus.unregister(id);
  }

  publishFrame(frameRealTime = performance.now()) {
    const entities = [];
    for (const agent of this.agents.values()) {
      entities.push(agent.toFrameEntity());
    }

    const auxiliary = collectAuxiliaryFrameData(this.agents);
    const buf = encodeFrame(this.clock, entities, { auxiliary });
    const publishedAt = Number.isFinite(frameRealTime) ? frameRealTime : performance.now();
    this.stats.framesPublished++;
    this.stats.lastFrameRealTime = publishedAt;
    this.stats.lastFrameSimTime = this.clock;

    if (this._framePort) {
      this._framePort.postMessage(buf, [buf]);
    } else {
      self.postMessage(buf, [buf]);
    }
    return publishedAt;
  }

  getRenderFrameInterval(targetFps = TARGET_RENDER_FPS) {
    const safeFps = Number(targetFps);
    return Number.isFinite(safeFps) && safeFps > 0 ? 1 / safeFps : 1 / TARGET_RENDER_FPS;
  }

  toSimDelay(realDelaySeconds) {
    const safeDelay = Math.max(0, Number(realDelaySeconds) || 0);
    return this.isUnlimitedSpeed() ? Number.POSITIVE_INFINITY : safeDelay * this._speed;
  }

  publishRenderFrame(frameStartedAt = performance.now(), targetFps = TARGET_RENDER_FPS) {
    const startedAt = Number.isFinite(frameStartedAt) ? frameStartedAt : performance.now();
    const frameInterval = this.getRenderFrameInterval(targetFps);

    const publishedAt = this.publishFrame();
    const completedAt = Number.isFinite(publishedAt) ? publishedAt : performance.now();

    const elapsedSeconds = Math.max(0, (completedAt - startedAt) / 1000);
    const waitSeconds = Math.max(0, frameInterval - elapsedSeconds);
    return this.toSimDelay(waitSeconds);
  }

  _notifyPlanners(hookName, ...args) {
    for (const planner of this.planners.values()) {
      const hook = planner?.[hookName];
      if (typeof hook === "function") {
        hook.call(planner, this, ...args);
      }
    }
  }
}

// ══════════════════════════════════════════════
//  运行调度器（时间片调度循环）
// ══════════════════════════════════════════════

class SimulationRunner {
  constructor(engine) {
    this.engine = engine;
    this._running = false;
    this._paused = false;
    this._stepHandle = null;
    this._lastStatusRealTime = 0;
  }

  get state() {
    return this._running ? (this._paused ? "paused" : "running") : "idle";
  }

  start(params) {
    this._clearScheduledStep();
    this._running = true;
    this._paused = false;
    this._lastStatusRealTime = 0;

    this.engine.initializeRun(params);

    const restoreClock = normalizeRestoreClock(params?.runtime?.simulationTime);
    if (restoreClock > 0) {
      this._restoreToClock(restoreClock);
      this.engine.publishFrame();
      this.engine.emitStatus(`已恢复到 ${this.engine.formatClock(this.engine.clock)}，场景实体: ${this.engine.agents.size}`);
    } else {
      this.engine.emitStatus(`已初始化场景: ${this.engine.agents.size} 个实体`);
    }

    if (this._isFinished()) {
      this._finishRun();
      return;
    }

    this.engine.emitState("running");
    this._scheduleAccordingToQueue();
  }

  pause(notify = false) {
    if (!this._running || this._paused) return false;
    this._paused = true;
    this._clearScheduledStep();
    this.engine._notifyPlanners("onSimulationPause");
    if (notify) this._emitRuntimeStatus("已暂停", { force: true });
    this.engine.emitState("paused");
    return true;
  }

  resume(notify = false) {
    if (!this._running || !this._paused) return false;
    this._paused = false;
    this.engine._notifyPlanners("onSimulationResume");
    if (notify) this._emitRuntimeStatus("已继续", { force: true });
    this.engine.emitState("running");
    this._scheduleAccordingToQueue();
    return true;
  }

  stop({ emitState = true } = {}) {
    this._clearScheduledStep();
    this._running = false;
    this._paused = false;
    this._lastStatusRealTime = 0;
    if (emitState) this.engine.emitState("idle");
  }

  setSpeed(speed) {
    const previousSpeed = this.engine._speed;
    this.engine.setRunSpeed(speed);
    this.engine._notifyPlanners("onSimulationSpeedChange", previousSpeed, this.engine._speed);
    if (this._running && !this._paused) {
      this._scheduleAccordingToQueue();
    }
  }

  _emitRuntimeStatus(prefix = "仿真时钟", { force = false } = {}) {
    const now = performance.now();
    if (!force && now - this._lastStatusRealTime < STATUS_EMIT_INTERVAL_MS) return;
    this._lastStatusRealTime = now;
    this.engine.emitStatus(
      `随机数种子: ${this.engine._seed} | ${prefix}: ${this.engine.formatClock(this.engine.clock)} / ${this.engine.formatClock(this.engine._duration)}`
    );
  }

  _isFinished() {
    return Number.isFinite(this.engine._duration) && this.engine.clock >= this.engine._duration - CLOCK_EPSILON;
  }

  _getNextBoundaryClock() {
    const nextEventTime = this.engine.peekNextEventTime();
    return Number.isFinite(this.engine._duration) ? Math.min(nextEventTime, this.engine._duration) : nextEventTime;
  }

  _clearScheduledStep() {
    if (this._stepHandle != null) {
      clearTimeout(this._stepHandle);
      this._stepHandle = null;
    }
  }

  _scheduleNextWake(delayMs = 0) {
    this._clearScheduledStep();
    const boundedDelay = Math.min(Math.max(0, delayMs), 0x7fffffff);
    this._stepHandle = setTimeout(() => {
      this._stepHandle = null;
      this._step();
    }, boundedDelay);
  }

  _scheduleAccordingToQueue() {
    if (!this._running || this._paused) return;
    const nextClock = this._getNextBoundaryClock();
    if (!Number.isFinite(nextClock)) {
      this._finishRun();
      return;
    }

    if (this.engine.isUnlimitedSpeed()) {
      this._scheduleNextWake(CONTINUE_DELAY_MS);
      return;
    }

    const simDelay = Math.max(0, nextClock - this.engine.clock);
    const waitMs = simDelay <= CLOCK_EPSILON ? CONTINUE_DELAY_MS : (simDelay / this.engine._speed) * 1000;
    this._scheduleNextWake(waitMs);
  }

  _runNextEventBatch() {
    const nextClock = this._getNextBoundaryClock();
    if (!Number.isFinite(nextClock)) return false;

    if (nextClock > this.engine.clock + CLOCK_EPSILON) {
      this.engine.advanceTo(nextClock);
    } else if (nextClock + CLOCK_EPSILON < this.engine.clock) {
      return false;
    }

    if (this.engine.hasDueEvents()) {
      this.engine.dispatchDueEvents();
    }
    return true;
  }

  _restoreToClock(targetClock) {
    const bounded = this.engine.boundTargetClock(targetClock);
    if (!(bounded > this.engine.clock + CLOCK_EPSILON)) return;
    this.engine._restoring = true;
    try {
      while (this.engine.clock + CLOCK_EPSILON < bounded) {
        const nextEventTime = this.engine.peekNextEventTime();
        const nextClock = Number.isFinite(nextEventTime) ? Math.min(bounded, nextEventTime) : bounded;
        if (nextClock > this.engine.clock + CLOCK_EPSILON) {
          this.engine.advanceTo(nextClock);
        }
        if (this.engine.hasDueEvents()) {
          this.engine.dispatchDueEvents();
        }
        if (this._isFinished()) break;
        if (!Number.isFinite(nextEventTime) && nextClock >= bounded - CLOCK_EPSILON) break;
      }
    } finally {
      this.engine._restoring = false;
    }
  }

  _finishRun() {
    this._clearScheduledStep();
    this._running = false;
    this._paused = false;
    this.engine.publishFrame();
    this.engine.emitState("idle");
    self.postMessage({ type: "done" });
  }

  _step() {
    if (!this._running || this._paused) return;
    try {
      if (this.engine.isUnlimitedSpeed()) {
        const sliceStartedAt = performance.now();
        while (this._running && !this._paused) {
          const advanced = this._runNextEventBatch();
          if (!advanced) {
            this._finishRun();
            return;
          }
          this._emitRuntimeStatus();
          if (this._isFinished()) {
            this._finishRun();
            return;
          }
          if (performance.now() - sliceStartedAt >= UNLIMITED_REALTIME_BUDGET_MS) {
            this._scheduleAccordingToQueue();
            return;
          }
        }
        return;
      }

      const advanced = this._runNextEventBatch();
      if (!advanced) {
        this._finishRun();
        return;
      }
      this._emitRuntimeStatus();
      if (this._isFinished()) {
        this._finishRun();
        return;
      }
      this._scheduleAccordingToQueue();
    } catch (error) {
      this._clearScheduledStep();
      this._running = false;
      this._paused = false;
      this.engine.emitState("idle");
      self.postMessage({ type: "error", text: error?.message || String(error) });
    }
  }
}

// ══════════════════════════════════════════════
//  参数辅助函数
// ══════════════════════════════════════════════

function normalizeRunSpeed(value, fallback = 1) {
  if (value === Number.POSITIVE_INFINITY || value === -1) return Number.POSITIVE_INFINITY;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function normalizeRunDuration(value) {
  const parsed = Number(value);
  if (parsed === -1) return Number.POSITIVE_INFINITY;
  return Number.isFinite(parsed) && parsed >= 1 ? parsed : DEFAULT_RUN_DURATION;
}

function normalizeRestoreClock(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function normalizeRunSeed(value) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.trunc(parsed) : (Date.now() % 10000);
}

// ══════════════════════════════════════════════
//  全局实例与 Comlink 暴露
// ══════════════════════════════════════════════

const engine = new DESEngineHost();
const runner = new SimulationRunner(engine);

const api = {
  attachFramePort(port) {
    engine.attachFramePort(port);
  },

  async start(options = {}) {
    runner.stop({ emitState: false });
    const speed = normalizeRunSpeed(options.speed, 1);
    const duration = normalizeRunDuration(options.duration);
    const seed = normalizeRunSeed(options.seed);

    engine.reset({
      speed,
      duration,
      seed,
      sceneSettings: options.sceneSettings,
    });

    try {
      if (typeof options.sceneScript === "string" && options.sceneScript.trim().length > 0) {
        // 支持由 DXF XRECORD 提取的内嵌 JS 脚本 (动态代码执行)
        // 将脚本中的相对路径重写为指向当前 Worker 环境的绝对 URL
        let script = options.sceneScript;
        const simFrameworkUrl = new URL("./core/sim_framework.js", import.meta.url).href;
        const settingsUrl = new URL("./apps/nb1/code/settings.js", import.meta.url).href;
        script = script.replace(/from\s+["'][^"']*sim_framework\.js["']/g, `from "${simFrameworkUrl}"`);
        script = script.replace(/from\s+["'][^"']*settings\.js["']/g, `from "${settingsUrl}"`);

        const blob = new Blob([script], { type: "application/javascript" });
        const blobUrl = URL.createObjectURL(blob);
        try {
          const sceneModule = await import(blobUrl);
          const setupScene = sceneModule.setupScene || sceneModule.setupWarehouseScene;
          if (typeof setupScene === "function") {
            await setupScene(engine, options.sceneSettings);
          }
        } finally {
          URL.revokeObjectURL(blobUrl);
        }
      } else if (options.sceneModuleUrl) {
        const moduleUrl = new URL(options.sceneModuleUrl, import.meta.url).href;
        const sceneModule = await import(moduleUrl);
        const setupScene = sceneModule.setupScene || sceneModule.setupWarehouseScene;
        if (typeof setupScene === "function") {
          await setupScene(engine, options.sceneSettings);
        }
      } else {
        const defaultModuleUrl = new URL("./apps/nb1/code/scene.js", import.meta.url).href;
        const sceneModule = await import(defaultModuleUrl);
        const setupScene = sceneModule.setupScene || sceneModule.setupWarehouseScene;
        if (typeof setupScene === "function") {
          await setupScene(engine, options.sceneSettings);
        }
      }

      runner.start({ speed, duration, seed, ...options });
    } catch (err) {
      runner.stop({ emitState: false });
      engine.reset();
      engine.publishFrame();
      self.postMessage({ type: "error", text: err?.message || String(err) });
      throw err;
    }
  },

  pause() {
    return runner.pause(true);
  },

  resume() {
    return runner.resume(true);
  },

  stop() {
    runner.stop({ emitState: false });
    engine.reset();
    engine.publishFrame();
    engine.emitStatus("已停止");
    engine.emitState("idle");
  },

  setSpeed(value) {
    const nextSpeed = normalizeRunSpeed(value, engine._speed);
    runner.setSpeed(nextSpeed);
  },

  getRuntimeState() {
    return {
      state: runner.state,
      clock: engine.clock,
      duration: engine._duration,
      seed: engine._seed,
      speed: engine._speed,
    };
  },
};

// 支持通过原生 postMessage 消息控制（与 CADFlow WebAssembly 前端解耦对接）
self.addEventListener("message", async (e) => {
  const msg = e.data;
  if (!msg || typeof msg !== "object") return;
  try {
    switch (msg.type) {
      case "start":
        await api.start(msg);
        break;
      case "pause":
        api.pause();
        break;
      case "resume":
        api.resume();
        break;
      case "stop":
        api.stop();
        break;
      case "setSpeed":
        api.setSpeed(msg.speed);
        break;
    }
  } catch (err) {
    console.error("[des_worker] Error handling message:", msg.type, err);
  }
});

Comlink.expose(api);
