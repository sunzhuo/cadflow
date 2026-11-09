/**
 * sim_sdk.js —— CADFlow 离散事件仿真（DES）轻量 SDK 库
 *
 * 保持与原有 cad2sim/sim_framework.js 100% 接口兼容：
 *  - Agent       智能体基类（空间实体、有限状态机、行为队列、指令接口）
 *  - Planner     规划器基类（事件驱动、状态查询、指令派发）
 *  - Initiator   启动器（仿真初始化）
 *  - Timer       定时器（周期性自触发）
 *  - Scheduler   调度器（事件触发）
 *  - MessageBus  消息总线（信封模式）
 */

// ══════════════════════════════════════════════
//  消息总线（Message Envelope）
// ══════════════════════════════════════════════

export class MessageBus {
  constructor() {
    /** @type {Map<number, Agent|Planner>} id → 已注册对象 */
    this._registry = new Map();
    /** @type {Array} 待处理消息队列 */
    this._queue = [];
  }

  register(obj) {
    this._registry.set(obj.id, obj);
  }

  unregister(id) {
    this._registry.delete(id);
  }

  send(fromId, toId, action, payload = null) {
    const target = this._registry.get(toId);
    if (!target) return undefined;
    const envelope = { from: fromId, to: toId, action, payload };
    return target.onMessage(envelope);
  }

  enqueue(fromId, toId, action, payload = null) {
    this._queue.push({ from: fromId, to: toId, action, payload });
  }

  flush() {
    const results = [];
    while (this._queue.length > 0) {
      const env = this._queue.shift();
      const target = this._registry.get(env.to);
      if (target) {
        results.push(target.onMessage(env));
      }
    }
    return results;
  }
}

// ══════════════════════════════════════════════
//  智能体基类（Agent）
// ══════════════════════════════════════════════

export class Agent {
  /**
   * @param {number} id         引擎分配的唯一 id
   * @param {number} type       类型编号（用于渲染区分）
   * @param {object} [options]
   * @param {string} [options.modelPath]  glb/gltf 模型路径（或 2D 图元标记）
   * @param {string[]} [options.states]   状态名列表，默认 ["idle"]
   * @param {string[]} [options.commands] 可接受指令列表
   */
  constructor(id, type, options = {}) {
    this.id = id;
    this.type = type;

    // ── 空间实体位姿 ──
    this.modelPath = options.modelPath || null;
    this.x = 0;
    this.y = 0;
    this.z = 0;
    this.rotY = 0;

    // ── 有限状态 ──
    this._states = options.states || ["idle"];
    this._stateIndex = 0;
    this._commands = new Set(options.commands || []);

    // ── 指令缓冲队列 ──
    this._cmdQueue = [];
  }

  getPosition() {
    return { x: this.x, y: this.y, z: this.z };
  }
  getRotation() {
    return this.rotY;
  }
  setPosition(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
  setRotation(rotY) {
    this.rotY = rotY;
  }

  getState() {
    return this._states[this._stateIndex];
  }
  getStateIndex() {
    return this._stateIndex;
  }
  setState(name) {
    const idx = this._states.indexOf(name);
    if (idx >= 0) this._stateIndex = idx;
  }

  canAccept(cmd) {
    return this._commands.has(cmd);
  }

  acceptCommand(cmd, payload = null) {
    if (!this.canAccept(cmd)) return false;
    this._cmdQueue.push({ cmd, payload });
    return true;
  }

  nextCommand() {
    return this._cmdQueue.shift() || null;
  }

  hasCommands() {
    return this._cmdQueue.length > 0;
  }

  onMessage(envelope) {
    const { action, payload } = envelope;
    if (this.canAccept(action)) {
      this.acceptCommand(action, payload);
      return { ok: true };
    }
    if (action === "query_state") return this.getState();
    if (action === "query_position") return this.getPosition();
    return undefined;
  }

  getAuxiliaryRenderData() {
    return null;
  }

  toFrameEntity() {
    return {
      id: this.id,
      type: this.type,
      state: this._stateIndex,
      x: this.x,
      y: this.y,
      z: this.z,
      rotY: this.rotY,
    };
  }

  update(dt, engine) {
    // 由具体子类覆盖
  }
}

// ══════════════════════════════════════════════
//  规划器基类（Planner）
// ══════════════════════════════════════════════

export class Planner {
  constructor(id, name) {
    this.id = id;
    this.name = name;
    /** @type {MessageBus|null} 由引擎注入 */
    this.bus = null;
    this._subscriptions = new Set();
  }

  subscribe(eventType) {
    this._subscriptions.add(eventType);
  }

  isSubscribed(eventType) {
    return this._subscriptions.has(eventType);
  }

  queryAgentState(agentId) {
    if (!this.bus) return undefined;
    return this.bus.send(this.id, agentId, "query_state");
  }

  queryAgentPosition(agentId) {
    if (!this.bus) return undefined;
    return this.bus.send(this.id, agentId, "query_position");
  }

  commandAgent(agentId, cmd, payload = null) {
    if (!this.bus) return undefined;
    return this.bus.send(this.id, agentId, cmd, payload);
  }

  sendToPlanner(plannerId, action, payload = null) {
    if (!this.bus) return undefined;
    return this.bus.send(this.id, plannerId, action, payload);
  }

  onMessage(envelope) {
    return undefined;
  }

  onEvent(evt, engine) {}

  update(dt, engine) {}
}

// ── 启动器（Initiator）──

export class Initiator extends Planner {
  constructor(id, name = "initiator") {
    super(id, name);
  }

  onInit(engine, params) {}
}

// ── 定时器（Timer）──

export class Timer extends Planner {
  constructor(id, name, eventType, interval) {
    super(id, name);
    this.eventType = eventType;
    this.interval = interval;
    this.subscribe(eventType);
  }

  start(engine) {
    engine.scheduleEvent(this.interval, this.eventType, { timerId: this.id });
  }

  onEvent(evt, engine) {
    if (evt.type === this.eventType) {
      this.onTick(engine);
      engine.scheduleEvent(this.interval, this.eventType, { timerId: this.id });
    }
  }

  onTick(engine) {}
}

// ── 调度器（Scheduler）──

export class Scheduler extends Planner {
  constructor(id, name) {
    super(id, name);
  }

  onEvent(evt, engine) {}
}
