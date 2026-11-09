import {
  Agent,
  Initiator,
  Scheduler,
  Timer,
} from "../../../core/sim_framework.js";
import {
  DEFAULT_WAREHOUSE_SETTINGS,
  normalizeWarehouseSettings,
} from "./settings.js";

const TYPE_LOCK_ZONE = 3;
const TYPE_IGV = 4;
const TYPE_CONTAINER = 5;
const TYPE_TASK_MARKER = 6;
const TYPE_YARD_CRANE = 7;
const SCENE_ASSET_BASE_URL = "/apps/ct2/data";

const FRAME_INTERVAL = 1 / 60;
const ROAD_WIDTH = 4;
const TURN_RADIUS = 9;
const H1_BUFFER_ACCESS_CANDIDATE_LIMIT = 1;
const IGV_LENGTH = 15;
const IGV_WIDTH = 2.75;
const LOCK_AHEAD_LENGTH = IGV_LENGTH * 2;
const LOCK_ZONE_LENGTH = IGV_LENGTH * 3;
const ROAD_SECTION_LENGTH = LOCK_AHEAD_LENGTH;
const LOCK_ZONE_CENTER_OFFSET = LOCK_ZONE_LENGTH / 2 - IGV_LENGTH / 2;
const LOCK_ZONE_HEAD_OFFSET = LOCK_ZONE_LENGTH - IGV_LENGTH / 2;
const LOCK_ZONE_TAIL_OFFSET = IGV_LENGTH / 2;
const LOCK_ZONE_RENDER_Y = 0.04;
const LOCK_ZONE_SLICE_LENGTH = 2;
const EPSILON = 1e-6;
const ROUTE_SEARCH_STEP = 0.25;
const HEADING_ALIGNMENT_THRESHOLD = 0.95;
const RESOURCE_EVENT_TYPE = "evt_ct2_resource_transition";
const INTERSECTION_TRAJECTORY_SAMPLE_STEP = 2.0;
const ARC_SWEEP_SAMPLE_STEP = 2.0;
const T_JUNCTION_CELL_SAMPLE_STEP = 2.0;
const INTERSECTION_CELL_SIZE = ROAD_WIDTH * 1.5;
const BIG_INTERSECTION_MAX_CONCURRENT = 6;
const SMALL_INTERSECTION_MAX_CONCURRENT = 3;
const RESOURCE_ENTER_REACTION_BUFFER = 2.0;
const MAX_RESOURCE_ENTER_LOOKAHEAD = 16.0;
const INTERSECTION_EXIT_SEGMENT_LIMIT = 1;
const EXIT_SEGMENT_RESERVATION_TTL = 99999;
const ROAD_SEGMENT_QUEUE_STARVATION_SECONDS = 99999;
const ROAD_SEGMENT_SMART_WAKE_ENABLED = true;
const EXIT_SEGMENT_RESERVATION_ENABLED = true;
const IDLE_REPOSITION_TO_DEPOT_ENABLED = false;
const IDLE_REPOSITION_RETRY_SECONDS = 2.0;
const IDLE_REPOSITION_CANDIDATE_LIMIT = 24;
const DEPOT_LANE_COUNT = 0;
const DEPOT_MIN_X = -1104;
const DEPOT_LANE_SPACING = 20;
const DEPOT_LANE_MIN_Z = 108;
const DEPOT_ACCESS_Z = 126;
const DEPOT_PARK_Z = 196;
const DEPOT_MAX_Z = 224;
const VIRTUAL_DEPOT_ENABLED = false;
const VIRTUAL_DEPOT_GATE_X = -430;
const VIRTUAL_DEPOT_HIDDEN_POINT = Object.freeze({
  x: -2000,
  z: 2000,
});
const VIRTUAL_DEPOT_GATE_DEFINITIONS = Object.freeze([]);
const COMMITTED_VEHICLE_PRIORITY_ENABLED = true;
const COMMITTED_TURN_CLEARANCE_PRIORITY = -10;
const MAINLINE_THROUGH_PRIORITY = 0;
const MAINLINE_TO_BUFFER_PRIORITY = 1;
const BUFFER_TO_MAINLINE_PRIORITY = 2;
const DEFAULT_INTERSECTION_PRIORITY = 1;
const SMALL_INTERSECTION_BUFFER_STARVATION_SECONDS = 8.0;
const COMMITTED_TURN_LOOKAHEAD_DISTANCE = IGV_LENGTH;
const RESOURCE_ACTION_PRIORITY = Object.freeze({
  leave: 0,
  enter: 1,
  finish: 2,
});

function getEngineClock(engine) {
  return Number.isFinite(engine?.clock) ? engine.clock : 0;
}
const CONTAINER_CARRY_HEIGHT = 0.12;
const CONTAINER_LENGTH = 12.192;
const CONTAINER_WIDTH = 2.591;
const TASK_PAIR_SAMPLE_ATTEMPTS = 64;
const INITIAL_IGV_COUNT = 32;
const READY_IGV_QUEUE_MAX_SIZE = 10;
const READY_IGV_QUEUE_MAX_WAIT_SECONDS = 30;
const MAX_PREFETCHED_USER_TASKS_PER_IGV = 2;
const ENABLE_RUNNING_TASK_PREFETCH = true;
const ENABLE_READY_QUEUE_BEFORE_DEPOT = true;
const VIRTUAL_DEPOT_RESERVE_COUNT = 0;
const MAX_TOTAL_IGV_COUNT = INITIAL_IGV_COUNT;
const ROUTE_RETRY_BACKOFF_SECONDS = 1.5;
const MAX_TRANSIT_ROUTE_RETRY_ATTEMPTS = 12;
const TASK_START_MARKER_COLOR = 0x4d97ff;
const TASK_END_MARKER_COLOR = 0xe85d5d;
const HORIZONTAL_MIN_X = -460;
const HORIZONTAL_MAX_X = 465;
const VERTICAL_MIN_Z = -290;
const VERTICAL_MAX_Z = 290;
const BUFFER_MIN_Z = -267.05;
const BUFFER_MAX_Z = -243.05;
const BUFFER_CENTER_Z = (BUFFER_MIN_Z + BUFFER_MAX_Z) / 2;
const BUFFER_SLOT_XS = Object.freeze(
  Array.from({ length: 124 }, (_, index) => Math.round((-447.9 + index * 7.0) * 100) / 100)
);
const BUFFER_SLOT_CENTER_XS = BUFFER_SLOT_XS;

const BOX_X_RANGES = Object.freeze([
  Object.freeze({ key: "l", minX: -420, maxX: -30, centerX: -225 }),
  Object.freeze({ key: "r", minX: 30, maxX: 420, centerX: 225 }),
]);
const BOX_CENTER_XS = Object.freeze(BOX_X_RANGES.map((range) => range.centerX));

export const SCENE_STATIC_SCENE_URL = new URL("../docs/scene.dxf", import.meta.url).href;
export const SCENE_ASSET_TEMPLATES = Object.freeze({});
export const SCENE_STATE_STYLE_MAPPING = Object.freeze({
  [TYPE_LOCK_ZONE]: Object.freeze({
    0: Object.freeze({ color: 0x66bb6a, emissive: 0x113311, opacity: 0.35 }),
    1: Object.freeze({ color: 0x66bb6a, emissive: 0x113311, opacity: 0.35 }),
  }),
  [TYPE_IGV]: Object.freeze({
    0: Object.freeze({ color: 0x66bb6a, emissive: 0x113311 }),
    1: Object.freeze({ color: 0x66bb6a, emissive: 0x113311 }),
    2: Object.freeze({ color: 0x66bb6a, emissive: 0x113311 }),
    3: Object.freeze({ color: 0x66bb6a, emissive: 0x113311 }),
    4: Object.freeze({ color: 0x66bb6a, emissive: 0x113311 }),
  }),
  [TYPE_CONTAINER]: Object.freeze({
    0: Object.freeze({ color: 0xcf6b52, emissive: 0x4a2017 }),
    1: Object.freeze({ color: 0xd4a24c, emissive: 0x47320f }),
    2: Object.freeze({ color: 0x5da0d1, emissive: 0x17304b }),
    3: Object.freeze({ color: 0x72b56d, emissive: 0x183417 }),
    4: Object.freeze({ color: 0xa37bc7, emissive: 0x2f1c47 }),
    5: Object.freeze({ color: 0xc98b99, emissive: 0x41202a }),
  }),
  [TYPE_TASK_MARKER]: Object.freeze({
    0: Object.freeze({ color: 0x4d97ff, opacity: 1.0 }),
    1: Object.freeze({ color: 0xe85d5d, opacity: 1.0 }),
  }),
  [TYPE_YARD_CRANE]: Object.freeze({
    0: Object.freeze({ color: 0xe6a23c, opacity: 0.85 }),
    1: Object.freeze({ color: 0xf0b24d, opacity: 0.85 }),
    2: Object.freeze({ color: 0xf0b24d, opacity: 0.85 }),
  }),
});
export const SCENE_RENDER_CONFIG = Object.freeze({
  staticSceneUrl: SCENE_STATIC_SCENE_URL,
  assetTemplates: SCENE_ASSET_TEMPLATES,
  stateColors: SCENE_STATE_STYLE_MAPPING,
  lighting: Object.freeze({
    ambientIntensity: 1.05,
    directionalIntensity: 2.25,
  }),
});

const S_IDLE = "idle";
const S_QUEUED = "queued";
const S_WAIT = "wait";
const S_MOVE_EMPTY = "move_empty";
const S_MOVE_LOADED = "move_loaded";
const DIR_EAST = Object.freeze({ dx: 1, dz: 0, label: "east" });
const DIR_WEST = Object.freeze({ dx: -1, dz: 0, label: "west" });
const DIR_SOUTH = Object.freeze({ dx: 0, dz: 1, label: "south" });
const DIR_NORTH = Object.freeze({ dx: 0, dz: -1, label: "north" });

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function clonePoint(point) {
  return { x: point.x, z: point.z };
}

function distance2D(a, b) {
  return Math.hypot(b.x - a.x, b.z - a.z);
}

function normalize2D(dx, dz) {
  const length = Math.hypot(dx, dz);
  if (length <= EPSILON) {
    return { x: 1, z: 0 };
  }
  return { x: dx / length, z: dz / length };
}

function dot2D(left, right) {
  return left.x * right.x + left.z * right.z;
}

function cross2D(left, right) {
  return left.x * right.z - left.z * right.x;
}

function vehicleRotationFromDirection(dx, dz) {
  return Math.atan2(-dz, dx);
}

function stripRotationFromDirection(dx, dz) {
  return Math.atan2(-dz, dx);
}

function leftNormal(direction) {
  return { x: -direction.z, z: direction.x };
}

function rightNormal(direction) {
  return { x: direction.z, z: -direction.x };
}

function vectorFromDirection(direction) {
  return { x: direction.dx, z: direction.dz };
}

function samePoint(left, right, epsilon = 1e-4) {
  return (
    Math.abs(left.x - right.x) <= epsilon &&
    Math.abs(left.z - right.z) <= epsilon
  );
}

function createLinearSegment(kind, start, end, maxSpeed, metadata = null) {
  const length = distance2D(start, end);
  if (length <= EPSILON) {
    return null;
  }

  const direction = normalize2D(end.x - start.x, end.z - start.z);
  const metadataFields =
    metadata && typeof metadata === "object" ? metadata : {};
  return Object.freeze({
    ...metadataFields,
    kind,
    geometry: "line",
    start: clonePoint(start),
    end: clonePoint(end),
    length,
    maxSpeed,
    direction,
    rotY: vehicleRotationFromDirection(direction.x, direction.z),
  });
}

function createArcSegment(
  kind,
  start,
  end,
  center,
  radius,
  clockwise,
  maxSpeed,
  metadata = null,
) {
  if (radius <= EPSILON) {
    return null;
  }

  const startAngle = Math.atan2(start.z - center.z, start.x - center.x);
  const deltaAngle = clockwise ? -Math.PI / 2 : Math.PI / 2;
  const length = Math.abs(deltaAngle) * radius;
  if (length <= EPSILON) {
    return null;
  }

  const startDirection = clockwise
    ? { x: Math.sin(startAngle), z: -Math.cos(startAngle) }
    : { x: -Math.sin(startAngle), z: Math.cos(startAngle) };
  const metadataFields =
    metadata && typeof metadata === "object" ? metadata : {};

  return Object.freeze({
    ...metadataFields,
    kind,
    geometry: "arc",
    start: clonePoint(start),
    end: clonePoint(end),
    center: clonePoint(center),
    radius,
    clockwise,
    startAngle,
    deltaAngle,
    length,
    maxSpeed,
    direction: startDirection,
    rotY: vehicleRotationFromDirection(startDirection.x, startDirection.z),
  });
}

function directionOnSegment(segment, distanceOnSegment) {
  if (!segment) {
    return { x: 1, z: 0 };
  }

  if (segment.geometry !== "arc") {
    return segment.direction;
  }

  const ratio = clamp(distanceOnSegment / segment.length, 0, 1);
  const angle = segment.startAngle + segment.deltaAngle * ratio;
  return segment.clockwise
    ? { x: Math.sin(angle), z: -Math.cos(angle) }
    : { x: -Math.sin(angle), z: Math.cos(angle) };
}

function pointOnSegment(segment, distanceOnSegment) {
  if (segment.geometry === "arc") {
    const ratio = clamp(distanceOnSegment / segment.length, 0, 1);
    const angle = segment.startAngle + segment.deltaAngle * ratio;
    return {
      x: segment.center.x + Math.cos(angle) * segment.radius,
      z: segment.center.z + Math.sin(angle) * segment.radius,
    };
  }

  const ratio = clamp(distanceOnSegment / segment.length, 0, 1);
  return {
    x: segment.start.x + (segment.end.x - segment.start.x) * ratio,
    z: segment.start.z + (segment.end.z - segment.start.z) * ratio,
  };
}

function finalizeRoute(startKey, endKey, segments) {
  let totalLength = 0;
  const annotatedSegments = segments.filter(Boolean).map((segment) => {
    const startDistance = totalLength;
    totalLength += segment.length;
    return Object.freeze({
      ...segment,
      startDistance,
      endDistance: totalLength,
    });
  });

  return Object.freeze({
    startKey,
    endKey,
    totalLength,
    segments: Object.freeze(annotatedSegments),
  });
}

function createRect(center, forward, length, width = ROAD_WIDTH) {
  const normalizedForward = normalize2D(forward.x, forward.z);
  const right = { x: -normalizedForward.z, z: normalizedForward.x };
  return {
    center: clonePoint(center),
    forward: normalizedForward,
    right,
    halfLength: length / 2,
    halfWidth: width / 2,
    rotY: stripRotationFromDirection(normalizedForward.x, normalizedForward.z),
    head: {
      x: center.x + normalizedForward.x * (length / 2),
      z: center.z + normalizedForward.z * (length / 2),
    },
  };
}

function createRectFromEndpoints(start, end, width = ROAD_WIDTH) {
  const length = distance2D(start, end);
  if (length <= EPSILON) {
    return null;
  }

  const forward = normalize2D(end.x - start.x, end.z - start.z);
  const center = {
    x: (start.x + end.x) / 2,
    z: (start.z + end.z) / 2,
  };
  return createRect(center, forward, length, width);
}

function rectEdgePoints(rect) {
  const start = {
    x: rect.center.x - rect.forward.x * rect.halfLength,
    z: rect.center.z - rect.forward.z * rect.halfLength,
  };
  const end = {
    x: rect.center.x + rect.forward.x * rect.halfLength,
    z: rect.center.z + rect.forward.z * rect.halfLength,
  };
  return {
    leftStart: {
      x: start.x - rect.right.x * rect.halfWidth,
      z: start.z - rect.right.z * rect.halfWidth,
    },
    leftEnd: {
      x: end.x - rect.right.x * rect.halfWidth,
      z: end.z - rect.right.z * rect.halfWidth,
    },
    rightStart: {
      x: start.x + rect.right.x * rect.halfWidth,
      z: start.z + rect.right.z * rect.halfWidth,
    },
    rightEnd: {
      x: end.x + rect.right.x * rect.halfWidth,
      z: end.z + rect.right.z * rect.halfWidth,
    },
  };
}

function pushUniquePoint(points, point) {
  if (!point) {
    return;
  }
  if (points.length > 0 && samePoint(points[points.length - 1], point)) {
    return;
  }
  points.push(clonePoint(point));
}

function buildZonePolygonFromSlices(slices) {
  if (!Array.isArray(slices) || slices.length === 0) {
    return [];
  }

  const leftBoundary = [];
  const rightBoundary = [];
  for (const slice of slices) {
    const edges = rectEdgePoints(slice);
    pushUniquePoint(leftBoundary, edges.leftStart);
    pushUniquePoint(leftBoundary, edges.leftEnd);
    pushUniquePoint(rightBoundary, edges.rightStart);
    pushUniquePoint(rightBoundary, edges.rightEnd);
  }

  const polygon = [];
  for (const point of leftBoundary) {
    pushUniquePoint(polygon, point);
  }
  for (let index = rightBoundary.length - 1; index >= 0; index -= 1) {
    pushUniquePoint(polygon, rightBoundary[index]);
  }
  if (
    polygon.length > 1 &&
    samePoint(polygon[0], polygon[polygon.length - 1])
  ) {
    polygon.pop();
  }
  return polygon.length >= 3 ? polygon : [];
}

function createLockZoneRect(center, forward) {
  return createRect(center, forward, LOCK_ZONE_LENGTH, ROAD_WIDTH);
}

function projectRectOnAxis(rect, axis) {
  const centerProjection = dot2D(rect.center, axis);
  const radius =
    rect.halfLength * Math.abs(dot2D(rect.forward, axis)) +
    rect.halfWidth * Math.abs(dot2D(rect.right, axis));
  return {
    min: centerProjection - radius,
    max: centerProjection + radius,
  };
}

function intervalsOverlap(left, right) {
  return left.min <= right.max + EPSILON && right.min <= left.max + EPSILON;
}

function rectsOverlap(left, right) {
  if (!left || !right) {
    return false;
  }

  const axes = [left.forward, left.right, right.forward, right.right];
  return axes.every((axis) =>
    intervalsOverlap(
      projectRectOnAxis(left, axis),
      projectRectOnAxis(right, axis),
    ),
  );
}

function routePoseAtDistance(route, distance) {
  if (!route?.segments?.length) {
    return null;
  }

  const clampedDistance = clamp(distance, 0, route.totalLength);
  let segment = route.segments[route.segments.length - 1];
  for (const candidate of route.segments) {
    if (clampedDistance <= candidate.endDistance + EPSILON) {
      segment = candidate;
      break;
    }
  }

  const localDistance = clamp(
    clampedDistance - segment.startDistance,
    0,
    segment.length,
  );
  return {
    routeDistance: clampedDistance,
    segment,
    point: pointOnSegment(segment, localDistance),
    direction: directionOnSegment(segment, localDistance),
  };
}

function locateRouteSegment(route, distance) {
  if (!route?.segments?.length) {
    return null;
  }

  const clampedDistance = clamp(distance, 0, route.totalLength);
  for (let index = 0; index < route.segments.length; index += 1) {
    const segment = route.segments[index];
    if (clampedDistance <= segment.endDistance + EPSILON) {
      return {
        index,
        segment,
        distanceOnSegment: clamp(
          clampedDistance - segment.startDistance,
          0,
          segment.length,
        ),
      };
    }
  }

  const index = route.segments.length - 1;
  const segment = route.segments[index];
  return {
    index,
    segment,
    distanceOnSegment: segment.length,
  };
}

function pointInIntersectionArea(point, intersectionArea) {
  if (!point || !intersectionArea) {
    return false;
  }

  const halfWidthX = Number.isFinite(intersectionArea.halfWidthX)
    ? intersectionArea.halfWidthX
    : intersectionArea.halfWidth;
  const halfWidthZ = Number.isFinite(intersectionArea.halfWidthZ)
    ? intersectionArea.halfWidthZ
    : intersectionArea.halfWidth;

  return (
    Math.abs(point.x - intersectionArea.x) <= halfWidthX + EPSILON &&
    Math.abs(point.z - intersectionArea.z) <= halfWidthZ + EPSILON
  );
}

function buildLockZoneSlices(route, startDistance, endDistance) {
  if (!route?.segments?.length) {
    return [];
  }

  const clampedStart = clamp(startDistance, 0, route.totalLength);
  const clampedEnd = clamp(endDistance, clampedStart, route.totalLength);
  const slices = [];

  for (const segment of route.segments) {
    if (
      segment.endDistance <= clampedStart + EPSILON ||
      segment.startDistance >= clampedEnd - EPSILON
    ) {
      continue;
    }

    const localStart = Math.max(0, clampedStart - segment.startDistance);
    const localEnd = Math.min(
      segment.length,
      clampedEnd - segment.startDistance,
    );
    const visibleLength = localEnd - localStart;
    if (visibleLength <= EPSILON) {
      continue;
    }

    const partCount = Math.max(
      1,
      Math.ceil(visibleLength / LOCK_ZONE_SLICE_LENGTH),
    );
    for (let index = 0; index < partCount; index += 1) {
      const partStart = localStart + (visibleLength * index) / partCount;
      const partEnd = localStart + (visibleLength * (index + 1)) / partCount;
      const startPoint = pointOnSegment(segment, partStart);
      const endPoint = pointOnSegment(segment, partEnd);
      const slice = createRectFromEndpoints(startPoint, endPoint);
      if (slice) {
        slices.push(slice);
      }
    }
  }

  return slices;
}

function buildFallbackLockZoneSlices(fallbackPose) {
  if (!fallbackPose?.point || !fallbackPose?.direction) {
    return [];
  }

  const tail = {
    x: fallbackPose.point.x - fallbackPose.direction.x * (IGV_LENGTH / 2),
    z: fallbackPose.point.z - fallbackPose.direction.z * (IGV_LENGTH / 2),
  };
  const head = {
    x: tail.x + fallbackPose.direction.x * LOCK_ZONE_LENGTH,
    z: tail.z + fallbackPose.direction.z * LOCK_ZONE_LENGTH,
  };

  const slices = [];
  const partCount = Math.max(
    1,
    Math.ceil(LOCK_ZONE_LENGTH / LOCK_ZONE_SLICE_LENGTH),
  );
  for (let index = 0; index < partCount; index += 1) {
    const startRatio = index / partCount;
    const endRatio = (index + 1) / partCount;
    const startPoint = {
      x: tail.x + (head.x - tail.x) * startRatio,
      z: tail.z + (head.z - tail.z) * startRatio,
    };
    const endPoint = {
      x: tail.x + (head.x - tail.x) * endRatio,
      z: tail.z + (head.z - tail.z) * endRatio,
    };
    const slice = createRectFromEndpoints(startPoint, endPoint);
    if (slice) {
      slices.push(slice);
    }
  }
  return slices;
}

function composeLockZoneFromSlices(slices) {
  if (!Array.isArray(slices) || slices.length === 0) {
    return null;
  }

  const polygon = buildZonePolygonFromSlices(slices);
  let weightedX = 0;
  let weightedZ = 0;
  let weightedLength = 0;

  for (const slice of slices) {
    const length = slice.halfLength * 2;
    weightedX += slice.center.x * length;
    weightedZ += slice.center.z * length;
    weightedLength += length;
  }

  return {
    center: {
      x:
        weightedLength > EPSILON
          ? weightedX / weightedLength
          : slices[0].center.x,
      z:
        weightedLength > EPSILON
          ? weightedZ / weightedLength
          : slices[0].center.z,
    },
    rotY: slices[0].rotY,
    head: clonePoint(slices[slices.length - 1].head),
    polygon,
    slices,
  };
}

function buildLockZone(route, travelDistance, fallbackPose = null) {
  if (route?.segments?.length) {
    const tailDistance = Math.max(0, travelDistance - IGV_LENGTH / 2);
    const headDistance = clamp(
      tailDistance + LOCK_ZONE_LENGTH,
      tailDistance,
      route.totalLength,
    );
    const slices = buildLockZoneSlices(route, tailDistance, headDistance);
    const zone = composeLockZoneFromSlices(slices);
    if (zone) {
      return zone;
    }
  }

  if (!fallbackPose) {
    return null;
  }

  return composeLockZoneFromSlices(buildFallbackLockZoneSlices(fallbackPose));
}

function createHorizontalLane(id, z, direction, role, row) {
  return Object.freeze({
    id,
    row,
    role,
    orientation: "horizontal",
    center: z,
    direction,
    minX: HORIZONTAL_MIN_X,
    maxX: HORIZONTAL_MAX_X,
    start: Object.freeze({
      x: direction.dx > 0 ? HORIZONTAL_MIN_X : HORIZONTAL_MAX_X,
      z,
    }),
    end: Object.freeze({
      x: direction.dx > 0 ? HORIZONTAL_MAX_X : HORIZONTAL_MIN_X,
      z,
    }),
  });
}

function createBoundedHorizontalLane(id, z, direction, role, row, minX, maxX) {
  return Object.freeze({
    id,
    row,
    role,
    orientation: "horizontal",
    center: z,
    direction,
    minX,
    maxX,
    start: Object.freeze({
      x: direction.dx > 0 ? minX : maxX,
      z,
    }),
    end: Object.freeze({
      x: direction.dx > 0 ? maxX : minX,
      z,
    }),
  });
}

function createVerticalLane(
  id,
  x,
  direction,
  role,
  row,
  minZ = VERTICAL_MIN_Z,
  maxZ = VERTICAL_MAX_Z,
) {
  return Object.freeze({
    id,
    row,
    role,
    orientation: "vertical",
    center: x,
    direction,
    minZ,
    maxZ,
    start: Object.freeze({
      x,
      z: direction.dz > 0 ? minZ : maxZ,
    }),
    end: Object.freeze({
      x,
      z: direction.dz > 0 ? maxZ : minZ,
    }),
  });
}

// Buffer slots lanes (bidirectional vertical lanes)
const BUFFER_LANES = Object.freeze(
  BUFFER_SLOT_XS.map((x, index) =>
    createVerticalLane(
      `buf_${index + 1}`,
      x,
      DIR_SOUTH,
      "buffer",
      "u",
      BUFFER_MIN_Z,
      BUFFER_MAX_Z
    )
  )
);

// QC Shoreline (Z ~ -289.33 to -273.19)
const QC_TRANSIT_LANES = Object.freeze([
  createHorizontalLane("qc_t1", -289.33, DIR_WEST, "transit", "qc"),
  createHorizontalLane("qc_t2", -273.19, DIR_WEST, "transit", "qc"),
]);
const QC_WORK_LANES = Object.freeze([
  createHorizontalLane("qc_w1", -278.16, DIR_WEST, "work", "qc"),
]);

// Buffer top & bottom connectors
const BUFFER_CONNECTORS = Object.freeze([
  createHorizontalLane("buf_top", BUFFER_MIN_Z, DIR_WEST, "transit", "buf_conn"),
  createHorizontalLane("buf_bot", BUFFER_MAX_Z, DIR_EAST, "transit", "buf_conn"),
]);

// Sub-buffer lanes (Z = -240 to -221)
const SUB_BUFFER_LANES = Object.freeze([
  createHorizontalLane("sb1", -239.87, DIR_WEST, "transit", "sub_buf"),
  createHorizontalLane("sb2", -237.00, DIR_EAST, "transit", "sub_buf"),
  createHorizontalLane("sb3", -233.54, DIR_WEST, "transit", "sub_buf"),
  createHorizontalLane("sb4", -229.63, DIR_EAST, "transit", "sub_buf"),
  createHorizontalLane("sb5", -225.65, DIR_WEST, "transit", "sub_buf"),
  createHorizontalLane("sb6", -221.74, DIR_EAST, "transit", "sub_buf"),
]);

// Yard Corridors (Zones 1-10)
const YARD_CORRIDORS = Object.freeze([
  { row: "y1", z: -173.6, workId: "y12_w", transitId: "y12_t", dir: DIR_EAST },
  { row: "y2", z: -118.9, workId: "y23_w", transitId: "y23_t", dir: DIR_EAST },
  { row: "y3", z: -64.4, workId: "y34_w", transitId: "y34_t", dir: DIR_EAST },
  { row: "y4", z: -10.0, workId: "y45_w", transitId: "y45_t", dir: DIR_EAST },
  { row: "y5", z: 45.0, workId: "y56_w", transitId: "y56_t", dir: DIR_EAST },
  { row: "y6", z: 118.0, workId: "y67_w", transitId: "y67_t", dir: DIR_EAST },
  { row: "y7", z: 172.0, workId: "y78_w", transitId: "y78_t", dir: DIR_EAST },
  { row: "y8", z: 227.0, workId: "y89_w", transitId: "y89_t", dir: DIR_EAST },
  { row: "y9", z: 281.0, workId: "y910_w", transitId: "y910_t", dir: DIR_EAST },
]);

const YARD_TRANSIT_LANES = Object.freeze(
  YARD_CORRIDORS.map(c => createHorizontalLane(c.transitId, c.z, c.dir, "transit", c.row))
);
const YARD_WORK_LANES = Object.freeze(
  YARD_CORRIDORS.map(c => createHorizontalLane(c.workId, c.z - 4, c.dir, "work", c.row))
);

// Vertical Transit Corridors
const TRANSIT_VERTICAL_LANES = Object.freeze([
  createVerticalLane("v_west_s", -448.5, DIR_SOUTH, "transit", "v_west", -289.33, 281.0),
  createVerticalLane("v_midw_n", -198.9, DIR_NORTH, "transit", "v_midw", -289.33, 281.0),
  createVerticalLane("v_midw_s", -189.4, DIR_SOUTH, "transit", "v_midw", -289.33, 281.0),
  createVerticalLane("v_mide_n", 250.8, DIR_NORTH, "transit", "v_mide", -289.33, 281.0),
  createVerticalLane("v_mide_s", 258.9, DIR_SOUTH, "transit", "v_mide", -289.33, 281.0),
  createVerticalLane("v_east_n", 462.4, DIR_NORTH, "transit", "v_east", -289.33, 281.0),
]);

const TRANSIT_HORIZONTAL_LANES = Object.freeze([
  ...QC_TRANSIT_LANES,
  ...BUFFER_CONNECTORS,
  ...SUB_BUFFER_LANES,
  ...YARD_TRANSIT_LANES,
]);

const WORK_LANES = Object.freeze([
  ...QC_WORK_LANES,
  ...YARD_WORK_LANES,
]);

const TRANSIT_LANES = Object.freeze([
  ...TRANSIT_HORIZONTAL_LANES,
  ...TRANSIT_VERTICAL_LANES,
  ...BUFFER_LANES,
]);

const ALL_LANES = Object.freeze([...TRANSIT_LANES, ...WORK_LANES]);
const LANE_BY_ID = new Map(ALL_LANES.map((lane) => [lane.id, lane]));
const ROAD_BY_ID = LANE_BY_ID;

const YARD_BLOCK_DEFINITIONS = Object.freeze([
  { row: 1, name: "YC0_1", isYC0: true, cz: -198.0, minX: -442, maxX: 450, workLaneIds: ["y12_w"] },
  { row: 2, name: "YC_2", isYC0: false, cz: -146.0, minX: -442, maxX: 450, workLaneIds: ["y12_w", "y23_w"] },
  { row: 3, name: "YC_3", isYC0: false, cz: -91.0, minX: -442, maxX: 450, workLaneIds: ["y23_w", "y34_w"] },
  { row: 4, name: "YC_4", isYC0: false, cz: -37.0, minX: -442, maxX: 450, workLaneIds: ["y34_w", "y45_w"] },
  { row: 5, name: "YC_5", isYC0: false, cz: 17.0, minX: -442, maxX: 450, workLaneIds: ["y45_w", "y56_w"] },
  { row: 6, name: "YC_6", isYC0: false, cz: 90.0, minX: -560, maxX: 225, workLaneIds: ["y56_w", "y67_w"] },
  { row: 7, name: "YC_7", isYC0: false, cz: 145.0, minX: -560, maxX: 225, workLaneIds: ["y67_w", "y78_w"] },
  { row: 8, name: "YC_8", isYC0: false, cz: 199.0, minX: -560, maxX: 225, workLaneIds: ["y78_w", "y89_w"] },
  { row: 9, name: "YC_9", isYC0: false, cz: 254.0, minX: -481, maxX: 205, workLaneIds: ["y89_w", "y910_w"] },
  { row: 10, name: "YC_10", isYC0: false, cz: 308.0, minX: -481, maxX: 225, workLaneIds: ["y910_w"] },
]);

const QC_TASK_LANES = Object.freeze([
  Object.freeze({ laneId: "qc_w1", transitLaneId: "qc_t2", row: "qc", z: -278.16 }),
]);
const YARD_TASK_LANES = Object.freeze(
  YARD_CORRIDORS.map(c => Object.freeze({ laneId: c.workId, transitLaneId: c.transitId, row: c.row, z: c.z - 4 }))
);
const TASK_WORK_LANES = Object.freeze([
  ...QC_TASK_LANES,
  ...YARD_TASK_LANES,
]);
const H1_TASK_LANES = QC_TASK_LANES;

const WORK_POINT_DEFINITIONS = Object.freeze(
  TASK_WORK_LANES.flatMap(lane =>
    BOX_X_RANGES.map(box => Object.freeze({
      id: `${lane.laneId}-${box.key}`,
      laneId: lane.laneId,
      transitLaneId: lane.transitLaneId,
      x: box.centerX,
    }))
  )
);

const WORK_POINTS = Object.freeze(
  WORK_POINT_DEFINITIONS.map((definition) => {
    const lane = LANE_BY_ID.get(definition.laneId);
    return Object.freeze({
      ...definition,
      row: lane.row,
      point: Object.freeze({ x: definition.x, z: lane.center }),
    });
  }),
);
const WORK_POINT_BY_ID = new Map(WORK_POINTS.map((point) => [point.id, point]));
const H1_TASK_POINTS = Object.freeze(
  WORK_POINTS.filter((point) => point.row === "qc"),
);
const YARD_TASK_POINTS = Object.freeze(
  WORK_POINTS.filter((point) => point.row !== "qc"),
);
function findBoxRangeForX(x) {
  if (x <= BOX_X_RANGES[0].maxX) {
    return BOX_X_RANGES[0];
  }
  if (x >= BOX_X_RANGES[1].minX) {
    return BOX_X_RANGES[1];
  }
  return Math.abs(x - BOX_X_RANGES[0].centerX) <=
    Math.abs(x - BOX_X_RANGES[1].centerX)
    ? BOX_X_RANGES[0]
    : BOX_X_RANGES[1];
}

function createTaskPoint(laneDefinition, x, boxRange = findBoxRangeForX(x)) {
  if (!laneDefinition || !boxRange) {
    return null;
  }

  const pointX = clamp(x, boxRange.minX, boxRange.maxX);
  return Object.freeze({
    kind: "work",
    id: `${laneDefinition.laneId}-${boxRange.key}-${pointX.toFixed(3)}`,
    laneId: laneDefinition.laneId,
    transitLaneId: laneDefinition.transitLaneId,
    row: laneDefinition.row,
    point: Object.freeze({ x: pointX, z: laneDefinition.z }),
  });
}

const QC_CRANE_LOCATIONS = Object.freeze([
  Object.freeze({ id: "QC_0", centerX: -351.56, minX: -363.13, maxX: -339.99 }),
  Object.freeze({ id: "QC_1", centerX: -276.56, minX: -288.13, maxX: -264.99 }),
  Object.freeze({ id: "QC_2", centerX: -201.56, minX: -213.13, maxX: -189.99 }),
  Object.freeze({ id: "QC_3", centerX: -126.56, minX: -138.13, maxX: -114.99 }),
  Object.freeze({ id: "QC_4", centerX: 120.32, minX: 108.75, maxX: 131.90 }),
  Object.freeze({ id: "QC_5", centerX: 195.32, minX: 183.75, maxX: 206.90 }),
  Object.freeze({ id: "QC_6", centerX: 270.32, minX: 258.75, maxX: 281.90 }),
  Object.freeze({ id: "QC_7", centerX: 345.32, minX: 333.75, maxX: 356.90 }),
]);

export const YARD_CRANE_DEFINITIONS = Object.freeze([
  Object.freeze({ index: 0, row: 0, isYC0: true, name: "YC0-1", x: -300.38, z: -197.05, minX: -439.7, maxX: -225.4 }),
  Object.freeze({ index: 1, row: 0, isYC0: true, name: "YC0-2", x: 111.26, z: -197.05, minX: -163.4, maxX: 224.6 }),
  Object.freeze({ index: 2, row: 0, isYC0: true, name: "YC0-3", x: 420.24, z: -197.05, minX: 286.6, maxX: 447.8 }),
  Object.freeze({ index: 3, row: 1, isYC0: false, name: "YC-1-1", x: -300.34, z: -146.26, minX: -439.1, maxX: -251.7 }),
  Object.freeze({ index: 4, row: 1, isYC0: false, name: "YC-1-2", x: -84.01, z: -146.26, minX: -163.4, maxX: 13.6 }),
  Object.freeze({ index: 5, row: 1, isYC0: false, name: "YC-1-3", x: 111.30, z: -146.26, minX: 13.6, maxX: 198.3 }),
  Object.freeze({ index: 6, row: 1, isYC0: false, name: "YC-1-4", x: 420.18, z: -146.26, minX: 286.6, maxX: 447.8 }),
  Object.freeze({ index: 7, row: 2, isYC0: false, name: "YC-2-1", x: -300.34, z: -91.76, minX: -439.1, maxX: -232.0 }),
  Object.freeze({ index: 8, row: 2, isYC0: false, name: "YC-2-2", x: 111.30, z: -91.76, minX: -156.9, maxX: 218.0 }),
  Object.freeze({ index: 9, row: 2, isYC0: false, name: "YC-2-3", x: 420.18, z: -91.76, minX: 293.1, maxX: 447.8 }),
  Object.freeze({ index: 10, row: 3, isYC0: false, name: "YC-3-1", x: -300.34, z: -37.26, minX: -439.1, maxX: -232.0 }),
  Object.freeze({ index: 11, row: 3, isYC0: false, name: "YC-3-2", x: -84.01, z: -37.26, minX: -156.9, maxX: 13.6 }),
  Object.freeze({ index: 12, row: 3, isYC0: false, name: "YC-3-3", x: 111.30, z: -37.26, minX: 13.6, maxX: 218.0 }),
  Object.freeze({ index: 13, row: 3, isYC0: false, name: "YC-3-4", x: 420.18, z: -37.26, minX: 293.1, maxX: 447.8 }),
  Object.freeze({ index: 14, row: 4, isYC0: false, name: "YC-4-1", x: -300.34, z: 17.24, minX: -439.1, maxX: -232.0 }),
  Object.freeze({ index: 15, row: 4, isYC0: false, name: "YC-4-2", x: -84.01, z: 17.24, minX: -156.9, maxX: 13.6 }),
  Object.freeze({ index: 16, row: 4, isYC0: false, name: "YC-4-3", x: 111.30, z: 17.24, minX: 13.6, maxX: 218.0 }),
  Object.freeze({ index: 17, row: 4, isYC0: false, name: "YC-4-4", x: 420.18, z: 17.24, minX: 293.1, maxX: 447.8 }),
  Object.freeze({ index: 18, row: 5, isYC0: false, name: "YC-5-1", x: -300.34, z: 90.24, minX: -557.2, maxX: -232.0 }),
  Object.freeze({ index: 19, row: 5, isYC0: false, name: "YC-5-2", x: 111.30, z: 90.24, minX: -30.9, maxX: 94.9 }),
  Object.freeze({ index: 20, row: 6, isYC0: false, name: "YC-6-1", x: -300.34, z: 144.74, minX: -557.2, maxX: -232.0 }),
  Object.freeze({ index: 21, row: 6, isYC0: false, name: "YC-6-2", x: 111.30, z: 144.74, minX: -156.9, maxX: 218.0 }),
  Object.freeze({ index: 22, row: 7, isYC0: false, name: "YC-7-1", x: -300.34, z: 199.24, minX: -557.2, maxX: -232.0 }),
  Object.freeze({ index: 23, row: 7, isYC0: false, name: "YC-7-2", x: -84.01, z: 199.24, minX: -156.9, maxX: 13.6 }),
  Object.freeze({ index: 24, row: 7, isYC0: false, name: "YC-7-3", x: 111.30, z: 199.24, minX: 13.6, maxX: 218.0 }),
  Object.freeze({ index: 25, row: 8, isYC0: false, name: "YC-8-1", x: -300.34, z: 253.74, minX: -478.5, maxX: -251.7 }),
  Object.freeze({ index: 26, row: 8, isYC0: false, name: "YC-8-2", x: 111.30, z: 253.74, minX: -156.9, maxX: 198.3 }),
  Object.freeze({ index: 27, row: 9, isYC0: false, name: "YC-9-1", x: -300.34, z: 308.24, minX: -478.5, maxX: -232.0 }),
  Object.freeze({ index: 28, row: 9, isYC0: false, name: "YC-9-2", x: -84.01, z: 308.24, minX: -156.9, maxX: 13.6 }),
  Object.freeze({ index: 29, row: 9, isYC0: false, name: "YC-9-3", x: 111.30, z: 308.24, minX: 13.6, maxX: 218.0 }),
]);

export function findCraneForWorkPoint(engine, workPoint) {
  if (!workPoint || workPoint.row === "qc") return null;
  const rowList = engine._ct2?.cranesByRow?.get(workPoint.row) || [];
  if (rowList.length === 0) return null;
  const x = workPoint.point?.x ?? workPoint.x;
  let best = null;
  let minDist = Infinity;
  for (const c of rowList) {
    if (x >= c.minX && x <= c.maxX) {
      return c;
    }
    const d = Math.min(Math.abs(x - c.minX), Math.abs(x - c.maxX));
    if (d < minDist) {
      minDist = d;
      best = c;
    }
  }
  return best;
}

function sampleTaskPoint(lanes, random) {
  const laneDefinition = pickRandom(lanes, random);
  if (!laneDefinition) {
    return null;
  }

  if (laneDefinition.row === "qc") {
    // 岸侧作业车道生成的任务起点和终点都在岸桥下
    const qc = pickRandom(QC_CRANE_LOCATIONS, random);
    const x = qc.minX + 2.0 + (qc.maxX - qc.minX - 4.0) * random();
    return Object.freeze({
      kind: "work",
      id: `${laneDefinition.laneId}-${qc.id}-${x.toFixed(3)}`,
      laneId: laneDefinition.laneId,
      transitLaneId: laneDefinition.transitLaneId,
      row: laneDefinition.row,
      point: Object.freeze({ x, z: laneDefinition.z }),
    });
  }

  const boxRange = pickRandom(BOX_X_RANGES, random);
  if (!boxRange) {
    return null;
  }

  const x = boxRange.minX + (boxRange.maxX - boxRange.minX) * random();
  return createTaskPoint(laneDefinition, x, boxRange);
}

function laneProgress(lane, point) {
  if (lane.orientation === "horizontal") {
    return lane.direction.dx > 0 ? point.x : -point.x;
  }
  return lane.direction.dz > 0 ? point.z : -point.z;
}

function clampPointToLane(lane, point) {
  if (lane.orientation === "horizontal") {
    return {
      x: clamp(point.x, lane.minX, lane.maxX),
      z: lane.center,
    };
  }
  return {
    x: lane.center,
    z: clamp(point.z, lane.minZ, lane.maxZ),
  };
}

let nextNodeId = 1;
const NODE_BY_ID = new Map();
const LANE_NODE_IDS = new Map(TRANSIT_LANES.map((lane) => [lane.id, []]));
const NODE_ID_BY_KEY = new Map();

function registerNode(point, laneId, conflict = false) {
  const key = `${point.x.toFixed(3)},${point.z.toFixed(3)}`;
  let nodeId = NODE_ID_BY_KEY.get(key);
  let nodeRecord = nodeId ? NODE_BY_ID.get(nodeId) : null;

  if (!nodeRecord) {
    nodeId = nextNodeId;
    nextNodeId += 1;
    nodeRecord = {
      id: nodeId,
      x: point.x,
      z: point.z,
      conflict,
      laneIds: new Set(),
    };
    NODE_BY_ID.set(nodeId, nodeRecord);
    NODE_ID_BY_KEY.set(key, nodeId);
  }

  nodeRecord.conflict = nodeRecord.conflict || conflict;
  nodeRecord.laneIds.add(laneId);
  const laneNodeIds = LANE_NODE_IDS.get(laneId);
  if (laneNodeIds && !laneNodeIds.includes(nodeId)) {
    laneNodeIds.push(nodeId);
  }
  return nodeId;
}

for (const lane of TRANSIT_LANES) {
  registerNode(lane.start, lane.id, false);
  registerNode(lane.end, lane.id, false);
}

for (const horizontalLane of TRANSIT_HORIZONTAL_LANES) {
  for (const verticalLane of TRANSIT_VERTICAL_LANES) {
    if (
      verticalLane.center >= horizontalLane.minX - EPSILON &&
      verticalLane.center <= horizontalLane.maxX + EPSILON &&
      horizontalLane.center >= verticalLane.minZ - EPSILON &&
      horizontalLane.center <= verticalLane.maxZ + EPSILON
    ) {
      const point = { x: verticalLane.center, z: horizontalLane.center };
      registerNode(point, horizontalLane.id, true);
      registerNode(point, verticalLane.id, true);
    }
  }
}

for (const bufferLane of BUFFER_LANES) {
  for (const conn of BUFFER_CONNECTORS) {
    if (
      bufferLane.center >= conn.minX - EPSILON &&
      bufferLane.center <= conn.maxX + EPSILON &&
      (Math.abs(conn.center - bufferLane.start.z) < 0.1 || Math.abs(conn.center - bufferLane.end.z) < 0.1)
    ) {
      const point = { x: bufferLane.center, z: conn.center };
      registerNode(point, conn.id, true);
      registerNode(point, bufferLane.id, true);
    }
  }
}
for (const lane of TRANSIT_LANES) {
  const nodeIds = LANE_NODE_IDS.get(lane.id) || [];
  nodeIds.sort((leftId, rightId) => {
    const left = NODE_BY_ID.get(leftId);
    const right = NODE_BY_ID.get(rightId);
    return laneProgress(lane, left) - laneProgress(lane, right);
  });
}

const ADJACENCY = new Map();

function addBaseEdge(fromId, toId, laneId) {
  const fromNode = NODE_BY_ID.get(fromId);
  const toNode = NODE_BY_ID.get(toId);
  const cost = distance2D(fromNode, toNode);
  if (cost <= EPSILON) {
    return;
  }
  let edges = ADJACENCY.get(fromId);
  if (!edges) {
    edges = [];
    ADJACENCY.set(fromId, edges);
  }
  edges.push(Object.freeze({ to: toId, laneId, cost }));
}

for (const lane of TRANSIT_LANES) {
  const nodeIds = LANE_NODE_IDS.get(lane.id) || [];
  for (let index = 0; index < nodeIds.length - 1; index += 1) {
    addBaseEdge(nodeIds[index], nodeIds[index + 1], lane.id);
    if (lane.role === "parking" || lane.role === "buffer") {
      addBaseEdge(nodeIds[index + 1], nodeIds[index], lane.id);
    }
  }
}

function cloneAdjacency() {
  const cloned = new Map();
  for (const [nodeId, edges] of ADJACENCY.entries()) {
    cloned.set(nodeId, [...edges]);
  }
  return cloned;
}

function addLocalEdge(localAdjacency, fromId, toId, laneId, cost) {
  if (!Number.isFinite(cost) || cost < -EPSILON) {
    return;
  }
  let edges = localAdjacency.get(fromId);
  if (!edges) {
    edges = [];
    localAdjacency.set(fromId, edges);
  }
  edges.push({ to: toId, laneId, cost: Math.max(0, cost) });
}

function resolveWorkPointAccess(workPoint) {
  const workLane = LANE_BY_ID.get(workPoint.laneId);
  const transitLane = LANE_BY_ID.get(workPoint.transitLaneId);
  if (!workLane || !transitLane) {
    return null;
  }
  const span = Math.abs(workLane.center - transitLane.center);
  const upstreamWorkPoint = clampPointToLane(workLane, {
    x: workPoint.point.x - workLane.direction.dx * span,
    z: workLane.center,
  });
  const upstreamTransitPoint = clampPointToLane(transitLane, {
    x: upstreamWorkPoint.x - workLane.direction.dx * span,
    z: transitLane.center,
  });
  const downstreamWorkPoint = clampPointToLane(workLane, {
    x: workPoint.point.x + workLane.direction.dx * span,
    z: workLane.center,
  });
  const downstreamTransitPoint = clampPointToLane(transitLane, {
    x: downstreamWorkPoint.x + workLane.direction.dx * span,
    z: transitLane.center,
  });

  return Object.freeze({
    entry: Object.freeze({
      work: Object.freeze({
        laneId: workLane.id,
        point: upstreamWorkPoint,
      }),
      transit: Object.freeze({
        kind: "transit",
        laneId: transitLane.id,
        point: upstreamTransitPoint,
      }),
    }),
    exit: Object.freeze({
      work: Object.freeze({
        laneId: workLane.id,
        point: downstreamWorkPoint,
      }),
      transit: Object.freeze({
        kind: "transit",
        laneId: transitLane.id,
        point: downstreamTransitPoint,
      }),
    }),
  });
}

function createTransitLocation(laneId, point) {
  return Object.freeze({
    kind: "transit",
    laneId,
    point: Object.freeze(clonePoint(point)),
  });
}

function buildDirectBufferWorkAccessOption(workPoint, bufferLane, mode) {
  const workLane = LANE_BY_ID.get(workPoint.laneId);
  if (
    !workLane ||
    workPoint.row !== "h1" ||
    bufferLane.row !== "u" ||
    bufferLane.orientation !== "vertical"
  ) {
    return null;
  }

  const intersection = { x: bufferLane.center, z: workLane.center };
  const workProgress = laneProgress(workLane, workPoint.point);
  const intersectionProgress = laneProgress(workLane, intersection);
  const available =
    mode === "entry"
      ? workProgress - intersectionProgress
      : intersectionProgress - workProgress;
  const minAdaptiveTurnRadius = 0.25;
  const radius = Math.min(TURN_RADIUS, Math.max(0, available) * 0.5);
  if (radius < minAdaptiveTurnRadius - EPSILON) {
    return null;
  }

  const bufferDirection = vectorFromDirection(bufferLane.direction);
  const transitPoint =
    mode === "entry"
      ? {
          x: intersection.x - bufferDirection.x * radius,
          z: intersection.z - bufferDirection.z * radius,
        }
      : {
          x: intersection.x + bufferDirection.x * radius,
          z: intersection.z + bufferDirection.z * radius,
        };

  if (!samePoint(clampPointToLane(bufferLane, transitPoint), transitPoint)) {
    return null;
  }

  return Object.freeze({
    type: "buffer-direct",
    mode,
    transit: createTransitLocation(bufferLane.id, transitPoint),
    intersection: Object.freeze(intersection),
    accessCost: (radius * Math.PI) / 2 + Math.max(0, available - radius),
  });
}

function appendPreferredDirectAccessOptions(
  options,
  directOptions,
  preferredLaneIds = [],
) {
  const selectedKeys = new Set();
  const pushOption = (option) => {
    if (!option) {
      return;
    }
    const optionKey = `${option.type}:${option.transit.laneId}`;
    if (selectedKeys.has(optionKey)) {
      return;
    }
    selectedKeys.add(optionKey);
    options.push(option);
  };

  const directOptionByLaneId = new Map(
    directOptions.map((option) => [option.transit.laneId, option]),
  );
  for (const laneId of preferredLaneIds) {
    pushOption(directOptionByLaneId.get(laneId));
  }

  let rankedCount = 0;
  for (const option of directOptions) {
    const optionKey = `${option.type}:${option.transit.laneId}`;
    if (selectedKeys.has(optionKey)) {
      continue;
    }
    if (rankedCount >= H1_BUFFER_ACCESS_CANDIDATE_LIMIT) {
      break;
    }
    selectedKeys.add(optionKey);
    options.push(option);
    rankedCount += 1;
  }
}

function buildWorkEntryAccessOptions(workPoint, preferredLaneIds = []) {
  const defaultAccess = resolveWorkPointAccess(workPoint);
  if (!defaultAccess) {
    return [];
  }
  return [
    Object.freeze({
      type: "default",
      transit: defaultAccess.entry.transit,
      work: defaultAccess.entry.work,
    }),
  ];
}

function buildWorkExitAccessOptions(workPoint, preferredLaneIds = []) {
  const defaultAccess = resolveWorkPointAccess(workPoint);
  if (!defaultAccess) {
    return [];
  }
  return [
    Object.freeze({
      type: "default",
      transit: defaultAccess.exit.transit,
      work: defaultAccess.exit.work,
    }),
  ];
}
function isBufferTransitLocation(location) {
  if (location?.kind !== "transit" || typeof location.laneId !== "string") {
    return false;
  }
  return LANE_BY_ID.get(location.laneId)?.row === "u";
}

function estimateRouteTravelTime(route) {
  if (!route?.segments?.length) {
    return 0;
  }

  let totalTime = 0;
  for (const segment of route.segments) {
    const maxSpeed = Number(segment?.maxSpeed);
    if (!Number.isFinite(maxSpeed) || maxSpeed <= EPSILON) {
      return Number.POSITIVE_INFINITY;
    }
    totalTime += segment.length / maxSpeed;
  }
  return totalTime;
}

function estimateLocationHeuristicDistance(startLocation, endLocation) {
  const startPoint = startLocation?.point;
  const endPoint = endLocation?.point;
  if (!startPoint || !endPoint) {
    return Number.POSITIVE_INFINITY;
  }
  return (
    Math.abs(startPoint.x - endPoint.x) + Math.abs(startPoint.z - endPoint.z)
  );
}

function buildEntryAccessSegments(workLocation, accessOption, settings) {
  if (!accessOption) {
    return [];
  }

  if (accessOption.type === "default") {
    return [
      createLinearSegment(
        "diagonal",
        accessOption.transit.point,
        accessOption.work.point,
        settings.diagonalSpeed,
      ),
      createLinearSegment(
        "straight",
        accessOption.work.point,
        workLocation.point,
        settings.straightSpeed,
        createRoadMetadata(LANE_BY_ID.get(workLocation.laneId)),
      ),
    ].filter(Boolean);
  }

  const bufferLane = LANE_BY_ID.get(accessOption.transit.laneId);
  const workLane = LANE_BY_ID.get(workLocation.laneId);
  if (!bufferLane || !workLane) {
    return [];
  }

  const turnSegments = buildTurnSegments(
    accessOption.transit.point,
    accessOption.intersection,
    bufferLane,
    workLane,
    workLocation.point,
    settings,
  );
  if (!turnSegments) {
    return null;
  }
  return [
    ...turnSegments.segments,
    createLinearSegment(
      "straight",
      turnSegments.exitPoint,
      workLocation.point,
      settings.straightSpeed,
      createRoadMetadata(workLane),
    ),
  ].filter(Boolean);
}

function buildExitAccessSegments(workLocation, accessOption, settings) {
  if (!accessOption) {
    return [];
  }

  if (accessOption.type === "default") {
    return [
      createLinearSegment(
        "straight",
        workLocation.point,
        accessOption.work.point,
        settings.straightSpeed,
        createRoadMetadata(LANE_BY_ID.get(workLocation.laneId)),
      ),
      createLinearSegment(
        "diagonal",
        accessOption.work.point,
        accessOption.transit.point,
        settings.diagonalSpeed,
      ),
    ].filter(Boolean);
  }

  const workLane = LANE_BY_ID.get(workLocation.laneId);
  const bufferLane = LANE_BY_ID.get(accessOption.transit.laneId);
  if (!workLane || !bufferLane) {
    return [];
  }

  return (
    buildTurnSegments(
      workLocation.point,
      accessOption.intersection,
      workLane,
      bufferLane,
      accessOption.transit.point,
      settings,
    )?.segments.filter(Boolean) ?? null
  );
}

function normalizeLocationSpec(input) {
  if (typeof input === "string") {
    const workPoint = WORK_POINT_BY_ID.get(input);
    if (workPoint) {
      return {
        kind: "work",
        id: workPoint.id,
        laneId: workPoint.laneId,
        transitLaneId: workPoint.transitLaneId,
        point: clonePoint(workPoint.point),
      };
    }

    const lane = LANE_BY_ID.get(input);
    if (lane) {
      return {
        kind: "transit",
        laneId: lane.id,
        point: clonePoint(lane.start),
      };
    }

    return null;
  }

  if (!input || typeof input !== "object") {
    return null;
  }

  if (input.kind === "work") {
    if (typeof input.id === "string" && WORK_POINT_BY_ID.has(input.id)) {
      return normalizeLocationSpec(input.id);
    }
    if (
      typeof input.laneId === "string" &&
      typeof input.transitLaneId === "string" &&
      input.point
    ) {
      const laneForRow = LANE_BY_ID.get(input.laneId);
      return {
        kind: "work",
        id:
          typeof input.id === "string"
            ? input.id
            : `${input.laneId}:${input.point.x},${input.point.z}`,
        laneId: input.laneId,
        transitLaneId: input.transitLaneId,
        row:
          typeof input.row === "string" ? input.row : (laneForRow?.row ?? null),
        point: clonePoint(input.point),
      };
    }
  }

  if (typeof input.pointId === "string") {
    return normalizeLocationSpec(input.pointId);
  }

  const lane =
    typeof input.laneId === "string" ? LANE_BY_ID.get(input.laneId) : null;
  if (!lane) {
    return null;
  }

  if (WORK_POINT_BY_ID.has(input.laneId)) {
    return normalizeLocationSpec(input.laneId);
  }

  let point = null;
  if (input.point && typeof input.point === "object") {
    point = clampPointToLane(lane, input.point);
  } else if (lane.orientation === "horizontal" && Number.isFinite(input.x)) {
    point = clampPointToLane(lane, { x: Number(input.x), z: lane.center });
  } else if (lane.orientation === "vertical" && Number.isFinite(input.z)) {
    point = clampPointToLane(lane, { x: lane.center, z: Number(input.z) });
  } else if (Number.isFinite(input.progressRatio)) {
    const ratio = clamp(Number(input.progressRatio), 0, 1);
    point =
      lane.orientation === "horizontal"
        ? { x: lane.minX + (lane.maxX - lane.minX) * ratio, z: lane.center }
        : { x: lane.center, z: lane.minZ + (lane.maxZ - lane.minZ) * ratio };
  }

  const normalized = {
    kind: "transit",
    laneId: lane.id,
    point: point ? clonePoint(point) : clonePoint(lane.start),
  };
  if (typeof input.virtualDepotGateId === "string") {
    normalized.virtualDepotGateId = input.virtualDepotGateId;
  }
  return normalized;
}

function locationKey(location) {
  if (location.kind === "work") {
    return location.id;
  }
  return `${location.laneId}:${location.point.x.toFixed(2)},${location.point.z.toFixed(2)}`;
}

function locationsEqual(left, right) {
  if (!left || !right || left.kind !== right.kind) {
    return false;
  }
  if (left.kind === "work" && right.kind === "work") {
    return left.id === right.id;
  }
  return left.laneId === right.laneId && samePoint(left.point, right.point);
}

function createVirtualDepotGateLocation(gateDefinition) {
  const lane = LANE_BY_ID.get(gateDefinition?.laneId);
  if (!lane) {
    return null;
  }

  return Object.freeze({
    kind: "transit",
    laneId: lane.id,
    virtualDepotGateId: gateDefinition.id,
    point: Object.freeze(
      clampPointToLane(lane, {
        x: VIRTUAL_DEPOT_GATE_X,
        z: lane.center,
      }),
    ),
  });
}

function getVirtualDepotGateLocations() {
  return VIRTUAL_DEPOT_GATE_DEFINITIONS.map((gate) =>
    createVirtualDepotGateLocation(gate),
  ).filter(Boolean);
}

function isVirtualDepotGateLocation(location) {
  return (
    location?.kind === "transit" &&
    typeof location.virtualDepotGateId === "string"
  );
}

function getGateRowForLocation(location) {
  const gateId = location?.virtualDepotGateId;
  const gate = VIRTUAL_DEPOT_GATE_DEFINITIONS.find(
    (item) => item.id === gateId,
  );
  return gate?.row ?? null;
}

function selectVirtualDepotGateByTaskStart(taskPair) {
  const gates = getVirtualDepotGateLocations();
  if (!gates.length) {
    return null;
  }

  const startLocation =
    normalizeLocationSpec(taskPair?.start) ||
    normalizeLocationSpec(taskPair?.startLocation) ||
    normalizeLocationSpec(taskPair?.pickupLocation) ||
    null;
  const startRow =
    startLocation?.row ?? LANE_BY_ID.get(startLocation?.laneId)?.row ?? null;
  const rowPreference = {
    h1: ["depot_h3", "depot_h4", "depot_h5", "depot_h6"],
    h2: ["depot_h3", "depot_h4", "depot_h5", "depot_h6"],
    h3: ["depot_h3", "depot_h4", "depot_h5", "depot_h6"],
    h4: ["depot_h4", "depot_h3", "depot_h5", "depot_h6"],
    h5: ["depot_h5", "depot_h6", "depot_h4", "depot_h3"],
    h6: ["depot_h6", "depot_h5", "depot_h4", "depot_h3"],
  };
  const preferredIds = rowPreference[startRow] || null;

  if (preferredIds) {
    for (const gateId of preferredIds) {
      const gate = gates.find((item) => item.virtualDepotGateId === gateId);
      if (gate) {
        return gate;
      }
    }
  }

  if (startLocation?.point) {
    let best = null;
    let bestDelta = Number.POSITIVE_INFINITY;
    for (const gate of gates) {
      const delta = Math.abs(gate.point.z - startLocation.point.z);
      if (delta < bestDelta) {
        best = gate;
        bestDelta = delta;
      }
    }
    return best;
  }

  return gates[0] ?? null;
}

function createDepotGatewayLocation(preferredGateId = null) {
  if (VIRTUAL_DEPOT_ENABLED) {
    const gates = getVirtualDepotGateLocations();
    if (preferredGateId) {
      const matched = gates.find(
        (gate) => gate.virtualDepotGateId === preferredGateId,
      );
      if (matched) {
        return matched;
      }
    }
    return gates[gates.length - 1] ?? null;
  }

  const lane = LANE_BY_ID.get("h62");
  if (!lane) {
    return null;
  }
  return Object.freeze({
    kind: "transit",
    laneId: lane.id,
    point: Object.freeze(clampPointToLane(lane, { x: -460, z: lane.center })),
  });
}

function iterEngineAgents(engine) {
  if (!engine?.agents) {
    return [];
  }
  if (typeof engine.agents.values === "function") {
    return [...engine.agents.values()];
  }
  return Array.isArray(engine.agents) ? engine.agents : [];
}

function isVirtualDepotGateSpawnSafe(engine, gateLocation, igvId = null) {
  if (!engine || !gateLocation?.point) {
    return false;
  }

  const gateLane = LANE_BY_ID.get(gateLocation.laneId);
  if (!gateLane) {
    return false;
  }

  const minDistance = IGV_LENGTH + ROAD_WIDTH;
  for (const agent of iterEngineAgents(engine)) {
    if (!agent || agent.id === igvId) {
      continue;
    }
    if (
      !(agent instanceof IGVAgent) &&
      agent.type !== TYPE_IGV &&
      agent._type !== TYPE_IGV
    ) {
      continue;
    }
    if (agent._inVirtualDepot) {
      continue;
    }
    const distance = distance2D({ x: agent.x, z: agent.z }, gateLocation.point);
    if (distance < minDistance) {
      return false;
    }
  }

  return true;
}

function findSafeVirtualDepotGateForTask(taskPair, engine, igvId = null) {
  const gates = getVirtualDepotGateLocations();
  if (!gates.length) {
    return null;
  }

  const preferred = selectVirtualDepotGateByTaskStart(taskPair);
  const ordered = [];
  if (preferred) {
    ordered.push(preferred);
  }
  for (const gate of gates) {
    if (
      !ordered.some(
        (item) => item.virtualDepotGateId === gate.virtualDepotGateId,
      )
    ) {
      ordered.push(gate);
    }
  }

  for (const gate of ordered) {
    if (isVirtualDepotGateSpawnSafe(engine, gate, igvId)) {
      return gate;
    }
  }
  return null;
}

function selectVirtualDepotEntryGateForIgv(igv) {
  const gates = getVirtualDepotGateLocations();
  if (!gates.length || !igv) {
    return createDepotGatewayLocation();
  }

  const current = igv.getCurrentLocation?.() ?? igv._location ?? null;
  const currentZ = current?.point?.z ?? igv.z;
  let best = null;
  let bestDelta = Number.POSITIVE_INFINITY;
  for (const gate of gates) {
    const delta = Math.abs(gate.point.z - currentZ);
    if (delta < bestDelta) {
      best = gate;
      bestDelta = delta;
    }
  }
  return best ?? gates[0] ?? null;
}

function selectBestDepotEntryGateForIgv(igv, engine, settings) {
  return selectVirtualDepotEntryGateForIgv(igv) ?? createDepotGatewayLocation();
}

function getTaskStartLocation(task) {
  return (
    normalizeLocationSpec(task?.startLocation) ||
    normalizeLocationSpec(task?.pickupLocation) ||
    normalizeLocationSpec(task?.start) ||
    normalizeLocationSpec(task?.destination) ||
    normalizeLocationSpec(task?.from) ||
    null
  );
}

function selectBestDepotExitGateForTask(task, engine, settings, igv = null) {
  return findSafeVirtualDepotGateForTask(task, engine, igv?.id ?? null);
}

function concatenateRoutes(startLocation, endLocation, routes, settings) {
  const segments = routes.flatMap((route) => route?.segments || []);
  return decorateRouteWithResources(
    finalizeRoute(locationKey(startLocation), locationKey(endLocation), segments),
    settings,
  );
}

function connectTempPoint(
  localAdjacency,
  tempId,
  tempPoint,
  laneId,
  mode,
  otherTempId,
  otherTempPoint,
) {
  const lane = LANE_BY_ID.get(laneId);
  const laneNodeIds = LANE_NODE_IDS.get(laneId) || [];
  const progress = laneProgress(lane, tempPoint);
  const bidirectional = lane?.role === "parking" || lane?.role === "buffer";
  let previousNode = null;
  let nextNode = null;

  for (const nodeId of laneNodeIds) {
    const node = NODE_BY_ID.get(nodeId);
    const nodeProgress = laneProgress(lane, node);
    if (Math.abs(nodeProgress - progress) <= EPSILON) {
      if (mode === "start") {
        addLocalEdge(localAdjacency, tempId, nodeId, laneId, 0);
        if (bidirectional) {
          addLocalEdge(localAdjacency, nodeId, tempId, laneId, 0);
        }
      } else {
        addLocalEdge(localAdjacency, nodeId, tempId, laneId, 0);
        if (bidirectional) {
          addLocalEdge(localAdjacency, tempId, nodeId, laneId, 0);
        }
      }
      previousNode = node;
      nextNode = node;
      continue;
    }

    if (nodeProgress < progress - EPSILON) {
      previousNode = node;
      continue;
    }

    if (nodeProgress > progress + EPSILON) {
      if (!nextNode) {
        nextNode = node;
      }
    }
  }

  if (mode === "start") {
    if (nextNode) {
      addLocalEdge(
        localAdjacency,
        tempId,
        nextNode.id,
        laneId,
        distance2D(tempPoint, nextNode),
      );
    }
    if (bidirectional && previousNode) {
      addLocalEdge(
        localAdjacency,
        tempId,
        previousNode.id,
        laneId,
        distance2D(tempPoint, previousNode),
      );
    }
  }
  if (mode === "end") {
    if (previousNode) {
      addLocalEdge(
        localAdjacency,
        previousNode.id,
        tempId,
        laneId,
        distance2D(previousNode, tempPoint),
      );
    }
    if (bidirectional && nextNode) {
      addLocalEdge(
        localAdjacency,
        nextNode.id,
        tempId,
        laneId,
        distance2D(nextNode, tempPoint),
      );
    }
  }

  if (otherTempId && otherTempPoint && mode === "start") {
    const otherProgress = laneProgress(lane, otherTempPoint);
    if (otherProgress >= progress - EPSILON || bidirectional) {
      addLocalEdge(
        localAdjacency,
        tempId,
        otherTempId,
        laneId,
        distance2D(tempPoint, otherTempPoint),
      );
    }
  }
}

function buildTransitEdgeKey(fromId, toId, laneId) {
  return `${fromId}:${toId}:${laneId}`;
}

function buildShortestTransitPath(
  startTransit,
  endTransit,
  blockedEdgeKeys = null,
) {
  const startId = "__start__";
  const endId = "__end__";
  const tempPoints = new Map([
    [startId, clonePoint(startTransit.point)],
    [endId, clonePoint(endTransit.point)],
  ]);
  const localAdjacency = cloneAdjacency();
  localAdjacency.set(startId, []);
  localAdjacency.set(endId, []);

  connectTempPoint(
    localAdjacency,
    startId,
    startTransit.point,
    startTransit.laneId,
    "start",
    endId,
    endTransit.laneId === startTransit.laneId ? endTransit.point : null,
  );
  connectTempPoint(
    localAdjacency,
    endId,
    endTransit.point,
    endTransit.laneId,
    "end",
  );

  const pending = [{ id: startId, cost: 0 }];
  const visited = new Set();
  const distances = new Map([[startId, 0]]);
  const previous = new Map();

  while (pending.length > 0) {
    let minIndex = 0;
    for (let index = 1; index < pending.length; index += 1) {
      if (pending[index].cost < pending[minIndex].cost) {
        minIndex = index;
      }
    }

    const current = pending[minIndex];
    pending[minIndex] = pending[pending.length - 1];
    pending.pop();
    if (!current || visited.has(current.id)) {
      continue;
    }
    if (current.id === endId) {
      break;
    }

    visited.add(current.id);
    for (const edge of localAdjacency.get(current.id) || []) {
      if (
        blockedEdgeKeys?.has(
          buildTransitEdgeKey(current.id, edge.to, edge.laneId),
        )
      ) {
        continue;
      }
      const nextCost = current.cost + edge.cost;
      const bestCost = distances.get(edge.to);
      if (bestCost != null && nextCost >= bestCost - EPSILON) {
        continue;
      }
      distances.set(edge.to, nextCost);
      previous.set(edge.to, { from: current.id, edge });
      pending.push({ id: edge.to, cost: nextCost });
    }
  }

  if (!previous.has(endId) && startId !== endId) {
    return null;
  }

  const edges = [];
  let cursor = endId;
  while (cursor !== startId) {
    const step = previous.get(cursor);
    if (!step) {
      return null;
    }
    edges.push({ ...step.edge, from: step.from, to: cursor });
    cursor = step.from;
  }
  edges.reverse();

  return {
    edges,
    tempPoints,
  };
}

function getPointForNodeId(nodeId, tempPoints) {
  if (tempPoints?.has(nodeId)) {
    return tempPoints.get(nodeId);
  }
  return NODE_BY_ID.get(nodeId);
}

function buildIntersectionIdFromPoint(point) {
  return `cross:${point.x.toFixed(3)},${point.z.toFixed(3)}`;
}

function createRoadMetadata(lane) {
  if (!lane) {
    return null;
  }
  return {
    roadId: lane.id,
    laneId: lane.id,
    roadRole: lane.role,
  };
}

function createIntersectionMetadata(point, lane = null, nextLane = null) {
  if (!point) {
    return null;
  }

  const intersection =
    getIntersectionResourceForLanePair(lane, nextLane, point) ??
    createCoordinateIntersectionResource(point);
  return {
    intersectionId: intersection.id,
    intersectionKind: intersection.kind,
    intersectionPoint: clonePoint(point),
    fromRoadId: lane?.id ?? null,
    toRoadId: nextLane?.id ?? null,
  };
}

function pointWithinLaneBounds(lane, point, tolerance = EPSILON) {
  if (!lane || !point) {
    return false;
  }
  if (lane.orientation === "horizontal") {
    return (
      Math.abs(point.z - lane.center) <= tolerance &&
      point.x >= lane.minX - tolerance &&
      point.x <= lane.maxX + tolerance
    );
  }
  return (
    Math.abs(point.x - lane.center) <= tolerance &&
    point.z >= lane.minZ - tolerance &&
    point.z <= lane.maxZ + tolerance
  );
}

const LANE_INTERSECTION_CACHE = new Map();
const INTERSECTION_RESOURCE_CACHE = new Map();

function isH1OrH2Lane(lane) {
  return (
    lane?.orientation === "horizontal" &&
    (lane.row === "buf_conn" || lane.row === "qc")
  );
}

function isBigIntersectionHorizontalLane(lane) {
  return lane?.orientation === "horizontal" && lane.row !== "u" && lane.row !== "buf_conn";
}

function isTransitVerticalLane(lane) {
  return lane?.orientation === "vertical" && typeof lane.row === "string" && lane.row.startsWith("v_");
}

function isBufferLane(lane) {
  return lane?.orientation === "vertical" && lane.row === "u";
}

function isParkingLane(lane) {
  return false;
}

function isDepotStandbyLocation(location) {
  return false;
}
function getBufferSlotKey(lane) {
  const match =
    typeof lane?.id === "string" ? lane.id.match(/^(ub\d+)[sn]$/) : null;
  return match?.[1] ?? lane?.id ?? "buffer";
}

function getHorizontalLanesForRow(row) {
  return ALL_LANES.filter(
    (lane) => lane.orientation === "horizontal" && lane.row === row,
  );
}

function getVerticalLanesForRow(row) {
  return ALL_LANES.filter(
    (lane) => lane.orientation === "vertical" && lane.row === row,
  );
}

function createCoordinateIntersectionResource(point) {
  const id = buildIntersectionIdFromPoint(point);
  let resource = INTERSECTION_RESOURCE_CACHE.get(id);
  if (!resource) {
    resource = Object.freeze({
      id,
      kind: "coordinate",
      x: point.x,
      z: point.z,
      point: Object.freeze(clonePoint(point)),
      halfWidth: ROAD_WIDTH / 2,
      halfWidthX: ROAD_WIDTH / 2,
      halfWidthZ: ROAD_WIDTH / 2,
    });
    INTERSECTION_RESOURCE_CACHE.set(id, resource);
  }
  return resource;
}

function createSmallIntersectionResource(horizontalLane, bufferLane) {
  if (!horizontalLane || !bufferLane) {
    return null;
  }

  const slotKey = getBufferSlotKey(bufferLane);
  const id = `small:${horizontalLane.id}:${slotKey}`;
  let resource = INTERSECTION_RESOURCE_CACHE.get(id);
  if (!resource) {
    const point = { x: bufferLane.center, z: horizontalLane.center };
    resource = Object.freeze({
      id,
      kind: "small",
      x: point.x,
      z: point.z,
      point: Object.freeze(clonePoint(point)),
      halfWidth: ROAD_WIDTH / 2,
      halfWidthX: ROAD_WIDTH / 2,
      halfWidthZ: ROAD_WIDTH / 2,
      horizontalRow: horizontalLane.row,
      horizontalLaneId: horizontalLane.id,
      bufferSlot: slotKey,
    });
    INTERSECTION_RESOURCE_CACHE.set(id, resource);
  }
  return resource;
}

function createBigIntersectionResource(horizontalLane, verticalLane) {
  if (!horizontalLane || !verticalLane) {
    return null;
  }

  const id = `big:${horizontalLane.row}:${verticalLane.row}`;
  let resource = INTERSECTION_RESOURCE_CACHE.get(id);
  if (!resource) {
    const horizontalLanes = getHorizontalLanesForRow(horizontalLane.row);
    const verticalLanes = getVerticalLanesForRow(verticalLane.row);
    const minX =
      Math.min(...verticalLanes.map((lane) => lane.center)) - ROAD_WIDTH / 2;
    const maxX =
      Math.max(...verticalLanes.map((lane) => lane.center)) + ROAD_WIDTH / 2;
    const minZ =
      Math.min(...horizontalLanes.map((lane) => lane.center)) - ROAD_WIDTH / 2;
    const maxZ =
      Math.max(...horizontalLanes.map((lane) => lane.center)) + ROAD_WIDTH / 2;
    const point = { x: (minX + maxX) / 2, z: (minZ + maxZ) / 2 };
    resource = Object.freeze({
      id,
      kind: "big",
      x: point.x,
      z: point.z,
      point: Object.freeze(clonePoint(point)),
      halfWidthX: (maxX - minX) / 2,
      halfWidthZ: (maxZ - minZ) / 2,
      halfWidth: Math.max((maxX - minX) / 2, (maxZ - minZ) / 2),
      horizontalRow: horizontalLane.row,
      verticalRow: verticalLane.row,
    });
    INTERSECTION_RESOURCE_CACHE.set(id, resource);
  }
  return resource;
}

function getIntersectionResourceForLanePair(
  leftLane,
  rightLane,
  fallbackPoint = null,
) {
  if (
    !leftLane ||
    !rightLane ||
    leftLane.orientation === rightLane.orientation
  ) {
    return fallbackPoint
      ? createCoordinateIntersectionResource(fallbackPoint)
      : null;
  }

  const horizontalLane =
    leftLane.orientation === "horizontal" ? leftLane : rightLane;
  const verticalLane =
    leftLane.orientation === "vertical" ? leftLane : rightLane;
  if (isBufferLane(verticalLane) && isH1OrH2Lane(horizontalLane)) {
    return createSmallIntersectionResource(horizontalLane, verticalLane);
  }
  if (
    isTransitVerticalLane(verticalLane) &&
    isBigIntersectionHorizontalLane(horizontalLane)
  ) {
    return createBigIntersectionResource(horizontalLane, verticalLane);
  }
  return fallbackPoint
    ? createCoordinateIntersectionResource(fallbackPoint)
    : null;
}

function getLaneIntersectionProgressRange(lane, intersection) {
  if (!lane || !intersection) {
    return null;
  }

  if (lane.orientation === "horizontal") {
    return {
      min:
        intersection.x -
        (Number.isFinite(intersection.halfWidthX)
          ? intersection.halfWidthX
          : ROAD_WIDTH / 2),
      max:
        intersection.x +
        (Number.isFinite(intersection.halfWidthX)
          ? intersection.halfWidthX
          : ROAD_WIDTH / 2),
    };
  }

  return {
    min:
      intersection.z -
      (Number.isFinite(intersection.halfWidthZ)
        ? intersection.halfWidthZ
        : ROAD_WIDTH / 2),
    max:
      intersection.z +
      (Number.isFinite(intersection.halfWidthZ)
        ? intersection.halfWidthZ
        : ROAD_WIDTH / 2),
  };
}

function getSegmentRoadLane(segment) {
  if (!segment || segment.geometry !== "line" || segment.kind !== "straight") {
    return null;
  }

  const explicitLane =
    ROAD_BY_ID.get(segment.roadId) || ROAD_BY_ID.get(segment.laneId);
  if (explicitLane) {
    return explicitLane;
  }

  const horizontal =
    Math.abs(segment.direction?.x ?? 0) >= Math.abs(segment.direction?.z ?? 0);
  const candidates = ALL_LANES.filter(
    (lane) => lane.orientation === (horizontal ? "horizontal" : "vertical"),
  );
  const midpoint = {
    x: (segment.start.x + segment.end.x) / 2,
    z: (segment.start.z + segment.end.z) / 2,
  };
  let bestLane = null;
  let bestDelta = Number.POSITIVE_INFINITY;

  for (const lane of candidates) {
    if (!pointWithinLaneBounds(lane, midpoint, ROAD_WIDTH / 2 + EPSILON)) {
      continue;
    }
    const alignment = dot2D(
      vectorFromDirection(lane.direction),
      segment.direction,
    );
    if (alignment < HEADING_ALIGNMENT_THRESHOLD) {
      continue;
    }
    const delta =
      lane.orientation === "horizontal"
        ? Math.abs(midpoint.z - lane.center)
        : Math.abs(midpoint.x - lane.center);
    if (delta < bestDelta) {
      bestDelta = delta;
      bestLane = lane;
    }
  }

  return bestLane;
}

function getLaneCrossingIntersections(lane) {
  if (!lane) {
    return [];
  }
  const cached = LANE_INTERSECTION_CACHE.get(lane.id);
  if (cached) {
    return cached;
  }

  const intersectionsById = new Map();
  for (const otherLane of ALL_LANES) {
    if (
      otherLane.id === lane.id ||
      otherLane.orientation === lane.orientation
    ) {
      continue;
    }

    const crossingPoint =
      lane.orientation === "horizontal"
        ? { x: otherLane.center, z: lane.center }
        : { x: lane.center, z: otherLane.center };
    const intersection = getIntersectionResourceForLanePair(
      lane,
      otherLane,
      null,
    );
    if (!intersection) {
      continue;
    }
    const laneRange = getLaneIntersectionProgressRange(lane, intersection);
    if (!laneRange) {
      continue;
    }
    const laneMin = lane.orientation === "horizontal" ? lane.minX : lane.minZ;
    const laneMax = lane.orientation === "horizontal" ? lane.maxX : lane.maxZ;
    if (
      intersection.kind !== "small" &&
      (laneRange.max < laneMin - EPSILON || laneRange.min > laneMax + EPSILON)
    ) {
      continue;
    }

    if (
      !pointWithinLaneBounds(
        lane,
        crossingPoint,
        intersection.halfWidth ?? ROAD_WIDTH / 2,
      ) &&
      !pointInIntersectionArea(crossingPoint, intersection)
    ) {
      continue;
    }

    intersectionsById.set(intersection.id, intersection);
  }

  const intersections = Object.freeze(
    [...intersectionsById.values()].sort((left, right) => {
      const leftRange = getLaneIntersectionProgressRange(lane, left);
      const rightRange = getLaneIntersectionProgressRange(lane, right);
      const progressDelta =
        (leftRange?.min ?? laneProgress(lane, left.point)) -
        (rightRange?.min ?? laneProgress(lane, right.point));
      if (Math.abs(progressDelta) > EPSILON) {
        return progressDelta;
      }
      return left.id.localeCompare(right.id);
    }),
  );
  LANE_INTERSECTION_CACHE.set(lane.id, intersections);
  return intersections;
}

function buildIntersectionResourceDefinitions() {
  const resourcesById = new Map();

  for (const lane of ALL_LANES) {
    for (const intersection of getLaneCrossingIntersections(lane)) {
      resourcesById.set(intersection.id, intersection);
    }
  }

  return Object.freeze(
    [...resourcesById.values()].sort((left, right) => {
      if (left.kind !== right.kind) {
        return String(left.kind).localeCompare(String(right.kind));
      }
      return left.id.localeCompare(right.id);
    }),
  );
}

const INTERSECTION_RESOURCES = buildIntersectionResourceDefinitions();
const INTERSECTION_RESOURCE_BY_ID = new Map(
  INTERSECTION_RESOURCES.map((intersection) => [intersection.id, intersection]),
);

function splitProgressRange(
  startProgress,
  endProgress,
  targetLength = ROAD_SECTION_LENGTH,
) {
  const length = endProgress - startProgress;
  if (length <= EPSILON) {
    return [];
  }

  const sectionCount = Math.max(1, Math.round(length / targetLength));
  const sectionLength = length / sectionCount;
  const sections = [];

  for (let index = 0; index < sectionCount; index += 1) {
    const sectionStart = startProgress + sectionLength * index;
    const sectionEnd =
      index === sectionCount - 1
        ? endProgress
        : startProgress + sectionLength * (index + 1);
    sections.push({ startProgress: sectionStart, endProgress: sectionEnd });
  }

  return sections;
}

function getLaneResourceProgressRange(lane, intersection) {
  const rawRange = getLaneIntersectionProgressRange(lane, intersection);
  if (!rawRange) {
    return null;
  }

  const pointA =
    lane.orientation === "horizontal"
      ? { x: rawRange.min, z: lane.center }
      : { x: lane.center, z: rawRange.min };
  const pointB =
    lane.orientation === "horizontal"
      ? { x: rawRange.max, z: lane.center }
      : { x: lane.center, z: rawRange.max };
  const progressA = laneProgress(lane, pointA);
  const progressB = laneProgress(lane, pointB);

  return {
    startProgress: Math.min(progressA, progressB),
    endProgress: Math.max(progressA, progressB),
  };
}

function buildRoadSectionDefinitions() {
  const sections = [];

  for (const lane of ALL_LANES) {
    const roadStartProgress = laneProgress(lane, lane.start);
    const roadEndProgress = laneProgress(lane, lane.end);
    const intersectionRanges = getLaneCrossingIntersections(lane)
      .map((intersection) => getLaneResourceProgressRange(lane, intersection))
      .filter(Boolean)
      .map((range) => ({
        startProgress: clamp(
          range.startProgress,
          roadStartProgress,
          roadEndProgress,
        ),
        endProgress: clamp(
          range.endProgress,
          roadStartProgress,
          roadEndProgress,
        ),
      }))
      .filter(
        (range) => range.endProgress > range.startProgress + EPSILON,
      )
      .sort((left, right) => {
        if (Math.abs(left.startProgress - right.startProgress) > EPSILON) {
          return left.startProgress - right.startProgress;
        }
        return left.endProgress - right.endProgress;
      });

    let cursor = roadStartProgress;
    let sectionIndex = 0;
    const pushRoadSections = (startProgress, endProgress) => {
      if (endProgress <= startProgress + EPSILON) {
        return;
      }
      for (const split of splitProgressRange(startProgress, endProgress)) {
        sections.push(
          Object.freeze({
            id: `${lane.id}:s${sectionIndex}`,
            roadId: lane.id,
            index: sectionIndex,
            startProgress: split.startProgress,
            endProgress: split.endProgress,
            length: split.endProgress - split.startProgress,
          }),
        );
        sectionIndex += 1;
      }
    };

    for (const range of intersectionRanges) {
      if (range.startProgress > cursor + EPSILON) {
        pushRoadSections(cursor, range.startProgress);
      }
      cursor = Math.max(cursor, range.endProgress);
    }

    pushRoadSections(cursor, roadEndProgress);
  }

  return Object.freeze(sections);
}

const ROAD_SECTIONS = buildRoadSectionDefinitions();
const ROAD_SECTION_BY_ID = new Map(
  ROAD_SECTIONS.map((section) => [section.id, section]),
);
const ROAD_SECTIONS_BY_ROAD = new Map(
  ALL_LANES.map((lane) => [
    lane.id,
    Object.freeze(ROAD_SECTIONS.filter((section) => section.roadId === lane.id)),
  ]),
);

function getRoadSegmentIdsCoveredByLocation(location) {
  const normalized = normalizeLocationSpec(location);
  const lane = LANE_BY_ID.get(normalized?.laneId);
  if (!normalized?.point || !lane) {
    return [];
  }
  const progress = laneProgress(lane, normalized.point);
  return (ROAD_SECTIONS_BY_ROAD.get(lane.id) || [])
    .filter(
      (section) =>
        progress >= section.startProgress - EPSILON &&
        progress <= section.endProgress + EPSILON,
    )
    .map((section) => section.id);
}

function expandRoadResourceInterval(route, interval) {
  const lane = ROAD_BY_ID.get(interval.id);
  const roadSections = ROAD_SECTIONS_BY_ROAD.get(interval.id) || [];
  if (!lane || roadSections.length === 0) {
    return [];
  }

  const startPose = routePoseAtDistance(route, interval.startDistance);
  const endPose = routePoseAtDistance(route, interval.endDistance);
  const startProgress = laneProgress(lane, startPose?.point ?? lane.start);
  const endProgress = laneProgress(lane, endPose?.point ?? lane.end);
  if (!Number.isFinite(startProgress) || !Number.isFinite(endProgress)) {
    return [];
  }

  const progressDelta = endProgress - startProgress;
  if (progressDelta <= EPSILON) {
    return [];
  }

  const expanded = [];
  for (const section of roadSections) {
    const overlapStart = Math.max(startProgress, section.startProgress);
    const overlapEnd = Math.min(endProgress, section.endProgress);
    if (overlapEnd <= overlapStart + EPSILON) {
      continue;
    }

    expanded.push(
      Object.freeze({
        type: "road_segment",
        id: section.id,
        roadId: lane.id,
        sectionIndex: section.index,
        startDistance: interval.startDistance + (overlapStart - startProgress),
        endDistance: interval.startDistance + (overlapEnd - startProgress),
      }),
    );
  }

  return expanded;
}

function pointResourceKey(point) {
  return `${point.x.toFixed(3)},${point.z.toFixed(3)}`;
}

function intersectionCellKey(intersection, point) {
  const center = intersection?.point || intersection;
  const cellSize =
    intersection?.kind === "small"
      ? Math.max(ROAD_WIDTH, INTERSECTION_CELL_SIZE)
      : INTERSECTION_CELL_SIZE;
  const cellX = Math.floor((point.x - center.x) / cellSize);
  const cellZ = Math.floor((point.z - center.z) / cellSize);
  return `icell:${intersection?.id ?? "dynamic"}:${cellX}:${cellZ}`;
}

function collectSweptCellsForRouteInterval(route, interval) {
  const intersection = INTERSECTION_RESOURCE_BY_ID.get(interval.id);
  if (!route?.segments?.length || !intersection) {
    return Object.freeze([]);
  }

  const cells = new Set();
  const startDistance = clamp(interval.startDistance, 0, route.totalLength);
  const endDistance = clamp(interval.endDistance, startDistance, route.totalLength);
  const step =
    intersection.kind === "small"
      ? T_JUNCTION_CELL_SAMPLE_STEP
      : INTERSECTION_TRAJECTORY_SAMPLE_STEP;

  for (
    let distance = startDistance;
    distance <= endDistance + EPSILON;
    distance += step
  ) {
    const pose = routePoseAtDistance(route, Math.min(distance, endDistance));
    if (pose?.point) {
      cells.add(intersectionCellKey(intersection, pose.point));
    }
  }

  return Object.freeze([...cells]);
}

function getExitSegmentIdsForIntersectionInterval(resourceIntervals, interval, limit = 2) {
  if (!Array.isArray(resourceIntervals) && !resourceIntervals?.length) {
    return Object.freeze([]);
  }

  const ids = [];
  for (const candidate of resourceIntervals) {
    if (candidate.startDistance < interval.endDistance - EPSILON) {
      continue;
    }
    if (candidate.type !== "road_segment") {
      continue;
    }
    if (!ids.includes(candidate.id)) {
      ids.push(candidate.id);
    }
    if (ids.length >= limit) {
      break;
    }
  }
  return Object.freeze(ids);
}

function buildDiagonalResourceIntervalsForSegment(segment) {
  if (!segment || segment.length <= EPSILON) {
    return [];
  }

  const roadId = `diag:${pointResourceKey(segment.start)}:${pointResourceKey(segment.end)}`;
  return Object.freeze(
    splitProgressRange(0, segment.length).map((split, index) =>
      Object.freeze({
        type: "road_segment",
        id: `${roadId}:s${index}`,
        roadId,
        sectionIndex: index,
        startDistance: segment.startDistance + split.startProgress,
        endDistance: segment.startDistance + split.endProgress,
      }),
    ),
  );
}

function expandRouteResourceIntervals(route, intervals) {
  const expanded = [];

  for (const interval of intervals) {
    if (interval.type !== "road") {
      expanded.push(interval);
      continue;
    }

    const roadIntervals = expandRoadResourceInterval(route, interval);
    if (roadIntervals.length > 0) {
      expanded.push(...roadIntervals);
    } else {
      expanded.push(
        Object.freeze({
          ...interval,
          type: "road_segment",
          roadId: interval.id,
        }),
      );
    }
  }

  return Object.freeze(expanded);
}

function estimateBrakeDistanceAtRouteDistance(
  route,
  distance,
  settings = DEFAULT_WAREHOUSE_SETTINGS,
) {
  const location = locateRouteSegment(route, distance);
  const speed = Math.max(0, location?.segment?.maxSpeed ?? 0);
  const maxAccel = Math.max(
    EPSILON,
    settings?.maxAccel ?? DEFAULT_WAREHOUSE_SETTINGS.maxAccel,
  );
  return (speed * speed) / (2 * maxAccel);
}

function buildRouteResourceIntervalsForSegment(segment) {
  if (!segment || segment.endDistance <= segment.startDistance + EPSILON) {
    return [];
  }

  if (segment.geometry === "arc" && segment.intersectionId) {
    return [
      Object.freeze({
        type: "intersection",
        id: segment.intersectionId,
        fromRoadId: segment.fromRoadId ?? null,
        toRoadId: segment.toRoadId ?? null,
        startDistance: segment.startDistance,
        endDistance: segment.endDistance,
      }),
    ];
  }

  if (segment.geometry === "line" && segment.kind === "diagonal") {
    return buildDiagonalResourceIntervalsForSegment(segment);
  }

  const lane = getSegmentRoadLane(segment);
  if (!lane) {
    return [];
  }

  const crossingIntervals = [];
  const startProgress = laneProgress(lane, segment.start);
  const endProgress = laneProgress(lane, segment.end);
  const minProgress = Math.min(startProgress, endProgress) - EPSILON;
  const maxProgress = Math.max(startProgress, endProgress) + EPSILON;

  for (const intersection of getLaneCrossingIntersections(lane)) {
    const range = getLaneIntersectionProgressRange(lane, intersection);
    if (!range || range.max < minProgress || range.min > maxProgress) {
      continue;
    }

    const overlapMin = Math.max(range.min, minProgress);
    const overlapMax = Math.min(range.max, maxProgress);
    const localOffsetA = Math.abs(overlapMin - startProgress);
    const localOffsetB = Math.abs(overlapMax - startProgress);
    const localStart = clamp(
      segment.startDistance + Math.min(localOffsetA, localOffsetB),
      segment.startDistance,
      segment.endDistance,
    );
    const localEnd = clamp(
      segment.startDistance + Math.max(localOffsetA, localOffsetB),
      segment.startDistance,
      segment.endDistance,
    );
    if (localEnd <= localStart + EPSILON) {
      continue;
    }

    crossingIntervals.push({
      type: "intersection",
      id: intersection.id,
      fromRoadId: lane.id,
      toRoadId: lane.id,
      startDistance: localStart,
      endDistance: localEnd,
    });
  }

  crossingIntervals.sort((left, right) => {
    if (Math.abs(left.startDistance - right.startDistance) > EPSILON) {
      return left.startDistance - right.startDistance;
    }
    if (Math.abs(left.endDistance - right.endDistance) > EPSILON) {
      return left.endDistance - right.endDistance;
    }
    return left.id.localeCompare(right.id);
  });

  const intervals = [];
  let cursor = segment.startDistance;
  const pushRoadInterval = (startDistance, endDistance) => {
    if (endDistance <= startDistance + EPSILON) {
      return;
    }
    intervals.push(
      Object.freeze({
        type: "road",
        id: lane.id,
        startDistance,
        endDistance,
      }),
    );
  };

  for (const intersection of crossingIntervals) {
    const startDistance = clamp(
      intersection.startDistance,
      segment.startDistance,
      segment.endDistance,
    );
    const endDistance = clamp(
      intersection.endDistance,
      segment.startDistance,
      segment.endDistance,
    );
    if (endDistance <= cursor + EPSILON) {
      continue;
    }
    pushRoadInterval(cursor, startDistance);
    intervals.push(
      Object.freeze({
        type: "intersection",
        id: intersection.id,
        fromRoadId: lane.id,
        toRoadId: lane.id,
        startDistance: Math.max(cursor, startDistance),
        endDistance,
      }),
    );
    cursor = endDistance;
  }

  pushRoadInterval(cursor, segment.endDistance);
  return intervals;
}

function normalizeResourceIntervals(intervals) {
  const sorted = [...intervals]
    .filter(
      (interval) =>
        interval && interval.endDistance > interval.startDistance + EPSILON,
    )
    .sort((left, right) => {
      if (Math.abs(left.startDistance - right.startDistance) > EPSILON) {
        return left.startDistance - right.startDistance;
      }
      if (Math.abs(left.endDistance - right.endDistance) > EPSILON) {
        return left.endDistance - right.endDistance;
      }
      if (left.type !== right.type) {
        return left.type.localeCompare(right.type);
      }
      return String(left.id).localeCompare(String(right.id));
    });

  const normalized = [];
  for (const interval of sorted) {
    const previous = normalized[normalized.length - 1];
    if (
      previous &&
      previous.type === interval.type &&
      previous.id === interval.id &&
      interval.startDistance <= previous.endDistance + EPSILON
    ) {
      normalized[normalized.length - 1] = Object.freeze({
        ...previous,
        endDistance: Math.max(previous.endDistance, interval.endDistance),
      });
      continue;
    }
    normalized.push(Object.freeze({ ...interval }));
  }
  return Object.freeze(normalized);
}

function buildRouteResourcePlan(route, settings = DEFAULT_WAREHOUSE_SETTINGS) {
  if (!route?.segments?.length) {
    return {
      path: Object.freeze([]),
      intervals: Object.freeze([]),
      checkpoints: Object.freeze([]),
    };
  }

  const intervals = normalizeResourceIntervals(
    route.segments.flatMap((segment) =>
      buildRouteResourceIntervalsForSegment(segment),
    ),
  );
  const resourceIntervals = normalizeResourceIntervals(
    expandRouteResourceIntervals(route, intervals),
  );
  const rawActions = [];

  for (const interval of resourceIntervals) {
    const sweptCells =
      interval.type === "intersection"
        ? collectSweptCellsForRouteInterval(route, interval)
        : Object.freeze([]);
    const exitSegmentIds =
      interval.type === "intersection"
        ? getExitSegmentIdsForIntersectionInterval(
            resourceIntervals,
            interval,
            INTERSECTION_EXIT_SEGMENT_LIMIT,
          )
        : Object.freeze([]);
    const enterBoundaryDistance = clamp(
      interval.startDistance,
      0,
      route.totalLength,
    );
    const enterStopDistance = clamp(
      enterBoundaryDistance - IGV_LENGTH / 2,
      0,
      route.totalLength,
    );
    const brakeDistance = estimateBrakeDistanceAtRouteDistance(
      route,
      enterStopDistance,
      settings,
    );
    const enterLookahead = Math.min(
      IGV_LENGTH / 2 + brakeDistance + RESOURCE_ENTER_REACTION_BUFFER,
      MAX_RESOURCE_ENTER_LOOKAHEAD,
    );
    const enterTriggerDistance = clamp(
      enterBoundaryDistance - enterLookahead,
      0,
      route.totalLength,
    );
    rawActions.push({
      triggerDistance: enterTriggerDistance,
      distance: interval.startDistance,
      action: "enter",
      resourceType: interval.type,
      resourceId: interval.id,
      roadId: interval.roadId ?? null,
      fromRoadId: interval.fromRoadId ?? null,
      toRoadId: interval.toRoadId ?? null,
      sweptCells,
      exitSegmentIds,
      stopTravelDistance: enterStopDistance,
    });
    rawActions.push({
      triggerDistance: clamp(
        interval.endDistance + LOCK_ZONE_TAIL_OFFSET,
        0,
        route.totalLength,
      ),
      distance: interval.endDistance,
      action: "leave",
      resourceType: interval.type,
      resourceId: interval.id,
      roadId: interval.roadId ?? null,
      fromRoadId: interval.fromRoadId ?? null,
      toRoadId: interval.toRoadId ?? null,
      sweptCells,
      exitSegmentIds: Object.freeze([]),
    });
  }

  rawActions.push({
    triggerDistance: route.totalLength,
    distance: route.totalLength,
    action: "finish",
    resourceType: null,
    resourceId: null,
  });

  rawActions.sort((left, right) => {
    if (Math.abs(left.triggerDistance - right.triggerDistance) > EPSILON) {
      return left.triggerDistance - right.triggerDistance;
    }
    if (Math.abs(left.distance - right.distance) > EPSILON) {
      return left.distance - right.distance;
    }
    return (
      RESOURCE_ACTION_PRIORITY[left.action] -
      RESOURCE_ACTION_PRIORITY[right.action]
    );
  });

  const checkpoints = [];
  for (const rawAction of rawActions) {
    let checkpoint = checkpoints[checkpoints.length - 1];
    if (
      !checkpoint ||
      Math.abs(checkpoint.distance - rawAction.triggerDistance) > EPSILON
    ) {
      checkpoint = { distance: rawAction.triggerDistance, actions: [] };
      checkpoints.push(checkpoint);
    }
    checkpoint.actions.push(
      Object.freeze({
        action: rawAction.action,
        resourceType: rawAction.resourceType,
        resourceId: rawAction.resourceId,
        roadId: rawAction.roadId,
        fromRoadId: rawAction.fromRoadId,
        toRoadId: rawAction.toRoadId,
        sweptCells: rawAction.sweptCells,
        exitSegmentIds: rawAction.exitSegmentIds,
        distance: rawAction.distance,
        triggerDistance: rawAction.triggerDistance,
        stopTravelDistance: rawAction.stopTravelDistance,
      }),
    );
  }

  return {
    path: Object.freeze(
      intervals.map((interval) =>
        Object.freeze({ type: interval.type, id: interval.id }),
      ),
    ),
    segmentPath: Object.freeze(
      resourceIntervals.map((interval) =>
        Object.freeze({
          type: interval.type,
          id: interval.id,
          ...(interval.roadId ? { roadId: interval.roadId } : {}),
        }),
      ),
    ),
    intervals: resourceIntervals,
    checkpoints: Object.freeze(
      checkpoints.map((checkpoint) =>
        Object.freeze({
          distance: checkpoint.distance,
          type: checkpoint.actions[0]?.action ?? null,
          resourceId: checkpoint.actions[0]?.resourceId ?? null,
          segmentId:
            checkpoint.actions[0]?.resourceType === "road_segment"
              ? checkpoint.actions[0]?.resourceId
              : null,
          intersectionCellId:
            checkpoint.actions[0]?.sweptCells?.[0] ?? null,
          actions: Object.freeze(checkpoint.actions),
        }),
      ),
    ),
  };
}

function decorateRouteWithResources(
  route,
  settings = DEFAULT_WAREHOUSE_SETTINGS,
) {
  if (!route) {
    return null;
  }

  const resourcePlan = buildRouteResourcePlan(route, settings);
  return Object.freeze({
    ...route,
    resourcePath: resourcePlan.path,
    resourceSegmentPath: resourcePlan.segmentPath,
    resourceIntervals: resourcePlan.intervals,
    resourceCheckpoints: resourcePlan.checkpoints,
  });
}

function findNextResourceEntryDistance(
  route,
  currentTravelDistance,
  resourceType,
) {
  if (!route?.resourceIntervals?.length) {
    return null;
  }
  const distance = clamp(currentTravelDistance, 0, route.totalLength);
  for (const interval of route.resourceIntervals) {
    if (interval.type !== resourceType) {
      continue;
    }
    if (interval.endDistance <= distance + EPSILON) {
      continue;
    }
    return Math.max(interval.startDistance, distance);
  }
  return null;
}

function findIntersectionStopDistance(igv) {
  const route = igv?._route;
  const currentDistance = igv?.getTravelDistance?.() ?? 0;
  if (!route?.segments?.length) {
    return null;
  }

  const entryDistance = findNextResourceEntryDistance(
    route,
    currentDistance,
    "intersection",
  );
  if (entryDistance == null) {
    return null;
  }

  const stopDistance = clamp(entryDistance - IGV_LENGTH / 2, 0, route.totalLength);
  return stopDistance > currentDistance + EPSILON ? stopDistance : null;
}

function isIgvMoving(igv) {
  const state = igv?.getState?.();
  return state === S_MOVE_EMPTY || state === S_MOVE_LOADED;
}

function estimateTravelTimeToRouteDistance(igv, targetTravelDistance) {
  const route = igv?._route;
  if (!route?.segments?.length) {
    return null;
  }

  const clampedTarget = clamp(targetTravelDistance, igv._travelDistance, route.totalLength);
  if (clampedTarget <= igv._travelDistance + EPSILON) {
    return 0;
  }

  let time = 0;
  let currentDist = igv._travelDistance;

  for (const segment of route.segments) {
    if (segment.endDistance <= currentDist + EPSILON) {
      continue;
    }

    const segmentTargetDist = Math.min(segment.endDistance, clampedTarget);
    const distOnSegment = segmentTargetDist - currentDist;
    const speed = Math.max(segment.maxSpeed, 0.5);
    time += distOnSegment / speed;

    currentDist = segmentTargetDist;
    if (currentDist >= clampedTarget - EPSILON) {
      break;
    }
  }

  return time;
}

function removeValue(items, value) {
  const index = items?.indexOf(value) ?? -1;
  if (index >= 0) {
    items.splice(index, 1);
  }
}

function pushUniqueValue(items, value) {
  if (items && !items.includes(value)) {
    items.push(value);
  }
}

function buildTurnSegments(
  currentPoint,
  targetPoint,
  lane,
  nextLane,
  nextTargetPoint,
  settings,
) {
  const prevAvailable = distance2D(currentPoint, targetPoint);
  const nextAvailable = distance2D(targetPoint, nextTargetPoint);
  const minAdaptiveTurnRadius = 0.25;
  const radius = Math.min(TURN_RADIUS, prevAvailable, nextAvailable * 0.5);
  if (radius < minAdaptiveTurnRadius - EPSILON) {
    return null;
  }

  const laneDirection = vectorFromDirection(lane.direction);
  const nextLaneDirection = vectorFromDirection(nextLane.direction);
  const entry = {
    x: targetPoint.x - laneDirection.x * radius,
    z: targetPoint.z - laneDirection.z * radius,
  };
  const exit = {
    x: targetPoint.x + nextLaneDirection.x * radius,
    z: targetPoint.z + nextLaneDirection.z * radius,
  };
  const turnCross = cross2D(laneDirection, nextLaneDirection);
  const turnNormal =
    turnCross > 0 ? leftNormal(laneDirection) : rightNormal(laneDirection);
  const center = {
    x: entry.x + turnNormal.x * radius,
    z: entry.z + turnNormal.z * radius,
  };

  return {
    segments: [
      createLinearSegment(
        "straight",
        currentPoint,
        entry,
        settings.straightSpeed,
        createRoadMetadata(lane),
      ),
      createArcSegment(
        "turn",
        entry,
        exit,
        center,
        radius,
        turnCross < 0,
        settings.turnSpeed,
        createIntersectionMetadata(targetPoint, lane, nextLane),
      ),
    ],
    exitPoint: exit,
  };
}

function buildTransitPathLegs(path) {
  if (!path?.edges?.length) {
    return [];
  }

  const legs = [];
  for (const edge of path.edges) {
    const edgeStartPoint = getPointForNodeId(edge.from, path.tempPoints);
    const edgeEndPoint = getPointForNodeId(edge.to, path.tempPoints);
    const currentLeg = legs[legs.length - 1] ?? null;

    if (!currentLeg || currentLeg.laneId !== edge.laneId) {
      legs.push({
        laneId: edge.laneId,
        start: clonePoint(edgeStartPoint),
        end: clonePoint(edgeEndPoint),
        startNodeId: edge.from,
        firstToNodeId: edge.to,
        lastFromNodeId: edge.from,
        endNodeId: edge.to,
      });
      continue;
    }

    currentLeg.end = clonePoint(edgeEndPoint);
    currentLeg.lastFromNodeId = edge.from;
    currentLeg.endNodeId = edge.to;
  }

  return legs;
}

function buildTransitRoute(startTransit, endTransit, settings) {
  if (
    samePoint(startTransit.point, endTransit.point) &&
    startTransit.laneId === endTransit.laneId
  ) {
    return finalizeRoute(
      locationKey(startTransit),
      locationKey(endTransit),
      [],
    );
  }

  const blockedEdgeKeys = new Set();

  for (let attempt = 0; attempt < MAX_TRANSIT_ROUTE_RETRY_ATTEMPTS; attempt += 1) {
    const path = buildShortestTransitPath(
      startTransit,
      endTransit,
      blockedEdgeKeys,
    );
    if (!path) {
      return null;
    }

    const legs = buildTransitPathLegs(path);
    const segments = [];
    let currentPoint = clonePoint(startTransit.point);
    let needsRetry = false;

    for (let index = 0; index < legs.length; index += 1) {
      const leg = legs[index];
      const lane = LANE_BY_ID.get(leg.laneId);
      const targetPoint = leg.end;
      const nextLeg = legs[index + 1] ?? null;

      if (nextLeg && nextLeg.laneId !== leg.laneId) {
        const nextLane = LANE_BY_ID.get(nextLeg.laneId);
        const nextTargetPoint = nextLeg.end;
        if (lane.orientation !== nextLane.orientation) {
          const turnSegments = buildTurnSegments(
            currentPoint,
            targetPoint,
            lane,
            nextLane,
            nextTargetPoint,
            settings,
          );
          if (!turnSegments) {
            blockedEdgeKeys.add(
              buildTransitEdgeKey(
                leg.lastFromNodeId,
                leg.endNodeId,
                leg.laneId,
              ),
            );
            needsRetry = true;
            break;
          }
          for (const segment of turnSegments.segments) {
            if (segment) {
              segments.push(segment);
            }
          }
          currentPoint = turnSegments.exitPoint;
          continue;
        }
      }

      const straightSegment = createLinearSegment(
        "straight",
        currentPoint,
        targetPoint,
        settings.straightSpeed,
        createRoadMetadata(lane),
      );
      if (straightSegment) {
        segments.push(straightSegment);
      }
      currentPoint = clonePoint(targetPoint);
    }

    if (!needsRetry) {
      return finalizeRoute(
        locationKey(startTransit),
        locationKey(endTransit),
        segments,
      );
    }
  }

  return null;
}

function buildRoute(
  startSpec,
  endSpec,
  settings = DEFAULT_WAREHOUSE_SETTINGS,
  useDepotGateway = true,
) {
  const normalizedSettings = normalizeWarehouseSettings(settings);
  const startLocation = normalizeLocationSpec(startSpec);
  const endLocation = normalizeLocationSpec(endSpec);
  if (!startLocation || !endLocation) {
    return null;
  }
  if (
    useDepotGateway &&
    isDepotStandbyLocation(startLocation) !== isDepotStandbyLocation(endLocation)
  ) {
    const gateway = createDepotGatewayLocation();
    if (gateway && !locationsEqual(startLocation, gateway) && !locationsEqual(endLocation, gateway)) {
      const firstRoute = isDepotStandbyLocation(startLocation)
        ? buildRoute(startLocation, gateway, normalizedSettings, false)
        : buildRoute(startLocation, gateway, normalizedSettings, false);
      const secondRoute = isDepotStandbyLocation(startLocation)
        ? buildRoute(gateway, endLocation, normalizedSettings, false)
        : buildRoute(gateway, endLocation, normalizedSettings, false);
      if (firstRoute && secondRoute) {
        return concatenateRoutes(
          startLocation,
          endLocation,
          [firstRoute, secondRoute],
          normalizedSettings,
        );
      }
    }
  }
  if (locationsEqual(startLocation, endLocation)) {
    return decorateRouteWithResources(
      finalizeRoute(locationKey(startLocation), locationKey(endLocation), []),
      normalizedSettings,
    );
  }

  const startWorkPoint =
    startLocation.kind === "work"
      ? WORK_POINT_BY_ID.get(startLocation.id) || startLocation
      : null;
  const endWorkPoint =
    endLocation.kind === "work"
      ? WORK_POINT_BY_ID.get(endLocation.id) || endLocation
      : null;
  const startPreferredLaneIds =
    startWorkPoint && isBufferTransitLocation(endLocation)
      ? [endLocation.laneId]
      : [];
  const startAccessOptions = startWorkPoint
    ? buildWorkExitAccessOptions(startWorkPoint, startPreferredLaneIds)
    : [null];

  let bestRoute = null;
  let bestRouteTravelTime = Number.POSITIVE_INFINITY;

  for (const startAccess of startAccessOptions) {
    const startTransit = startAccess?.transit ?? startLocation;
    const endPreferredLaneIds =
      endWorkPoint && isBufferTransitLocation(startTransit)
        ? [startTransit.laneId]
        : [];
    const endAccessOptions = endWorkPoint
      ? buildWorkEntryAccessOptions(endWorkPoint, endPreferredLaneIds)
      : [null];

    for (const endAccess of endAccessOptions) {
      const endTransit = endAccess?.transit ?? endLocation;
      const transitRoute = buildTransitRoute(
        startTransit,
        endTransit,
        normalizedSettings,
      );
      if (!transitRoute) {
        continue;
      }

      const segments = [];
      if (startAccess) {
        const startAccessSegments = buildExitAccessSegments(
          startLocation,
          startAccess,
          normalizedSettings,
        );
        if (!startAccessSegments) {
          continue;
        }
        segments.push(...startAccessSegments);
      }
      segments.push(...transitRoute.segments);
      if (endAccess) {
        const endAccessSegments = buildEntryAccessSegments(
          endLocation,
          endAccess,
          normalizedSettings,
        );
        if (!endAccessSegments) {
          continue;
        }
        segments.push(...endAccessSegments);
      }

      const candidateRoute = finalizeRoute(
        locationKey(startLocation),
        locationKey(endLocation),
        segments,
      );
      const candidateRouteTravelTime = estimateRouteTravelTime(candidateRoute);
      if (
        !bestRoute ||
        candidateRouteTravelTime < bestRouteTravelTime - EPSILON ||
        (Math.abs(candidateRouteTravelTime - bestRouteTravelTime) <= EPSILON &&
          candidateRoute.totalLength < bestRoute.totalLength - EPSILON)
      ) {
        bestRoute = candidateRoute;
        bestRouteTravelTime = candidateRouteTravelTime;
      }
    }
  }

  return decorateRouteWithResources(bestRoute, normalizedSettings);
}

const REACHABLE_SPAWN_LANE_INDEX_RANGES = Object.freeze([
  Object.freeze([4, 4]),
  Object.freeze([10, 65]),
  Object.freeze([74, 130]),
  Object.freeze([133, 135]),
]);

function isReachableSpawnLaneIndex(index) {
  return REACHABLE_SPAWN_LANE_INDEX_RANGES.some(
    ([minIndex, maxIndex]) => index >= minIndex && index <= maxIndex,
  );
}

function normalizeFootprintLocation(location) {
  if (!location) {
    return null;
  }

  if (Number.isFinite(location.x) && Number.isFinite(location.z)) {
    return {
      point: clonePoint(location),
      laneId: null,
      transitLaneId: null,
    };
  }

  if (
    location.point &&
    Number.isFinite(location.point.x) &&
    Number.isFinite(location.point.z)
  ) {
    return {
      point: clonePoint(location.point),
      laneId: typeof location.laneId === "string" ? location.laneId : null,
      transitLaneId:
        typeof location.transitLaneId === "string"
          ? location.transitLaneId
          : null,
    };
  }

  return normalizeLocationSpec(location);
}

function getLocationDirection(location) {
  const laneId =
    typeof location?.laneId === "string"
      ? location.laneId
      : typeof location?.transitLaneId === "string"
        ? location.transitLaneId
        : null;
  const lane = laneId ? LANE_BY_ID.get(laneId) : null;
  return lane ? vectorFromDirection(lane.direction) : { x: 1, z: 0 };
}

function buildTaskFootprintRect(location) {
  const normalizedLocation = normalizeFootprintLocation(location);
  if (!normalizedLocation?.point) {
    return null;
  }

  return createRect(
    normalizedLocation.point,
    getLocationDirection(normalizedLocation),
    CONTAINER_LENGTH,
    CONTAINER_WIDTH,
  );
}

function taskLocationsOverlap(left, right) {
  return rectsOverlap(
    buildTaskFootprintRect(left),
    buildTaskFootprintRect(right),
  );
}

function buildMarkerPolygon(location) {
  const rect = buildTaskFootprintRect(location);
  return rect ? buildZonePolygonFromSlices([rect]) : [];
}

function pickRandom(items, random) {
  if (!Array.isArray(items) || items.length === 0) {
    return null;
  }
  const index = Math.min(items.length - 1, Math.floor(random() * items.length));
  return items[index];
}

function getActiveIgvs(engine) {
  const context = engine._ct2;
  if (!context) {
    return [];
  }
  return [...context.activeIgvIds]
    .map((igvId) => engine.agents.get(igvId))
    .filter((agent) => agent instanceof IGVAgent);
}

function getIgvUserTaskCount(igv) {
  if (!(igv instanceof IGVAgent) || !Array.isArray(igv._taskQueue)) {
    return 0;
  }
  const tokens = new Set();
  let anonymousUserTasks = 0;
  for (const task of igv._taskQueue) {
    if (
      !task ||
      task.internal === true ||
      task.purpose === "idle_reposition_to_depot" ||
      task.isIdleReposition === true ||
      task.parkToVirtualDepot === true
    ) {
      continue;
    }
    if (Number.isFinite(task.taskToken)) {
      tokens.add(task.taskToken);
    } else if (task.userTask === true) {
      anonymousUserTasks += 1;
    }
  }
  return tokens.size + anonymousUserTasks;
}

function getReadyIgvQueue(engine) {
  if (!engine._ct2) {
    engine._ct2 = {};
  }
  if (!Array.isArray(engine._ct2.readyIgvQueue)) {
    engine._ct2.readyIgvQueue = [];
  }
  if (!(engine._ct2.readyIgvQueuedAt instanceof Map)) {
    engine._ct2.readyIgvQueuedAt = new Map();
  }
  return engine._ct2.readyIgvQueue;
}

function removeReadyIgv(engine, igvId) {
  const queue = getReadyIgvQueue(engine);
  const index = queue.indexOf(igvId);
  if (index >= 0) {
    queue.splice(index, 1);
  }
  engine._ct2?.readyIgvQueuedAt?.delete?.(igvId);
}

function isIgvEligibleForReadyQueue(igv) {
  if (!igv) {
    return false;
  }
  if (igv._inVirtualDepot) {
    return false;
  }
  if (igv.getCarriedContainerId?.() != null || igv._carriedContainerId != null) {
    return false;
  }
  if (igv.hasActiveRoute?.()) {
    return false;
  }
  if (igv._activeAction != null) {
    return false;
  }
  if (getIgvUserTaskCount(igv) !== 0) {
    return false;
  }
  return igv.getState?.() === S_IDLE || igv.getState?.() === S_QUEUED;
}

function enqueueReadyIgv(engine, igv) {
  if (
    !ENABLE_READY_QUEUE_BEFORE_DEPOT ||
    !igv ||
    !isIgvEligibleForReadyQueue(igv)
  ) {
    return false;
  }

  const queue = getReadyIgvQueue(engine);
  if (!queue.includes(igv.id)) {
    queue.push(igv.id);
    engine._ct2.readyIgvQueuedAt.set(igv.id, getEngineClock(engine));
  }
  igv._readyQueueSince = getEngineClock(engine);
  igv._setStoppedState?.();
  return true;
}

function trimInvalidReadyQueueEntries(engine) {
  const queue = getReadyIgvQueue(engine);
  const kept = [];
  for (const igvId of queue) {
    const igv = engine.agents?.get?.(igvId) ?? null;
    if (igv && isIgvEligibleForReadyQueue(igv)) {
      kept.push(igvId);
    } else {
      engine._ct2?.readyIgvQueuedAt?.delete?.(igvId);
    }
  }
  queue.length = 0;
  queue.push(...kept);
}

function isIgvReturningToVirtualDepot(igv) {
  return (
    igv?._taskQueue?.some?.(
      (task) =>
        task?.purpose === "idle_reposition_to_depot" ||
        task?.isIdleReposition === true ||
        task?.parkToVirtualDepot === true,
    ) ?? false
  );
}

function canPrefetchUserTaskForIgv(igv) {
  if (!ENABLE_RUNNING_TASK_PREFETCH || !igv) {
    return false;
  }
  if (igv._inVirtualDepot || isIgvReturningToVirtualDepot(igv)) {
    return false;
  }
  if (
    igv.getCarriedContainerId?.() == null &&
    !igv.hasActiveRoute?.() &&
    igv._activeAction == null
  ) {
    return false;
  }
  const userTaskCount = getIgvUserTaskCount(igv);
  return userTaskCount > 0 && userTaskCount < MAX_PREFETCHED_USER_TASKS_PER_IGV;
}

function hasQueuedUserTask(igv) {
  return (
    igv?._taskQueue?.some?.(
      (task) => task?.userTask === true && task?.taskToken != null,
    ) ?? false
  );
}

function getIgvProjectedFinalLocation(igv) {
  if (!igv?._taskQueue?.length) {
    return igv?.getCurrentLocation?.() ?? igv?._location ?? null;
  }
  for (let index = igv._taskQueue.length - 1; index >= 0; index -= 1) {
    const task = igv._taskQueue[index];
    if (task?.userTask && task?.type === "move_to" && task.destination) {
      return normalizeLocationSpec(task.destination);
    }
  }
  return igv.getCurrentLocation?.() ?? igv._location ?? null;
}

function estimatePrefetchCostForIgv(igv, taskPair) {
  const finalLocation = getIgvProjectedFinalLocation(igv);
  const startLocation = normalizeLocationSpec(taskPair?.start);
  return estimateLocationHeuristicDistance(finalLocation, startLocation);
}

function assignUserTaskPairToIgv(
  taskGenerator,
  engine,
  igv,
  taskPair,
  options = {},
) {
  if (!taskGenerator || !engine || !igv || !taskPair?.start || !taskPair?.end) {
    return false;
  }

  removeReadyIgv(engine, igv.id);
  const context = engine._ct2;
  let taskToken = null;
  if (context) {
    context.taskSequence = (context.taskSequence ?? 0) + 1;
    taskToken = context.taskSequence;
    registerPendingTaskPair(engine, taskToken, taskPair);
  }

  if (Array.isArray(igv._taskQueue)) {
    igv._taskQueue = igv._taskQueue.filter(
      (task) =>
        !(
          task?.purpose === "idle_reposition_to_depot" ||
          task?.isIdleReposition === true ||
          task?.parkToVirtualDepot === true
        ),
    );
  }

  taskGenerator.commandAgent(igv.id, "move_to", {
    destination: taskPair.start,
    taskToken,
    userTask: true,
    assignmentMode: options.mode ?? null,
  });
  taskGenerator.commandAgent(igv.id, "load", {
    taskToken,
    userTask: true,
  });
  taskGenerator.commandAgent(igv.id, "move_to", {
    destination: taskPair.end,
    taskToken,
    userTask: true,
  });
  taskGenerator.commandAgent(igv.id, "unload", {
    taskToken,
    userTask: true,
  });
  igv._prefetchedTaskCount = getIgvUserTaskCount(igv);
  return true;
}

function maybeSendExcessReadyIgvsToDepot(engine) {
  if (!ENABLE_READY_QUEUE_BEFORE_DEPOT || !IDLE_REPOSITION_TO_DEPOT_ENABLED) {
    return;
  }

  trimInvalidReadyQueueEntries(engine);
  const queue = getReadyIgvQueue(engine);
  const now = getEngineClock(engine);
  const toDepotIds = [];

  if (queue.length > READY_IGV_QUEUE_MAX_SIZE) {
    toDepotIds.push(...queue.slice(READY_IGV_QUEUE_MAX_SIZE));
  }

  for (const igvId of queue) {
    const queuedAt = engine._ct2?.readyIgvQueuedAt?.get?.(igvId);
    if (
      Number.isFinite(queuedAt) &&
      now - queuedAt >= READY_IGV_QUEUE_MAX_WAIT_SECONDS
    ) {
      toDepotIds.push(igvId);
    }
  }

  for (const igvId of [...new Set(toDepotIds)]) {
    const igv = engine.agents?.get?.(igvId);
    if (!igv || !isIgvEligibleForReadyQueue(igv)) {
      removeReadyIgv(engine, igvId);
      continue;
    }
    removeReadyIgv(engine, igvId);
    igv._enqueueIdleDepotReposition?.(engine, true);
  }
}

function getPendingTaskPairStore(engine) {
  const pendingTaskPairs = engine._ct2?.pendingTaskPairs;
  return pendingTaskPairs instanceof Map ? pendingTaskPairs : null;
}

function createTaskMarkerAgent(engine, color, location) {
  const marker = new TaskMarkerAgent(engine.nextId(), color);
  marker.setMarker(location);
  engine.addAgent(marker);
  return marker;
}

function registerPendingTaskPair(engine, taskToken, taskPair) {
  const pendingTaskPairs = getPendingTaskPairStore(engine);
  if (
    !pendingTaskPairs ||
    !Number.isFinite(taskToken) ||
    !taskPair?.start ||
    !taskPair?.end
  ) {
    return;
  }

  const start = normalizeLocationSpec(taskPair.start);
  const end = normalizeLocationSpec(taskPair.end);
  if (!start || !end) {
    return;
  }

  releasePendingTaskPair(engine, taskToken);

  const startMarker = createTaskMarkerAgent(
    engine,
    TASK_START_MARKER_COLOR,
    start,
  );
  const endMarker = createTaskMarkerAgent(engine, TASK_END_MARKER_COLOR, end);
  pendingTaskPairs.set(
    taskToken,
    Object.freeze({
      start,
      end,
      startMarkerId: startMarker.id,
      endMarkerId: endMarker.id,
    }),
  );
}

function releasePendingTaskStartMarker(engine, taskToken) {
  if (!Number.isFinite(taskToken)) {
    return;
  }

  const pendingTaskPairs = getPendingTaskPairStore(engine);
  const taskPair = pendingTaskPairs?.get(taskToken);
  if (!taskPair) {
    return;
  }

  if (Number.isInteger(taskPair.startMarkerId)) {
    engine.removeAgent(taskPair.startMarkerId);
  }
  pendingTaskPairs.set(
    taskToken,
    Object.freeze({
      ...taskPair,
      startMarkerId: null,
    }),
  );
}

function releasePendingTaskPair(engine, taskToken) {
  if (!Number.isFinite(taskToken)) {
    return;
  }

  const pendingTaskPairs = getPendingTaskPairStore(engine);
  const taskPair = pendingTaskPairs?.get(taskToken);
  if (!taskPair) {
    return;
  }

  if (Number.isInteger(taskPair.startMarkerId)) {
    engine.removeAgent(taskPair.startMarkerId);
  }
  if (Number.isInteger(taskPair.endMarkerId)) {
    engine.removeAgent(taskPair.endMarkerId);
  }
  pendingTaskPairs.delete(taskToken);
}

function overlapsPendingTaskLocations(engine, location) {
  const pendingTaskPairs = getPendingTaskPairStore(engine);
  const normalizedLocation =
    normalizeLocationSpec(location) || normalizeFootprintLocation(location);
  if (!pendingTaskPairs || !normalizedLocation) {
    return false;
  }

  for (const taskPair of pendingTaskPairs.values()) {
    if (
      taskLocationsOverlap(normalizedLocation, taskPair.start) ||
      taskLocationsOverlap(normalizedLocation, taskPair.end)
    ) {
      return true;
    }
  }
  return false;
}

function isParkingSlotOccupiedByIgv(engine, laneId, requesterId = null) {
  if (!isParkingLane(LANE_BY_ID.get(laneId))) {
    return false;
  }

  for (const igvId of engine?._ct2?.activeIgvIds || []) {
    if (igvId === requesterId) {
      continue;
    }
    const igv = engine.agents?.get?.(igvId);
    if (!(igv instanceof IGVAgent)) {
      continue;
    }
    const location = igv.getCurrentLocation?.();
    if (location?.kind === "transit" && location.laneId === laneId) {
      return true;
    }
    if (
      Array.isArray(igv._taskQueue) &&
      igv._taskQueue.some(
        (task) =>
          task?.purpose === "idle_reposition_to_depot" &&
          task.destination?.laneId === laneId,
      )
    ) {
      return true;
    }
  }

  return false;
}

function createDepotStandbyLocation(laneId) {
  const lane = LANE_BY_ID.get(laneId);
  if (!isParkingLane(lane)) {
    return null;
  }

  return Object.freeze({
    kind: "transit",
    laneId,
    point: Object.freeze(
      clampPointToLane(lane, { x: lane.center, z: DEPOT_PARK_Z }),
    ),
  });
}

function findNearestAvailableDepotStandbyLocation(
  engine,
  requesterId,
  originLocation,
) {
  const origin = normalizeLocationSpec(originLocation);
  if (!origin?.point) {
    return null;
  }

  const candidates = PARKING_LANES.map((lane) => ({
    lane,
    location: createDepotStandbyLocation(lane.id),
  }))
    .filter(
      (candidate) =>
        candidate.location &&
        !isParkingSlotOccupiedByIgv(engine, candidate.lane.id, requesterId),
    )
    .map((candidate) => ({
      ...candidate,
      distance: estimateLocationHeuristicDistance(origin, candidate.location),
    }))
    .sort((left, right) => left.distance - right.distance);

  let checked = 0;
  for (const candidate of candidates) {
    if (checked >= IDLE_REPOSITION_CANDIDATE_LIMIT) {
      break;
    }
    checked += 1;
    const route = buildRoute(origin, candidate.location, DEFAULT_WAREHOUSE_SETTINGS);
    if (route?.segments?.length) {
      return candidate.location;
    }
  }

  return candidates[0]?.location ?? null;
}

function buildAllSpawnLocations() {
  return Object.freeze(
    BUFFER_SLOT_XS.map((x, index) => {
      return Object.freeze({
        kind: "transit",
        laneId: `buf_${index + 1}`,
        point: Object.freeze({ x, z: BUFFER_CENTER_Z }),
      });
    }),
  );
}
const ALL_SPAWN_LOCATIONS = buildAllSpawnLocations();
const SPAWN_LOCATIONS = ALL_SPAWN_LOCATIONS;
const CONTAINER_STATE_NAMES = Object.freeze([
  "c0",
  "c1",
  "c2",
  "c3",
  "c4",
  "c5",
]);

class ContainerAgent extends Agent {
  constructor(id, colorIndex = 0) {
    super(id, TYPE_CONTAINER, {
      modelPath: SCENE_ASSET_TEMPLATES[TYPE_CONTAINER],
      states: CONTAINER_STATE_NAMES,
      commands: [],
    });

    this._boundToIgv = null;
    this.setState(
      CONTAINER_STATE_NAMES[
        clamp(colorIndex, 0, CONTAINER_STATE_NAMES.length - 1)
      ],
    );
  }

  bindTo(igvId) {
    this._boundToIgv = igvId;
  }

  update(dt, engine) {
    if (this._boundToIgv == null) {
      return;
    }

    const igv = engine.agents.get(this._boundToIgv);
    if (!igv) {
      return;
    }

    this.setPosition(igv.x, CONTAINER_CARRY_HEIGHT, igv.z);
    this.rotY = igv.rotY;
  }
}

class TaskMarkerAgent extends Agent {
  constructor(id, color) {
    super(id, TYPE_TASK_MARKER, {
      states: ["hidden", "visible"],
      commands: [],
    });
    this._color = color;
    this._location = null;
  }

  setMarker(location) {
    this._location = normalizeFootprintLocation(location);
    this.setState(this._location ? "visible" : "hidden");
  }

  getAuxiliaryRenderData() {
    if (!this._location) {
      return null;
    }

    const vertices = buildMarkerPolygon(this._location);
    if (vertices.length === 0) {
      return null;
    }

    return {
      type: "polygon",
      vertices,
      color: this._color,
      opacity: 1,
      yOffset: 0.05,
    };
  }
}

class YardCraneAgent extends Agent {
  constructor(id, craneDef) {
    super(id, TYPE_YARD_CRANE, {
      states: ["idle", "moving", "working"],
      commands: ["assign_pickup", "assign_dropoff"],
    });

    this.craneIndex = craneDef.index;
    this.row = craneDef.row;
    this.name = craneDef.name;
    this.isYC0 = craneDef.isYC0;
    this.minX = craneDef.minX;
    this.maxX = craneDef.maxX;
    this.x = craneDef.x;
    this.y = 0.05;
    this.z = craneDef.z;
    this.rotY = 0;
    this.targetX = this.x;
    this.speed = 3.0;
    this.currentTask = null;
    this.taskQueue = [];
    this.carriedContainerId = null;
    this.setState("idle");
    this.setPosition(this.x, this.y, this.z);
  }

  assignPickup(taskToken, workPoint, containerId = null) {
    this.taskQueue.push({
      type: "pickup",
      taskToken,
      workPoint,
      containerId,
      status: "pending",
    });
    if (!this.currentTask) {
      this._startNextTask();
    }
  }

  assignDropoff(taskToken, workPoint) {
    this.taskQueue.push({
      type: "dropoff",
      taskToken,
      workPoint,
      containerId: null,
      status: "pending",
    });
    if (!this.currentTask) {
      this._startNextTask();
    }
  }

  _startNextTask() {
    if (this.taskQueue.length === 0) {
      this.currentTask = null;
      this.setState("idle");
      return;
    }
    this.currentTask = this.taskQueue.shift();
    this.currentTask.status = "moving_to_target";
    const rawX = this.currentTask.workPoint?.point?.x ?? this.currentTask.workPoint?.x ?? this.x;
    this.targetX = clamp(rawX, this.minX, this.maxX);
    this.setState("moving");
  }

  isReadyAtWorkPoint(workPoint) {
    const rawX = workPoint?.point?.x ?? workPoint?.x ?? this.targetX;
    const targetX = clamp(rawX, this.minX, this.maxX);
    return Math.abs(this.x - targetX) <= 6.0;
  }

  completeExchange(igv, engine) {
    if (!this.currentTask) {
      return false;
    }
    this.currentTask.status = "done";
    this._startNextTask();
    return true;
  }

  update(dt, engine) {
    if (this.currentTask) {
      const dx = this.targetX - this.x;
      const dist = Math.abs(dx);
      if (dist > EPSILON) {
        const step = Math.min(dist, this.speed * Math.max(0, dt));
        this.x = clamp(this.x + Math.sign(dx) * step, this.minX, this.maxX);
        this.setState("moving");
      } else {
        this.x = this.targetX;
        this.setState("working");
      }
    } else {
      this.setState("idle");
    }
    this.setPosition(this.x, this.y, this.z);
  }

  getAuxiliaryRenderData() {
    // 不在仿真场景中另外添加代表场桥的面片，直接使用现有的图形对象（YC0和YC块）
    return null;
  }
}
class IGVAgent extends Agent {
  constructor(id, startLocation, settings = DEFAULT_WAREHOUSE_SETTINGS) {
    super(id, TYPE_IGV, {
      modelPath: SCENE_ASSET_TEMPLATES[TYPE_IGV],
      states: [S_IDLE, S_QUEUED, S_WAIT, S_MOVE_EMPTY, S_MOVE_LOADED],
      commands: ["move_to", "load", "unload", "accelerate", "decelerate"],
    });

    this._settings = normalizeWarehouseSettings(settings);
    this._location = normalizeLocationSpec(startLocation);
    this._route = null;
    this._segmentIndex = 0;
    this._distanceOnSegment = 0;
    this._travelDistance = 0;
    this._currentSpeed = 0;
    this._stopTargetDistance = null;
    this._taskQueue = [];
    this._activeAction = null;
    this._actionRemaining = 0;
    this._carriedContainerId = null;
    this._spawnOrder = 0;
    this._forward = { x: 1, z: 0 };
    this._resourceCheckpointIndex = 0;
    this._resourceScheduleToken = 0;
    this._pendingResourceSchedule = false;
    this._routeJustStarted = false;
    this._boundEngine = null;
    this._boundBus = null;
    this._lastLockZoneDistance = -1;
    this._cachedLockZone = null;
    this._inVirtualDepot = false;
    this._virtualDepotGateId = null;
    this._virtualDepotSince = null;
    this._virtualDepotHidden = false;
    this._readyQueueSince = null;
    this._prefetchedTaskCount = 0;
    this._routeRetryAt = null;
    this._lastRouteFailKey = null;

    const lane =
      this._location?.kind === "work"
        ? LANE_BY_ID.get(this._location.laneId)
        : LANE_BY_ID.get(this._location?.laneId);
    const laneDirection = lane
      ? vectorFromDirection(lane.direction)
      : { x: 1, z: 0 };
    this._setForward(laneDirection.x, laneDirection.z);
    if (this._location?.point) {
      this.setPosition(this._location.point.x, 0, this._location.point.z);
    }
    this.setState(S_IDLE);
  }

  setSpawnOrder(spawnOrder) {
    this._spawnOrder = spawnOrder;
  }

  getSpawnOrder() {
    return this._spawnOrder;
  }

  bindEngine(engine) {
    this._boundEngine = engine;
    this._boundBus = engine?.bus ?? null;
    this._syncResourceSchedule(engine);
  }

  _sendPlannerMessage(plannerId, action, payload = null) {
    if (
      plannerId == null ||
      !this._boundBus ||
      typeof this._boundBus.send !== "function"
    ) {
      return undefined;
    }
    return this._boundBus.send(this.id, plannerId, action, payload);
  }

  _notifyRoadQueueManager(action, payload = null) {
    const roadQueueManagerId = this._boundEngine?._ct2?.roadQueueManagerId;
    return this._sendPlannerMessage(roadQueueManagerId, action, payload);
  }

  _notifyIntersectionManager(action, payload = null) {
    const intersectionManagerId =
      this._boundEngine?._ct2?.intersectionManagerId;
    return this._sendPlannerMessage(intersectionManagerId, action, payload);
  }

  _notifyRoadMotion(command, payload = null) {
    return this._notifyRoadQueueManager("igv_motion_command", {
      command,
      ...(payload && typeof payload === "object" ? payload : {}),
    });
  }

  getTravelDistance() {
    return this._travelDistance;
  }

  getCurrentSpeed() {
    return this._currentSpeed;
  }

  getStopTargetDistance() {
    return this._stopTargetDistance;
  }

  getResourcePath() {
    return this._route?.resourcePath || [];
  }

  getResourceSegmentPath() {
    return this._route?.resourceSegmentPath || [];
  }

  getNextResourceCheckpoint() {
    return (
      this._route?.resourceCheckpoints?.[this._resourceCheckpointIndex] || null
    );
  }

  _invalidateResourceSchedule() {
    this._resourceScheduleToken += 1;
    this._pendingResourceSchedule = true;
  }

  _syncResourceSchedule(engine = this._boundEngine) {
    const runtimeEngine = engine || this._boundEngine;
    this._boundEngine = runtimeEngine || this._boundEngine;
    this._boundBus = this._boundEngine?.bus ?? this._boundBus;
    this._pendingResourceSchedule = false;
  }

  consumeScheduledCheckpoint(eventData, engine) {
    if (
      !eventData ||
      eventData.token !== this._resourceScheduleToken ||
      eventData.checkpointIndex !== this._resourceCheckpointIndex
    ) {
      return null;
    }

    const checkpoint = this.getNextResourceCheckpoint();
    if (!checkpoint) {
      return null;
    }

    this._resourceCheckpointIndex += 1;
    this._invalidateResourceSchedule();
    this._syncResourceSchedule(engine);
    return checkpoint.actions;
  }

  _processDistanceCheckpoints(engine = this._boundEngine) {
    if (!this._route?.resourceCheckpoints?.length) {
      return true;
    }

    while (this._route) {
      const checkpoint = this.getNextResourceCheckpoint();
      if (!checkpoint) {
        return true;
      }
      if (this._travelDistance < checkpoint.distance - EPSILON) {
        return true;
      }

      const result = this._notifyIntersectionManager("sync_resource_actions", {
        checkpointIndex: this._resourceCheckpointIndex,
        checkpointDistance: checkpoint.distance,
        actions: checkpoint.actions,
      });

      if (result && result.ok === false) {
        const stopDistance =
          Number.isFinite(result.stopTravelDistance)
            ? result.stopTravelDistance
            : checkpoint.actions.find((action) =>
                Number.isFinite(action.stopTravelDistance),
              )?.stopTravelDistance;
        if (Number.isFinite(stopDistance)) {
          this._stopTargetDistance = clamp(
            stopDistance,
            0,
            this._route.totalLength,
          );
          if (this._travelDistance > this._stopTargetDistance + EPSILON) {
            this._travelDistance = this._stopTargetDistance;
            const pose = routePoseAtDistance(this._route, this._travelDistance);
            if (pose) {
              this.setPosition(pose.point.x, 0, pose.point.z);
              this._setForward(
                pose.direction.x,
                pose.direction.z,
                pose.segment?.kind === "diagonal",
              );
            }
          }
        }
        this._currentSpeed = 0;
        this.setState(S_WAIT);
        return false;
      }

      this._resourceCheckpointIndex += 1;
      this._invalidateResourceSchedule();
    }

    return true;
  }

  _releaseTrafficResources(engine = this._boundEngine) {
    this._boundEngine = engine || this._boundEngine;
    this._boundBus = this._boundEngine?.bus ?? this._boundBus;
    this._notifyIntersectionManager("release_igv", { igvId: this.id });
    this._notifyRoadQueueManager("release_igv", { igvId: this.id });
  }

  getCurrentLocation() {
    return this._location ? JSON.parse(JSON.stringify(this._location)) : null;
  }

  getPendingTaskCount() {
    return this._taskQueue.length;
  }

  getCarriedContainerId() {
    return this._carriedContainerId;
  }

  enterVirtualDepot(engine, gateLocation) {
    if (this._carriedContainerId != null) {
      return false;
    }

    this._releaseTrafficResources?.(engine);
    this._inVirtualDepot = true;
    this._virtualDepotGateId = gateLocation?.virtualDepotGateId ?? null;
    this._virtualDepotSince = getEngineClock(engine);
    this._virtualDepotHidden = true;

    this._route = null;
    this._segmentIndex = 0;
    this._distanceOnSegment = 0;
    this._travelDistance = 0;
    this._currentSpeed = 0;
    this._stopTargetDistance = null;
    this._resourceCheckpointIndex = 0;
    this._routeJustStarted = false;
    this._activeAction = null;
    this._actionRemaining = 0;
    this._taskQueue = [];
    this._idleDepotRetryAt = null;
    this._location = gateLocation ?? this._location;
    this._invalidateLockZoneCache();
    engine?._ct2?.virtualDepotPool?.add?.(this.id);

    if ("visible" in this) {
      this.visible = false;
    }
    this.setPosition?.(
      VIRTUAL_DEPOT_HIDDEN_POINT.x,
      0,
      VIRTUAL_DEPOT_HIDDEN_POINT.z,
    );
    this._setStoppedState?.();
    return true;
  }

  exitVirtualDepot(engine, gateLocation) {
    if (!gateLocation || !isVirtualDepotGateSpawnSafe(engine, gateLocation, this.id)) {
      return false;
    }

    this._inVirtualDepot = false;
    this._virtualDepotGateId = gateLocation.virtualDepotGateId ?? null;
    this._virtualDepotHidden = false;

    if ("visible" in this) {
      this.visible = true;
    }

    this._location = gateLocation;
    this._route = null;
    this._segmentIndex = 0;
    this._distanceOnSegment = 0;
    this._travelDistance = 0;
    this._currentSpeed = 0;
    this._stopTargetDistance = null;
    this._resourceCheckpointIndex = 0;
    this._routeJustStarted = false;
    this._activeAction = null;
    this._actionRemaining = 0;
    this._invalidateLockZoneCache();
    engine?._ct2?.virtualDepotPool?.delete?.(this.id);

    const lane = LANE_BY_ID.get(gateLocation.laneId);
    const direction = lane ? vectorFromDirection(lane.direction) : DIR_EAST;
    this.x = gateLocation.point.x;
    this.z = gateLocation.point.z;
    this.rotY = vehicleRotationFromDirection(direction.x, direction.z);
    this.setPosition?.(this.x, 0, this.z);
    this._setForward(direction.x, direction.z, false);
    this._setStoppedState?.();
    return true;
  }

  hasActiveRoute() {
    return this._route != null;
  }

  _setForward(dx, dz, preserveRotation = false) {
    this._forward = normalize2D(dx, dz);
    if (!preserveRotation) {
      this.rotY = vehicleRotationFromDirection(
        this._forward.x,
        this._forward.z,
      );
    }
  }

  _setStoppedState() {
    if (this._route || this._activeAction) {
      this.setState(S_WAIT);
      return;
    }
    if (this._taskQueue.length > 0) {
      this.setState(S_QUEUED);
      return;
    }
    this.setState(S_IDLE);
  }

  _setMovingState() {
    this.setState(
      this._carriedContainerId != null ? S_MOVE_LOADED : S_MOVE_EMPTY,
    );
  }

  _parseDestination(payload) {
    if (typeof payload === "string") {
      return normalizeLocationSpec(payload);
    }
    if (typeof payload?.destinationId === "string") {
      return normalizeLocationSpec(payload.destinationId);
    }
    if (payload?.destination) {
      return normalizeLocationSpec(payload.destination);
    }
    if (payload?.location) {
      return normalizeLocationSpec(payload.location);
    }
    return null;
  }

  _enqueueTask(task) {
    this._taskQueue.push(task);
    if (!this._route && !this._activeAction) {
      this._setStoppedState();
    }
  }

  _enqueueIdleDepotReposition(engine, force = false) {
    if (!IDLE_REPOSITION_TO_DEPOT_ENABLED || this._taskQueue.length > 0) {
      return false;
    }
    if (ENABLE_READY_QUEUE_BEFORE_DEPOT && force !== true) {
      return false;
    }
    if (this._inVirtualDepot || this._carriedContainerId != null) {
      return false;
    }
    if (isDepotStandbyLocation(this._location)) {
      return false;
    }

    const destination = VIRTUAL_DEPOT_ENABLED
      ? selectVirtualDepotEntryGateForIgv(this)
      : findNearestAvailableDepotStandbyLocation(engine, this.id, this._location);
    if (!destination) {
      this._idleDepotRetryAt =
        getEngineClock(engine) + IDLE_REPOSITION_RETRY_SECONDS;
      return false;
    }

    this._enqueueTask({
      type: "move_to",
      destination,
      internal: true,
      purpose: "idle_reposition_to_depot",
      isIdleReposition: true,
      virtualDepotGateId: destination.virtualDepotGateId ?? null,
      parkToVirtualDepot: VIRTUAL_DEPOT_ENABLED,
    });
    return true;
  }

  _currentSegment() {
    return this._route?.segments?.[this._segmentIndex] || null;
  }

  _invalidateLockZoneCache() {
    this._lastLockZoneDistance = -1;
    this._cachedLockZone = null;
  }

  getLockZone() {
    if (this._inVirtualDepot) {
      return null;
    }
    if (
      this._cachedLockZone &&
      Math.abs(this._travelDistance - this._lastLockZoneDistance) < 0.5
    ) {
      return this._cachedLockZone;
    }

    this._lastLockZoneDistance = this._travelDistance;
    this._cachedLockZone = buildLockZone(this._route, this._travelDistance, {
      point: { x: this.x, z: this.z },
      direction: this._forward,
    });
    return this._cachedLockZone;
  }

  getRoutePose() {
    if (this._route) {
      return routePoseAtDistance(this._route, this._travelDistance);
    }
    return {
      point: { x: this.x, z: this.z },
      direction: this._forward,
    };
  }

  getAuxiliaryRenderData() {
    if (!this._settings.showLockZone) {
      return null;
    }

    const zone = this.getLockZone();
    if (!zone?.polygon?.length) {
      return null;
    }

    const moving =
      this.getState() === S_MOVE_EMPTY || this.getState() === S_MOVE_LOADED;
    const style = SCENE_STATE_STYLE_MAPPING[TYPE_LOCK_ZONE][moving ? 0 : 1];
    return {
      type: "polygon",
      vertices: zone.polygon,
      color: style.color,
      opacity: style.opacity,
      yOffset: LOCK_ZONE_RENDER_Y,
    };
  }

  _handleMoveTo(payload) {
    const destination = this._parseDestination(payload);
    if (!destination) {
      return { ok: false };
    }
    this._enqueueTask({
      type: "move_to",
      destination,
      taskToken: Number.isFinite(payload?.taskToken) ? payload.taskToken : null,
      userTask: payload?.userTask === true,
    });
    return { ok: true };
  }

  _handleLoad(payload) {
    this._enqueueTask({
      type: "load",
      taskToken: Number.isFinite(payload?.taskToken) ? payload.taskToken : null,
      userTask: payload?.userTask === true,
    });
    return { ok: true };
  }

  _handleUnload(payload) {
    this._enqueueTask({
      type: "unload",
      clearTaskMarkers: payload?.clearTaskMarkers === true,
      taskToken: Number.isFinite(payload?.taskToken) ? payload.taskToken : null,
      userTask: payload?.userTask === true,
    });
    return { ok: true };
  }

  _handleAccelerate() {
    const previousMoving = isIgvMoving(this);
    const previousStopTargetDistance = this._stopTargetDistance;
    let routeCreated = false;

    if (!this._route && this._taskQueue[0]?.type === "move_to") {
      const now = getEngineClock(this._boundEngine);
      if (Number.isFinite(this._routeRetryAt) && now < this._routeRetryAt) {
        this._setStoppedState();
        return { ok: false };
      }
      const route = buildRoute(
        this._location,
        this._taskQueue[0].destination,
        this._settings,
      );
      if (route?.segments?.length) {
        this._routeRetryAt = null;
        this._lastRouteFailKey = null;
        this._route = route;
        routeCreated = true;
        this._segmentIndex = 0;
        this._distanceOnSegment = 0;
        this._travelDistance = 0;
        this._resourceCheckpointIndex = 0;
        this._routeJustStarted = true;
        this._invalidateLockZoneCache();
        const firstSegment = this._currentSegment();
        if (firstSegment) {
          const firstDirection = directionOnSegment(firstSegment, 0);
          this._setForward(
            firstDirection.x,
            firstDirection.z,
            firstSegment.kind === "diagonal",
          );
        }
      } else {
        const task = this._taskQueue[0];
        if (
          task?.purpose === "idle_reposition_to_depot" ||
          task?.isIdleReposition === true ||
          task?.parkToVirtualDepot === true
        ) {
          this._taskQueue.shift();
          this._idleDepotRetryAt = now + IDLE_REPOSITION_RETRY_SECONDS;
          this._setStoppedState();
          return { ok: false };
        }
        this._routeRetryAt = now + ROUTE_RETRY_BACKOFF_SECONDS;
        this._lastRouteFailKey = `${locationKey(this._location)}>${locationKey(
          task.destination,
        )}`;
      }
    }

    if (this._route) {
      this._stopTargetDistance = null;
      this._setMovingState();
      if (
        routeCreated ||
        !previousMoving ||
        previousStopTargetDistance != null
      ) {
        this._invalidateResourceSchedule();
        this._syncResourceSchedule();
        this._notifyRoadMotion("accelerate");
      }
      return { ok: true };
    }

    return { ok: false };
  }

  _handleDecelerate(payload) {
    const previousMoving = isIgvMoving(this);
    const previousStopTargetDistance = this._stopTargetDistance;
    const stopTravelDistance =
      typeof payload === "number"
        ? payload
        : typeof payload?.stopTravelDistance === "number"
          ? payload.stopTravelDistance
          : null;

    if (Number.isFinite(stopTravelDistance) && this._route) {
      this._stopTargetDistance = clamp(
        stopTravelDistance,
        this._travelDistance,
        this._route.totalLength,
      );
    } else {
      this._stopTargetDistance = null;
    }

    if (this._route) {
      this.setState(S_WAIT);
    }

    const stopTargetChanged =
      (previousStopTargetDistance == null) !==
        (this._stopTargetDistance == null) ||
      (previousStopTargetDistance != null &&
        this._stopTargetDistance != null &&
        Math.abs(previousStopTargetDistance - this._stopTargetDistance) >
          EPSILON);
    if (previousMoving || stopTargetChanged) {
      this._invalidateResourceSchedule();
      this._syncResourceSchedule();
      this._notifyRoadMotion(
        "decelerate",
        Number.isFinite(this._stopTargetDistance)
          ? { stopTravelDistance: this._stopTargetDistance }
          : null,
      );
    }
    return { ok: true };
  }

  onMessage(envelope) {
    const { action, payload } = envelope;

    if (action === "move_to") {
      return this._handleMoveTo(payload);
    }
    if (action === "load") {
      return this._handleLoad(payload);
    }
    if (action === "unload") {
      return this._handleUnload(payload);
    }
    if (action === "accelerate") {
      return this._handleAccelerate();
    }
    if (action === "decelerate") {
      return this._handleDecelerate(payload);
    }
    if (action === "query_state") {
      return this.getState();
    }
    if (action === "query_position") {
      return this.getPosition();
    }
    if (action === "query_lock_zone") {
      return this.getLockZone();
    }

    return super.onMessage(envelope);
  }

  _syncCarriedContainer(engine) {
    if (this._carriedContainerId == null) {
      return;
    }

    const container = engine.agents.get(this._carriedContainerId);
    if (!container) {
      return;
    }

    container.bindTo(this.id);
    container.setPosition(this.x, CONTAINER_CARRY_HEIGHT, this.z);
    container.rotY = this.rotY;
  }

  _applySegmentPose(segment, distanceOnSegment) {
    const point = pointOnSegment(segment, distanceOnSegment);
    const direction = directionOnSegment(segment, distanceOnSegment);
    this.setPosition(point.x, 0, point.z);
    this._setForward(direction.x, direction.z, segment.kind === "diagonal");
  }

  _completeMoveTask(engine) {
    this._releaseTrafficResources(engine);
    this._invalidateResourceSchedule();
    this._resourceCheckpointIndex = 0;
    const task = this._taskQueue.shift();
    if (
      VIRTUAL_DEPOT_ENABLED &&
      task?.parkToVirtualDepot &&
      isVirtualDepotGateLocation(task.destination) &&
      this._carriedContainerId == null
    ) {
      this.enterVirtualDepot(engine, task.destination);
      return;
    }
    if (task?.destination) {
      this._location = normalizeLocationSpec(task.destination);
    }
    this._route = null;
    this._segmentIndex = 0;
    this._distanceOnSegment = 0;
    this._travelDistance = 0;
    this._currentSpeed = 0;
    this._stopTargetDistance = null;
    this._routeRetryAt = null;
    this._lastRouteFailKey = null;
    this._invalidateLockZoneCache();
    if (this._location?.point) {
      this.setPosition(this._location.point.x, 0, this._location.point.z);
      const lane = LANE_BY_ID.get(
        this._location.kind === "work"
          ? this._location.laneId
          : this._location.laneId,
      );
      const direction = lane
        ? vectorFromDirection(lane.direction)
        : this._forward;
      this._setForward(direction.x, direction.z, false);
    }
    this._setStoppedState();
    this._syncCarriedContainer(engine);
  }

  _completeHandlingTask(engine) {
    const task = this._taskQueue.shift();
    if (task?.type === "load") {
      if (this._carriedContainerId == null) {
        const colorIndex = Math.min(
          CONTAINER_STATE_NAMES.length - 1,
          Math.floor(engine.rng.uniform(0, CONTAINER_STATE_NAMES.length)()),
        );
        const container = new ContainerAgent(engine.nextId(), colorIndex);
        container.bindTo(this.id);
        container.setPosition(this.x, CONTAINER_CARRY_HEIGHT, this.z);
        container.rotY = this.rotY;
        engine.addAgent(container);
        this._carriedContainerId = container.id;
      }
      releasePendingTaskStartMarker(engine, task.taskToken);
    }

    if (task?.type === "unload" && this._carriedContainerId != null) {
      engine.removeAgent(this._carriedContainerId);
      this._carriedContainerId = null;
    }
    if (task?.type === "unload") {
      releasePendingTaskPair(engine, task.taskToken);
    }

    this._activeAction = null;
    this._actionRemaining = 0;
    if (this._taskQueue.length > 0) {
      if (task?.type === "unload" && hasQueuedUserTask(this)) {
        this._setStoppedState();
        this._startCurrentTask(engine);
        return;
      }
      if (task?.type === "load") {
        this._setStoppedState();
        this._startCurrentTask(engine);
        return;
      }
    }
    if (task?.type === "unload") {
      if (!hasQueuedUserTask(this)) {
        enqueueReadyIgv(engine, this);
      }
    }
    this._setStoppedState();
  }

  _startCurrentTask(engine) {
    const task = this._taskQueue[0];
    if (!task || this._route || this._activeAction) {
      return;
    }

    if (task.type === "move_to") {
      const now = getEngineClock(engine);
      if (Number.isFinite(this._routeRetryAt) && now < this._routeRetryAt) {
        this._setStoppedState();
        return;
      }
      if (locationsEqual(this._location, task.destination)) {
        this._completeMoveTask(engine);
        return;
      }

      const route = buildRoute(
        this._location,
        task.destination,
        this._settings,
      );
      if (!route) {
        if (
          task.purpose === "idle_reposition_to_depot" ||
          task.isIdleReposition === true ||
          task.parkToVirtualDepot === true
        ) {
          this._taskQueue.shift();
          this._idleDepotRetryAt = now + IDLE_REPOSITION_RETRY_SECONDS;
          this._setStoppedState();
          return;
        }
        this._routeRetryAt = now + ROUTE_RETRY_BACKOFF_SECONDS;
        this._lastRouteFailKey = `${locationKey(this._location)}>${locationKey(
          task.destination,
        )}`;
        this._setStoppedState();
        return;
      }
      this._routeRetryAt = null;
      this._lastRouteFailKey = null;

      if (!route.segments.length) {
        this._completeMoveTask(engine);
        return;
      }

      this._route = route;
      this._segmentIndex = 0;
      this._distanceOnSegment = 0;
      this._travelDistance = 0;
      this._stopTargetDistance = null;
      this._resourceCheckpointIndex = 0;
      this._routeJustStarted = true;
      this._invalidateLockZoneCache();
      const firstSegment = this._currentSegment();
      if (firstSegment) {
        const firstDirection = directionOnSegment(firstSegment, 0);
        this._setForward(
          firstDirection.x,
          firstDirection.z,
          firstSegment.kind === "diagonal",
        );
      }
      this._setMovingState();
      this._invalidateResourceSchedule();
      this._syncResourceSchedule(engine);
      this._notifyRoadMotion("accelerate");
      return;
    }

    this._activeAction = task.type;
    this._actionRemaining = this._settings.handlingDuration;
    if (this._actionRemaining <= EPSILON) {
      this._completeHandlingTask(engine);
      return;
    }
    this.setState(S_WAIT);
  }

  _advanceAlongRoute(moveDistance, engine) {
    let remainingDistance = moveDistance;

    while (remainingDistance > EPSILON && this._route) {
      const segment = this._currentSegment();
      if (!segment) {
        this._completeMoveTask(engine);
        return;
      }

      const remainingOnSegment = segment.length - this._distanceOnSegment;
      if (remainingDistance + EPSILON >= remainingOnSegment) {
        this._distanceOnSegment = segment.length;
        this._applySegmentPose(segment, this._distanceOnSegment);
        this._travelDistance += remainingOnSegment;
        remainingDistance -= remainingOnSegment;
        this._segmentIndex += 1;
        this._distanceOnSegment = 0;

        const nextSegment = this._currentSegment();
        if (!nextSegment) {
          this._travelDistance = this._route.totalLength;
          this._processDistanceCheckpoints(engine);
          this._completeMoveTask(engine);
          return;
        }

        this.setPosition(nextSegment.start.x, 0, nextSegment.start.z);
        const nextDirection = directionOnSegment(nextSegment, 0);
        this._setForward(
          nextDirection.x,
          nextDirection.z,
          nextSegment.kind === "diagonal",
        );
        continue;
      }

      this._distanceOnSegment += remainingDistance;
      this._travelDistance += remainingDistance;
      this._applySegmentPose(segment, this._distanceOnSegment);
      remainingDistance = 0;
    }
  }

  update(dt, engine) {
    this._boundEngine = engine || this._boundEngine;
    this._boundBus = this._boundEngine?.bus ?? this._boundBus;
    this._syncResourceSchedule(engine);
    if (this._inVirtualDepot) {
      this._currentSpeed = 0;
      this._setStoppedState?.();
      return;
    }
    const safeDt = Math.max(0, dt);

    if (!this._route && !this._activeAction && this._taskQueue.length > 0) {
      this._startCurrentTask(engine);
    }
    if (
      IDLE_REPOSITION_TO_DEPOT_ENABLED &&
      !this._route &&
      !this._activeAction &&
      this._taskQueue.length === 0 &&
      !isDepotStandbyLocation(this._location) &&
      (!Number.isFinite(this._idleDepotRetryAt) ||
        getEngineClock(engine) >= this._idleDepotRetryAt - EPSILON)
    ) {
      if (!enqueueReadyIgv(engine, this)) {
        this._enqueueIdleDepotReposition(engine);
      }
    }

    if (this._routeJustStarted) {
      this._routeJustStarted = false;
      this._syncCarriedContainer(engine);
      this._processDistanceCheckpoints(engine);
      return;
    }

    if (this._activeAction) {
      const isYard = this._location?.row && this._location.row !== "qc";
      const crane = isYard ? findCraneForWorkPoint(engine, this._location) : null;
      if (crane && !crane.isReadyAtWorkPoint(this._location)) {
        this.setState(S_WAIT);
        this._syncCarriedContainer(engine);
        return;
      }
      this._actionRemaining = Math.max(0, this._actionRemaining - safeDt);
      if (this._actionRemaining <= EPSILON) {
        if (crane) {
          crane.completeExchange(this, engine);
        }
        this._completeHandlingTask(engine);
      }
      this._syncCarriedContainer(engine);
      return;
    }

    const segment = this._currentSegment();
    const brakingToTarget =
      this.getState() === S_WAIT &&
      segment &&
      Number.isFinite(this._stopTargetDistance) &&
      this._stopTargetDistance > this._travelDistance + EPSILON;
    const remainingStopDistance = brakingToTarget
      ? Math.max(0, this._stopTargetDistance - this._travelDistance)
      : 0;
    const brakingEnvelopeSpeed = brakingToTarget
      ? Math.sqrt(2 * this._settings.maxAccel * remainingStopDistance)
      : 0;
    const targetSpeed =
      (this.getState() === S_MOVE_EMPTY || this.getState() === S_MOVE_LOADED) &&
      segment
        ? segment.maxSpeed
        : brakingToTarget
          ? Math.min(this._currentSpeed, segment.maxSpeed, brakingEnvelopeSpeed)
          : 0;
    const accelDelta = this._settings.maxAccel * safeDt;
    const speedDelta = clamp(
      targetSpeed - this._currentSpeed,
      -accelDelta,
      accelDelta,
    );
    let nextSpeed = Math.max(0, this._currentSpeed + speedDelta);
    let moveDistance = ((this._currentSpeed + nextSpeed) / 2) * safeDt;

    if (brakingToTarget && moveDistance >= remainingStopDistance - EPSILON) {
      moveDistance = remainingStopDistance;
      nextSpeed = 0;
    }

    this._currentSpeed = nextSpeed;
    if (moveDistance > EPSILON && this._route) {
      this._advanceAlongRoute(moveDistance, engine);
      this._processDistanceCheckpoints(engine);
    }

    if (
      Number.isFinite(this._stopTargetDistance) &&
      this._route &&
      this._travelDistance >= this._stopTargetDistance - EPSILON
    ) {
      this._travelDistance = this._stopTargetDistance;
      const stopPose = routePoseAtDistance(this._route, this._travelDistance);
      if (stopPose) {
        this.setPosition(stopPose.point.x, 0, stopPose.point.z);
        this._setForward(
          stopPose.direction.x,
          stopPose.direction.z,
          stopPose.segment?.kind === "diagonal",
        );
      }
      this._currentSpeed = 0;
    }

    if (this._currentSpeed <= EPSILON) {
      this._currentSpeed = 0;
      if (
        this._route &&
        this.getState() !== S_MOVE_EMPTY &&
        this.getState() !== S_MOVE_LOADED
      ) {
        this.setState(S_WAIT);
      }
    }

    this._syncCarriedContainer(engine);
    this._processDistanceCheckpoints(engine);
  }
}

function createTrafficManagerState() {
  return {
    roadResources: new Map(
      ALL_LANES.map((lane) => [lane.id, { id: lane.id, queue: [] }]),
    ),
    roadSegmentResources: new Map(
      ROAD_SECTIONS.map((section) => [
        section.id,
        {
          id: section.id,
          roadId: section.roadId,
          occupiedBy: null,
          reservedBy: null,
          reservedUntil: null,
          reservedReason: null,
          reservedIntersectionId: null,
          waitingQueue: [],
          waitingSinceByIgvId: new Map(),
        },
      ]),
    ),
    intersectionResources: new Map(
      INTERSECTION_RESOURCES.map((intersection) => [
        intersection.id,
        {
          id: intersection.id,
          kind: intersection.kind,
          occupiedBy: null,
          activeVehicles: new Set(),
          activeCells: new Map(),
          waitingQueue: [],
        },
      ]),
    ),
    igvStates: new Map(),
  };
}

function ensureTrafficIgvState(trafficState, igvId) {
  let state = trafficState.igvStates.get(igvId);
  if (!state) {
    state = {
      roadIds: new Set(),
      roadSegmentIds: new Set(),
      waitingRoadSegmentId: null,
      waitingRoadStopDistance: null,
      intersectionIds: new Set(),
      waitingIntersectionId: null,
    };
    trafficState.igvStates.set(igvId, state);
  }
  return state;
}

function ensureTrafficIntersectionState(trafficState, intersectionId) {
  if (!intersectionId) {
    return null;
  }
  let state = trafficState.intersectionResources.get(intersectionId);
  if (!state) {
    const definition = INTERSECTION_RESOURCE_BY_ID.get(intersectionId);
    state = {
      id: intersectionId,
      kind: definition?.kind ?? "dynamic",
      occupiedBy: null,
      activeVehicles: new Set(),
      activeCells: new Map(),
      waitingQueue: [],
    };
    trafficState.intersectionResources.set(intersectionId, state);
  }
  return state;
}

class RoadQueueManager extends Scheduler {
  constructor(id, trafficState = createTrafficManagerState()) {
    super(id, "road-queue-manager");
    this._trafficState = trafficState;
    this._boundEngine = null;
  }

  bindEngine(engine) {
    this._boundEngine = engine;
  }

  _ensureIgvState(igvId) {
    return ensureTrafficIgvState(this._trafficState, igvId);
  }

  _ensureRoadResource(roadId) {
    if (!roadId) {
      return null;
    }
    let roadState = this._trafficState.roadResources.get(roadId);
    if (!roadState) {
      roadState = { id: roadId, queue: [] };
      this._trafficState.roadResources.set(roadId, roadState);
    }
    return roadState;
  }

  _getRoadIdForSegment(segmentId, fallbackRoadId = null) {
    return (
      this._trafficState.roadSegmentResources.get(segmentId)?.roadId ||
      ROAD_SECTION_BY_ID.get(segmentId)?.roadId ||
      fallbackRoadId
    );
  }

  _ensureRoadSegmentState(segmentId, roadId = null) {
    if (!segmentId) {
      return null;
    }

    let segmentState = this._trafficState.roadSegmentResources.get(segmentId);
    const resolvedRoadId = this._getRoadIdForSegment(segmentId, roadId);
    if (!segmentState) {
      segmentState = {
        id: segmentId,
        roadId: resolvedRoadId,
        occupiedBy: null,
        reservedBy: null,
        reservedUntil: null,
        reservedReason: null,
        reservedIntersectionId: null,
        waitingQueue: [],
        waitingSinceByIgvId: new Map(),
      };
      this._trafficState.roadSegmentResources.set(segmentId, segmentState);
    } else if (!segmentState.roadId && resolvedRoadId) {
      segmentState.roadId = resolvedRoadId;
    }
    if (!(segmentState.waitingSinceByIgvId instanceof Map)) {
      segmentState.waitingSinceByIgvId = new Map();
    }
    if (!("reservedBy" in segmentState)) {
      segmentState.reservedBy = null;
      segmentState.reservedUntil = null;
      segmentState.reservedReason = null;
      segmentState.reservedIntersectionId = null;
    }

    if (segmentState.roadId) {
      this._ensureRoadResource(segmentState.roadId);
    }
    return segmentState;
  }

  getRoadQueue(roadId) {
    return [...(this._trafficState.roadResources.get(roadId)?.queue || [])];
  }

  getRoadSegmentState(segmentId) {
    const segmentState = this._trafficState.roadSegmentResources.get(segmentId);
    if (!segmentState) {
      return null;
    }
    return {
      occupiedBy: segmentState.occupiedBy,
      reservedBy: segmentState.reservedBy ?? null,
      reservedUntil: segmentState.reservedUntil ?? null,
      reservedReason: segmentState.reservedReason ?? null,
      reservedIntersectionId: segmentState.reservedIntersectionId ?? null,
      waitingQueue: [...segmentState.waitingQueue],
    };
  }

  getIgvResourceState(igvId) {
    const igvState = this._trafficState.igvStates.get(igvId);
    if (!igvState) {
      return null;
    }
    return {
      roadIds: [...igvState.roadIds],
      roadSegmentIds: [...igvState.roadSegmentIds],
      waitingRoadSegmentId: igvState.waitingRoadSegmentId,
      waitingRoadStopDistance: igvState.waitingRoadStopDistance,
      intersectionIds: [...igvState.intersectionIds],
      waitingIntersectionId: igvState.waitingIntersectionId,
    };
  }

  _findRouteRoadSegmentId(igv, roadId = null) {
    if (!(igv instanceof IGVAgent) || !igv._route?.resourceIntervals?.length) {
      return null;
    }

    const travelDistance = igv.getTravelDistance?.() ?? 0;
    const activeInterval = igv._route.resourceIntervals.find(
      (interval) =>
        interval.type === "road_segment" &&
        (!roadId || interval.roadId === roadId) &&
        travelDistance >=
          interval.startDistance - LOCK_ZONE_HEAD_OFFSET - ROUTE_SEARCH_STEP &&
        travelDistance <=
          interval.endDistance + LOCK_ZONE_TAIL_OFFSET + ROUTE_SEARCH_STEP,
    );
    if (activeInterval) {
      return activeInterval.id;
    }

    const nextInterval = igv._route.resourceIntervals.find(
      (interval) =>
        interval.type === "road_segment" &&
        (!roadId || interval.roadId === roadId) &&
        interval.startDistance >= travelDistance - ROUTE_SEARCH_STEP,
    );
    return nextInterval?.id || null;
  }

  _resolveRoadSegmentId(envelope, igv = null) {
    const payload = envelope.payload || {};
    if (typeof payload.segmentId === "string") {
      return payload.segmentId;
    }
    if (
      typeof payload.resourceId === "string" &&
      (ROAD_SECTION_BY_ID.has(payload.resourceId) ||
        this._trafficState.roadSegmentResources.has(payload.resourceId))
    ) {
      return payload.resourceId;
    }
    return this._findRouteRoadSegmentId(igv, payload.roadId);
  }

  _addIgvToRoadState(igvId, roadId, segmentId) {
    const igvState = this._ensureIgvState(igvId);
    if (roadId) {
      igvState.roadIds.add(roadId);
      pushUniqueValue(this._ensureRoadResource(roadId)?.queue || [], igvId);
    }
    if (segmentId) {
      igvState.roadSegmentIds.add(segmentId);
    }
  }

  _releaseRoadQueueIfDone(igvId, roadId) {
    if (!roadId) {
      return;
    }

    const igvState = this._trafficState.igvStates.get(igvId);
    const stillOccupiesRoad = [...(igvState?.roadSegmentIds || [])].some(
      (segmentId) => this._getRoadIdForSegment(segmentId) === roadId,
    );
    const stillWaitsForRoad =
      igvState?.waitingRoadSegmentId &&
      this._getRoadIdForSegment(igvState.waitingRoadSegmentId) === roadId;

    if (!stillOccupiesRoad && !stillWaitsForRoad) {
      igvState?.roadIds.delete(roadId);
      removeValue(this._trafficState.roadResources.get(roadId)?.queue || [], igvId);
    }
  }

  _removeIgvFromRoadSegmentState(igvId, segmentId) {
    const igvState = this._trafficState.igvStates.get(igvId);
    if (!igvState || !segmentId) {
      return;
    }

    igvState.roadSegmentIds.delete(segmentId);
    const roadId = this._getRoadIdForSegment(segmentId);
    if (roadId) {
      this._releaseRoadQueueIfDone(igvId, roadId);
    }
  }

  _clearRoadSegmentReservation(segmentState) {
    if (!segmentState) {
      return;
    }
    segmentState.reservedBy = null;
    segmentState.reservedUntil = null;
    segmentState.reservedReason = null;
    segmentState.reservedIntersectionId = null;
  }

  _cleanupExpiredRoadSegmentReservations(engine = this._boundEngine) {
    const now = getEngineClock(engine);
    for (const segmentState of this._trafficState.roadSegmentResources.values()) {
      if (
        segmentState.reservedBy != null &&
        Number.isFinite(segmentState.reservedUntil) &&
        segmentState.reservedUntil <= now
      ) {
        this._clearRoadSegmentReservation(segmentState);
      }
    }
  }

  _isRoadSegmentReservedByOther(
    segmentState,
    igvId,
    engine = this._boundEngine,
  ) {
    if (!segmentState) {
      return false;
    }
    this._cleanupExpiredRoadSegmentReservations(engine);
    return segmentState.reservedBy != null && segmentState.reservedBy !== igvId;
  }

  _canReserveRoadSegments(segmentIds, igvId, engine = this._boundEngine) {
    this._cleanupExpiredRoadSegmentReservations(engine);

    for (const segmentId of segmentIds || []) {
      const segmentState = this._ensureRoadSegmentState(segmentId);
      if (!segmentState) {
        return false;
      }
      if (segmentState.occupiedBy != null && segmentState.occupiedBy !== igvId) {
        return false;
      }
      if (segmentState.reservedBy != null && segmentState.reservedBy !== igvId) {
        return false;
      }
    }

    return true;
  }

  reserveExitSegmentsForIntersection(
    segmentIds,
    igvId,
    intersectionId,
    engine = this._boundEngine,
  ) {
    if (!EXIT_SEGMENT_RESERVATION_ENABLED) {
      return true;
    }

    const ids = Array.isArray(segmentIds) ? segmentIds.filter(Boolean) : [];
    if (ids.length === 0) {
      return true;
    }

    this._cleanupExpiredRoadSegmentReservations(engine);
    if (!this._canReserveRoadSegments(ids, igvId, engine)) {
      return false;
    }

    const now = getEngineClock(engine);
    const until = now + EXIT_SEGMENT_RESERVATION_TTL;
    for (const segmentId of ids) {
      const segmentState = this._ensureRoadSegmentState(segmentId);
      if (!segmentState) {
        return false;
      }
      if (segmentState.occupiedBy == null || segmentState.occupiedBy === igvId) {
        segmentState.reservedBy = igvId;
        segmentState.reservedUntil = until;
        segmentState.reservedReason = "intersection_exit";
        segmentState.reservedIntersectionId = intersectionId;
      }
    }

    return true;
  }

  releaseRoadSegmentReservationsForIgv(igvId, intersectionId = null) {
    for (const segmentState of this._trafficState.roadSegmentResources.values()) {
      if (segmentState.reservedBy !== igvId) {
        continue;
      }
      if (
        intersectionId != null &&
        segmentState.reservedIntersectionId !== intersectionId
      ) {
        continue;
      }
      this._clearRoadSegmentReservation(segmentState);
    }
  }

  _dryCanWakeRoadSegmentWaiter(
    igvId,
    segmentId,
    engine = this._boundEngine,
  ) {
    const igv = engine?.agents?.get?.(igvId);
    if (!(igv instanceof IGVAgent)) {
      return false;
    }

    const segmentState = this._ensureRoadSegmentState(segmentId);
    if (!segmentState) {
      return false;
    }
    if (segmentState.occupiedBy != null && segmentState.occupiedBy !== igvId) {
      return false;
    }
    if (this._isRoadSegmentReservedByOther(segmentState, igvId, engine)) {
      return false;
    }

    const igvState = this._ensureIgvState(igvId);
    if (
      igvState?.waitingRoadSegmentId &&
      igvState.waitingRoadSegmentId !== segmentId
    ) {
      return false;
    }

    return true;
  }

  _wakeNextRoadSegmentWaiter(segmentId, engine = this._boundEngine) {
    const segmentState = this._ensureRoadSegmentState(segmentId);
    if (!segmentState || segmentState.occupiedBy != null) {
      return;
    }

    const roadId = this._getRoadIdForSegment(segmentId, segmentState.roadId);
    if (!ROAD_SEGMENT_SMART_WAKE_ENABLED) {
      while (segmentState.waitingQueue.length > 0) {
        const [nextIgvId] = segmentState.waitingQueue.splice(0, 1);
        segmentState.waitingSinceByIgvId?.delete(nextIgvId);
        const nextIgv = engine?.agents?.get(nextIgvId);
        if (!(nextIgv instanceof IGVAgent)) {
          continue;
        }

        segmentState.occupiedBy = nextIgvId;
        if (segmentState.reservedBy === nextIgvId) {
          this._clearRoadSegmentReservation(segmentState);
        }
        const nextState = this._ensureIgvState(nextIgvId);
        nextState.waitingRoadSegmentId = null;
        nextState.waitingRoadStopDistance = null;
        this._addIgvToRoadState(nextIgvId, roadId, segmentId);
        this.commandAgent(nextIgvId, "accelerate");
        return;
      }
      return;
    }

    const now = getEngineClock(engine);
    segmentState.waitingQueue = segmentState.waitingQueue.filter((igvId) => {
      const igv = engine?.agents?.get(igvId);
      const igvState = this._trafficState.igvStates.get(igvId);
      const keep =
        igv instanceof IGVAgent &&
        igvState?.waitingRoadSegmentId === segmentId;
      if (!keep) {
        segmentState.waitingSinceByIgvId?.delete(igvId);
      }
      return keep;
    });

    if (segmentState.waitingQueue.length === 0) {
      return;
    }

    let selectedIndex = -1;
    const headIgvId = segmentState.waitingQueue[0];
    if (this._dryCanWakeRoadSegmentWaiter(headIgvId, segmentId, engine)) {
      selectedIndex = 0;
    } else {
      const headSince =
        segmentState.waitingSinceByIgvId?.get(headIgvId) ?? now;
      const headStarved =
        now - headSince >= ROAD_SEGMENT_QUEUE_STARVATION_SECONDS;
      if (!headStarved) {
        for (let index = 1; index < segmentState.waitingQueue.length; index += 1) {
          const candidateIgvId = segmentState.waitingQueue[index];
          if (
            this._dryCanWakeRoadSegmentWaiter(
              candidateIgvId,
              segmentId,
              engine,
            )
          ) {
            selectedIndex = index;
            break;
          }
        }
      }
    }

    if (selectedIndex < 0) {
      return;
    }

    const [nextIgvId] = segmentState.waitingQueue.splice(selectedIndex, 1);
    segmentState.waitingSinceByIgvId?.delete(nextIgvId);
    const nextIgv = engine?.agents?.get(nextIgvId);
    if (!(nextIgv instanceof IGVAgent)) {
      return;
    }

    segmentState.occupiedBy = nextIgvId;
    if (segmentState.reservedBy === nextIgvId) {
      this._clearRoadSegmentReservation(segmentState);
    }
    const nextState = this._ensureIgvState(nextIgvId);
    nextState.waitingRoadSegmentId = null;
    nextState.waitingRoadStopDistance = null;
    this._addIgvToRoadState(nextIgvId, roadId, segmentId);
    this.commandAgent(nextIgvId, "accelerate");
  }

  _handleEnterRoad(envelope, engine = this._boundEngine) {
    const igvId = envelope.from;
    const igv = engine?.agents?.get(igvId);
    const payload = envelope.payload || {};
    const segmentId = this._resolveRoadSegmentId(envelope, igv);
    const roadId =
      payload.roadId || this._getRoadIdForSegment(segmentId, payload.resourceId);

    if (!roadId && !segmentId) {
      return;
    }

    if (!segmentId) {
      this._addIgvToRoadState(igvId, roadId, null);
      return;
    }

    const segmentState = this._ensureRoadSegmentState(segmentId, roadId);
    const resolvedRoadId = this._getRoadIdForSegment(segmentId, roadId);
    if (!segmentState || !resolvedRoadId) {
      return;
    }

    const reservedByOther = this._isRoadSegmentReservedByOther(
      segmentState,
      igvId,
      engine,
    );
    if (
      !reservedByOther &&
      (segmentState.occupiedBy == null || segmentState.occupiedBy === igvId)
    ) {
      segmentState.occupiedBy = igvId;
      if (segmentState.reservedBy === igvId) {
        this._clearRoadSegmentReservation(segmentState);
      }
      removeValue(segmentState.waitingQueue, igvId);
      segmentState.waitingSinceByIgvId?.delete(igvId);
      const igvState = this._ensureIgvState(igvId);
      igvState.waitingRoadSegmentId = null;
      igvState.waitingRoadStopDistance = null;
      this._addIgvToRoadState(igvId, resolvedRoadId, segmentId);
      return;
    }

    if (!segmentState.waitingQueue.includes(igvId)) {
      segmentState.waitingQueue.push(igvId);
    }
    if (!(segmentState.waitingSinceByIgvId instanceof Map)) {
      segmentState.waitingSinceByIgvId = new Map();
    }
    if (!segmentState.waitingSinceByIgvId.has(igvId)) {
      segmentState.waitingSinceByIgvId.set(igvId, getEngineClock(engine));
    }
    const igvState = this._ensureIgvState(igvId);
    igvState.waitingRoadSegmentId = segmentId;
    this._addIgvToRoadState(igvId, resolvedRoadId, null);
    const stopTravelDistance = Number.isFinite(payload.stopTravelDistance)
      ? payload.stopTravelDistance
      : Number.isFinite(payload.triggerDistance)
        ? payload.triggerDistance
        : (igv?.getTravelDistance?.() ?? 0);
    igvState.waitingRoadStopDistance = stopTravelDistance;
    this.commandAgent(igvId, "decelerate", { stopTravelDistance });
  }

  _handleLeaveRoad(envelope, engine = this._boundEngine) {
    const igvId = envelope.from;
    const igv = engine?.agents?.get(igvId);
    const payload = envelope.payload || {};
    const segmentId = this._resolveRoadSegmentId(envelope, igv);
    const roadId = payload.roadId || this._getRoadIdForSegment(segmentId);

    if (!segmentId) {
      if (roadId) {
        removeValue(this._trafficState.roadResources.get(roadId)?.queue || [], igvId);
      }
      const igvState = this._trafficState.igvStates.get(igvId);
      igvState?.roadIds.delete(roadId);
      return;
    }

    const segmentState = this._ensureRoadSegmentState(segmentId, roadId);
    const resolvedRoadId = this._getRoadIdForSegment(segmentId, roadId);
    if (!segmentState || !resolvedRoadId) {
      return;
    }

    removeValue(segmentState.waitingQueue, igvId);
    segmentState.waitingSinceByIgvId?.delete(igvId);
    const igvState = this._trafficState.igvStates.get(igvId);
    if (igvState?.waitingRoadSegmentId === segmentId) {
      igvState.waitingRoadSegmentId = null;
      igvState.waitingRoadStopDistance = null;
      this._releaseRoadQueueIfDone(igvId, resolvedRoadId);
    }
    if (segmentState.occupiedBy === igvId) {
      segmentState.occupiedBy = null;
      this._removeIgvFromRoadSegmentState(igvId, segmentId);
      this._wakeNextRoadSegmentWaiter(segmentId, engine);
    }
  }

  _releaseIgv(igvId, engine = this._boundEngine) {
    this.releaseRoadSegmentReservationsForIgv(igvId);
    const igvState = this._trafficState.igvStates.get(igvId);
    if (igvState) {
      igvState.roadIds.clear();
      igvState.roadSegmentIds.clear();
      igvState.waitingRoadSegmentId = null;
      igvState.waitingRoadStopDistance = null;
    }

    for (const roadState of this._trafficState.roadResources.values()) {
      removeValue(roadState.queue, igvId);
    }

    for (const [segmentId, segmentState] of this._trafficState.roadSegmentResources.entries()) {
      removeValue(segmentState.waitingQueue, igvId);
      segmentState.waitingSinceByIgvId?.delete(igvId);
      if (segmentState.occupiedBy === igvId) {
        segmentState.occupiedBy = null;
        this._wakeNextRoadSegmentWaiter(segmentId, engine);
      }
    }
  }

  registerInitialIgvOccupancy(igv, location = igv?.getCurrentLocation?.()) {
    if (!(igv instanceof IGVAgent)) {
      return;
    }
    const normalized = normalizeLocationSpec(location);
    if (!normalized?.laneId) {
      return;
    }
    for (const segmentId of getRoadSegmentIdsCoveredByLocation(normalized)) {
      const segmentState = this._ensureRoadSegmentState(
        segmentId,
        normalized.laneId,
      );
      if (!segmentState || segmentState.occupiedBy != null) {
        continue;
      }
      segmentState.occupiedBy = igv.id;
      this._addIgvToRoadState(igv.id, normalized.laneId, segmentId);
    }
  }

  onMessage(envelope) {
    if (envelope.action === "enter_road") {
      this._handleEnterRoad(envelope);
      return { ok: true };
    }

    if (envelope.action === "leave_road") {
      this._handleLeaveRoad(envelope);
      return { ok: true };
    }

    if (envelope.action === "igv_motion_command") {
      return { ok: true };
    }

    if (envelope.action === "release_igv") {
      const igvId = envelope.payload?.igvId ?? envelope.from;
      this._releaseIgv(igvId);
      return { ok: true };
    }

    if (envelope.action === "reserve_exit_segments") {
      return {
        ok: this.reserveExitSegmentsForIntersection(
          envelope.payload?.segmentIds,
          envelope.payload?.igvId ?? envelope.from,
          envelope.payload?.intersectionId,
        ),
      };
    }

    if (envelope.action === "release_road_segment_reservations") {
      this.releaseRoadSegmentReservationsForIgv(
        envelope.payload?.igvId ?? envelope.from,
        envelope.payload?.intersectionId ?? null,
      );
      return { ok: true };
    }

    if (envelope.action === "can_reserve_exit_segments") {
      return {
        ok: this._canReserveRoadSegments(
          envelope.payload?.segmentIds,
          envelope.payload?.igvId ?? envelope.from,
        ),
      };
    }

    if (envelope.action === "query_road_queue") {
      const roadId =
        typeof envelope.payload === "string"
          ? envelope.payload
          : envelope.payload?.roadId;
      return this.getRoadQueue(roadId);
    }

    if (envelope.action === "query_road_segment_state") {
      const segmentId =
        typeof envelope.payload === "string"
          ? envelope.payload
          : envelope.payload?.segmentId;
      return this.getRoadSegmentState(segmentId);
    }

    return undefined;
  }

  update(dt, engine) {
    this._boundEngine = engine || this._boundEngine;
    this._deadlockCheckTimer = (this._deadlockCheckTimer || 0) + (dt || 0);
    if (this._deadlockCheckTimer >= 1.0) {
      this._deadlockCheckTimer = 0;
      this._checkAndResolveDeadlocks(engine);
    }
  }

  _checkAndResolveDeadlocks(engine) {
    const waitFor = new Map();
    for (const intersectionState of this._trafficState.intersectionResources.values()) {
      const occ = intersectionState.occupiedBy || (intersectionState.activeVehicles?.size ? [...intersectionState.activeVehicles][0] : null);
      if (occ && intersectionState.waitingQueue?.length) {
        for (const entry of intersectionState.waitingQueue) {
          const waiterId = typeof entry === "object" ? entry.igvId : entry;
          if (waiterId && waiterId !== occ) {
            waitFor.set(waiterId, occ);
          }
        }
      }
    }

    const visited = new Set();
    for (const [startId] of waitFor) {
      if (visited.has(startId)) continue;
      const path = [];
      let curr = startId;
      while (curr && !path.includes(curr)) {
        path.push(curr);
        curr = waitFor.get(curr);
      }
      if (curr && path.includes(curr)) {
        const cycle = path.slice(path.indexOf(curr));
        let earliestIgv = null;
        let lowestOrder = Infinity;
        for (const igvId of cycle) {
          const igv = engine?.agents?.get(igvId);
          const order = igv?.getSpawnOrder?.() ?? igvId;
          if (order < lowestOrder) {
            lowestOrder = order;
            earliestIgv = igv;
          }
        }
        if (earliestIgv) {
          earliestIgv.acceptCommand("accelerate");
          this.commandAgent(earliestIgv.id, "accelerate");
        }
        cycle.forEach(id => visited.add(id));
      } else {
        path.forEach(id => visited.add(id));
      }
    }
  }

  onEvent(evt, engine) {
    this._boundEngine = engine;
  }

  update(dt, engine) {
    this._boundEngine = engine;
  }
}

class IntersectionManager extends Scheduler {
  constructor(
    id,
    roadQueueManagerId,
    settings = DEFAULT_WAREHOUSE_SETTINGS,
    trafficState = createTrafficManagerState(),
  ) {
    super(id, "intersection-manager");
    this._settings = normalizeWarehouseSettings(settings);
    this._roadQueueManagerId = roadQueueManagerId;
    this._trafficState = trafficState;
    this._boundEngine = null;
    this.subscribe(RESOURCE_EVENT_TYPE);
  }

  bindEngine(engine) {
    this._boundEngine = engine;
  }

  _ensureIgvState(igvId) {
    return ensureTrafficIgvState(this._trafficState, igvId);
  }

  _ensureIntersectionState(intersectionId) {
    return ensureTrafficIntersectionState(this._trafficState, intersectionId);
  }

  _notifyRoadQueueManager(fromId, action, payload = null) {
    if (!this.bus || this._roadQueueManagerId == null) {
      return undefined;
    }
    return this.bus.send(fromId, this._roadQueueManagerId, action, payload);
  }

  _reserveExitSegmentsForIntersection(
    segmentIds,
    igvId,
    intersectionId,
  ) {
    const result = this._notifyRoadQueueManager(
      this.id,
      "reserve_exit_segments",
      { segmentIds, igvId, intersectionId },
    );
    return result?.ok === true;
  }

  _releaseRoadSegmentReservationsForIgv(igvId, intersectionId = null) {
    this._notifyRoadQueueManager(
      this.id,
      "release_road_segment_reservations",
      { igvId, intersectionId },
    );
  }

  getIntersectionState(intersectionId) {
    const intersectionState =
      this._trafficState.intersectionResources.get(intersectionId);
    if (!intersectionState) {
      return null;
    }
    return {
      occupiedBy: intersectionState.occupiedBy,
      activeVehicles: [...(intersectionState.activeVehicles || [])],
      waitingQueue: intersectionState.waitingQueue.map((entry) =>
        typeof entry === "object" ? entry.igvId : entry,
      ),
    };
  }

  getIgvResourceState(igvId) {
    const igvState = this._trafficState.igvStates.get(igvId);
    if (!igvState) {
      return null;
    }
    return {
      roadIds: [...igvState.roadIds],
      roadSegmentIds: [...igvState.roadSegmentIds],
      waitingRoadSegmentId: igvState.waitingRoadSegmentId,
      waitingRoadStopDistance: igvState.waitingRoadStopDistance,
      intersectionIds: [...igvState.intersectionIds],
      waitingIntersectionId: igvState.waitingIntersectionId,
    };
  }

  _grantIntersection(intersectionId, igv, action = null) {
    const intersectionState = this._ensureIntersectionState(intersectionId);
    if (!intersectionState || !(igv instanceof IGVAgent)) {
      return false;
    }

    const exitSegmentIds = this._getExitSegmentIds(action);
    let exitReserved = false;
    if (EXIT_SEGMENT_RESERVATION_ENABLED && exitSegmentIds.length > 0) {
      exitReserved = this._reserveExitSegmentsForIntersection(
        exitSegmentIds,
        igv.id,
        intersectionId,
      );
      if (!exitReserved) {
        return false;
      }
    }

    try {
      const igvState = this._ensureIgvState(igv.id);
      intersectionState.waitingQueue = intersectionState.waitingQueue.filter(
        (entry) =>
          (typeof entry === "object" ? entry.igvId : entry) !== igv.id,
      );
      if (!(intersectionState.activeVehicles instanceof Set)) {
        intersectionState.activeVehicles = new Set();
      }
      if (!(intersectionState.activeCells instanceof Map)) {
        intersectionState.activeCells = new Map();
      }
      intersectionState.activeVehicles.add(igv.id);
      intersectionState.activeCells.set(
        igv.id,
        new Set(Array.isArray(action?.sweptCells) ? action.sweptCells : []),
      );
      intersectionState.occupiedBy =
        intersectionState.kind === "big" ? null : igv.id;
      igvState.waitingIntersectionId = null;
      igvState.intersectionIds.add(intersectionId);
      this.commandAgent(igv.id, "accelerate");
      return true;
    } catch (error) {
      if (exitReserved) {
        this._releaseRoadSegmentReservationsForIgv(igv.id, intersectionId);
      }
      throw error;
    }
  }

  _intersectionPriority(igv, action, waitingSince = getEngineClock(this._boundEngine)) {
    if (!COMMITTED_VEHICLE_PRIORITY_ENABLED) {
      return DEFAULT_INTERSECTION_PRIORITY;
    }
    const fromLane = LANE_BY_ID.get(action?.fromRoadId);
    const toLane = LANE_BY_ID.get(action?.toRoadId);
    if (action?.committed === true) {
      return COMMITTED_TURN_CLEARANCE_PRIORITY;
    }
    if (
      isBufferLane(fromLane) &&
      getEngineClock(this._boundEngine) - waitingSince >=
        SMALL_INTERSECTION_BUFFER_STARVATION_SECONDS
    ) {
      return MAINLINE_THROUGH_PRIORITY;
    }
    if (!isBufferLane(fromLane) && !isBufferLane(toLane)) {
      return MAINLINE_THROUGH_PRIORITY;
    }
    if (!isBufferLane(fromLane) && isBufferLane(toLane)) {
      return MAINLINE_TO_BUFFER_PRIORITY;
    }
    if (isBufferLane(fromLane) && !isBufferLane(toLane)) {
      return BUFFER_TO_MAINLINE_PRIORITY;
    }
    return DEFAULT_INTERSECTION_PRIORITY;
  }

  _queueIntersectionWaiter(intersectionState, igv, action) {
    const existing = intersectionState.waitingQueue.find(
      (entry) => (typeof entry === "object" ? entry.igvId : entry) === igv.id,
    );
    if (existing) {
      return;
    }
    intersectionState.waitingQueue.push({
      igvId: igv.id,
      action,
      waitingSince: getEngineClock(this._boundEngine),
    });
  }

  _wakeNextIntersectionWaiter(intersectionId, engine) {
    const intersectionState = this._ensureIntersectionState(intersectionId);
    if (!intersectionState) {
      return;
    }

    const grantedThisRound = [];
    while (intersectionState.waitingQueue.length > 0) {
      const ranked = intersectionState.waitingQueue
        .map((entry, index) => ({
          entry: typeof entry === "object" ? entry : { igvId: entry },
          index,
        }))
        .sort((left, right) => {
          if (intersectionState.kind === "small") {
            const leftIgv = engine?.agents?.get(left.entry.igvId);
            const rightIgv = engine?.agents?.get(right.entry.igvId);
            const leftPriority = this._intersectionPriority(
              leftIgv,
              left.entry.action,
              left.entry.waitingSince,
            );
            const rightPriority = this._intersectionPriority(
              rightIgv,
              right.entry.action,
              right.entry.waitingSince,
            );
            if (leftPriority !== rightPriority) {
              return leftPriority - rightPriority;
            }
          }
          return (
            (left.entry.waitingSince ?? 0) - (right.entry.waitingSince ?? 0)
          );
        });

      const candidate = ranked.find(({ entry }) => {
        const nextIgv = engine?.agents?.get(entry.igvId);
        return (
          nextIgv instanceof IGVAgent &&
          this._canGrantIntersection(intersectionId, nextIgv, entry.action)
        );
      });
      if (!candidate) {
        return;
      }

      const nextIgv = engine?.agents?.get(candidate.entry.igvId);
      if (!(nextIgv instanceof IGVAgent)) {
        intersectionState.waitingQueue.splice(candidate.index, 1);
        continue;
      }
      if (!this._grantIntersection(intersectionId, nextIgv, candidate.entry.action)) {
        return;
      }
      grantedThisRound.push(nextIgv.id);
      if (intersectionState.kind !== "big") {
        return;
      }
      if (grantedThisRound.length >= BIG_INTERSECTION_MAX_CONCURRENT) {
        return;
      }
    }
  }

  _canEnterRoadSegmentAction(igv, action) {
    if (action?.resourceType !== "road_segment") {
      return true;
    }
    const segmentId = action.resourceId;
    const segmentState = this._trafficState.roadSegmentResources.get(segmentId);
    if (!segmentState || segmentState.occupiedBy == null) {
      const now = getEngineClock(this._boundEngine);
      const reservedByOther =
        segmentState?.reservedBy != null &&
        segmentState.reservedBy !== igv.id &&
        (!Number.isFinite(segmentState.reservedUntil) ||
          segmentState.reservedUntil > now);
      return !reservedByOther;
    }
    return segmentState.occupiedBy === igv.id;
  }

  _getExitSegmentIds(action) {
    return Array.isArray(action?.exitSegmentIds)
      ? action.exitSegmentIds.filter((id) => typeof id === "string")
      : [];
  }

  _isExitAvailable(igv, action) {
    for (const segmentId of this._getExitSegmentIds(action)) {
      const segmentState = this._trafficState.roadSegmentResources.get(segmentId);
      if (!segmentState) {
        continue;
      }
      if (
        segmentState.occupiedBy != null &&
        segmentState.occupiedBy !== igv.id
      ) {
        return false;
      }
      const now = getEngineClock(this._boundEngine);
      const reservedByOther =
        segmentState.reservedBy != null &&
        segmentState.reservedBy !== igv.id &&
        (!Number.isFinite(segmentState.reservedUntil) ||
          segmentState.reservedUntil > now);
      if (reservedByOther) {
        return false;
      }
    }
    return true;
  }

  _cellsConflict(cells, activeCells) {
    if (!Array.isArray(cells) || cells.length === 0) {
      return false;
    }
    for (const cellId of cells) {
      if (activeCells?.has?.(cellId)) {
        return true;
      }
    }
    return false;
  }

  _canGrantIntersection(intersectionId, igv, action) {
    const intersectionState = this._ensureIntersectionState(intersectionId);
    if (!intersectionState || !(igv instanceof IGVAgent)) {
      return true;
    }
    if (!this._isExitAvailable(igv, action)) {
      return false;
    }

    const kind = intersectionState.kind ?? "small";
    const activeVehicles = intersectionState.activeVehicles ?? new Set();
    const activeCells = intersectionState.activeCells ?? new Map();
    if (activeVehicles.has(igv.id) || intersectionState.occupiedBy === igv.id) {
      return true;
    }

    const cells = Array.isArray(action?.sweptCells) ? action.sweptCells : [];
    if (kind === "big") {
      if (activeVehicles.size >= BIG_INTERSECTION_MAX_CONCURRENT) {
        return false;
      }
      for (const ownerCells of activeCells.values()) {
        if (this._cellsConflict(cells, ownerCells)) {
          return false;
        }
      }
      return true;
    }

    if (activeVehicles.size >= SMALL_INTERSECTION_MAX_CONCURRENT) {
      return false;
    }
    for (const ownerCells of activeCells.values()) {
      if (this._cellsConflict(cells, ownerCells)) {
        return false;
      }
    }
    return true;
  }

  _canEnterResourceAction(igv, action) {
    if (!action || action.action !== "enter") {
      return true;
    }
    if (action.resourceType === "road_segment") {
      return this._canEnterRoadSegmentAction(igv, action);
    }
    if (action.resourceType === "intersection") {
      return this._canGrantIntersection(action.resourceId, igv, action);
    }
    return true;
  }

  _handleIntersectionAction(igv, action, engine) {
    const intersectionId = action.resourceId;
    const intersectionState = this._ensureIntersectionState(intersectionId);
    if (!intersectionState) {
      return true;
    }

    const igvState = this._ensureIgvState(igv.id);
    if (action.action === "enter") {
      if (
        this._canGrantIntersection(intersectionId, igv, action) &&
        this._grantIntersection(intersectionId, igv, action)
      ) {
        return true;
      }

      this._queueIntersectionWaiter(intersectionState, igv, action);
      igvState.waitingIntersectionId = intersectionId;
      const stopTravelDistance = Number.isFinite(action.stopTravelDistance)
        ? action.stopTravelDistance
        : findIntersectionStopDistance(igv);
      this.commandAgent(
        igv.id,
        "decelerate",
        stopTravelDistance != null ? { stopTravelDistance } : undefined,
      );
      return false;
    }

    intersectionState.waitingQueue = intersectionState.waitingQueue.filter(
      (entry) => (typeof entry === "object" ? entry.igvId : entry) !== igv.id,
    );
    igvState.intersectionIds.delete(intersectionId);
    if (intersectionState.occupiedBy === igv.id) {
      intersectionState.occupiedBy = null;
    }
    intersectionState.activeVehicles?.delete?.(igv.id);
    intersectionState.activeCells?.delete?.(igv.id);
    if (igvState.waitingIntersectionId === intersectionId) {
      igvState.waitingIntersectionId = null;
    }
    this._releaseRoadSegmentReservationsForIgv(igv.id, intersectionId);
    this._wakeNextIntersectionWaiter(intersectionId, engine);
    return true;
  }

  _releaseIgvResources(igvId, engine) {
    this._releaseRoadSegmentReservationsForIgv(igvId, null);
    const igvState = this._trafficState.igvStates.get(igvId);
    if (!igvState) {
      return;
    }

    for (const intersectionId of [...igvState.intersectionIds]) {
      this._releaseRoadSegmentReservationsForIgv(igvId, intersectionId);
      const intersectionState = this._ensureIntersectionState(intersectionId);
      if (intersectionState?.occupiedBy === igvId) {
        intersectionState.occupiedBy = null;
      }
      intersectionState?.activeVehicles?.delete?.(igvId);
      intersectionState?.activeCells?.delete?.(igvId);
      if (intersectionState) {
        intersectionState.waitingQueue = intersectionState.waitingQueue.filter(
          (entry) => (typeof entry === "object" ? entry.igvId : entry) !== igvId,
        );
        this._wakeNextIntersectionWaiter(intersectionId, engine);
      }
    }
    if (igvState.waitingIntersectionId) {
      this._releaseRoadSegmentReservationsForIgv(
        igvId,
        igvState.waitingIntersectionId,
      );
      const waitingState = this._ensureIntersectionState(
        igvState.waitingIntersectionId,
      );
      if (waitingState) {
        waitingState.waitingQueue = waitingState.waitingQueue.filter(
          (entry) => (typeof entry === "object" ? entry.igvId : entry) !== igvId,
        );
      }
    }
    this._trafficState.igvStates.delete(igvId);
  }

  processResourceActions(igv, actions, engine) {
    if (
      !(igv instanceof IGVAgent) ||
      !Array.isArray(actions) ||
      actions.length === 0
    ) {
      return { ok: true };
    }

    const leaveActions = [];
    const enterActions = [];
    let shouldFinish = false;
    for (const action of actions) {
      if (action.action === "finish") {
        shouldFinish = true;
        continue;
      }
      if (action.action === "leave") {
        leaveActions.push(action);
        continue;
      }
      if (action.action === "enter") {
        enterActions.push(action);
      }
    }

    for (const action of leaveActions) {
      if (action.resourceType === "road" || action.resourceType === "road_segment") {
        this._notifyRoadQueueManager(
          igv.id,
          "leave_road",
          {
            roadId: action.roadId ?? action.resourceId,
            segmentId:
              action.resourceType === "road_segment" ? action.resourceId : null,
            resourceId: action.resourceId,
            stopTravelDistance: action.stopTravelDistance ?? action.triggerDistance,
            triggerDistance: action.triggerDistance,
            boundaryDistance: action.distance,
          },
        );
        continue;
      }

      if (action.resourceType === "intersection") {
        this._handleIntersectionAction(igv, action, engine);
      }
    }

    const blockedEnter = enterActions.find(
      (action) => !this._canEnterResourceAction(igv, action),
    );
    if (blockedEnter) {
      if (
        blockedEnter.resourceType === "road" ||
        blockedEnter.resourceType === "road_segment"
      ) {
        this._notifyRoadQueueManager(igv.id, "enter_road", {
          roadId: blockedEnter.roadId ?? blockedEnter.resourceId,
          segmentId:
            blockedEnter.resourceType === "road_segment"
              ? blockedEnter.resourceId
              : null,
          resourceId: blockedEnter.resourceId,
          stopTravelDistance:
            blockedEnter.stopTravelDistance ?? blockedEnter.triggerDistance,
          triggerDistance: blockedEnter.triggerDistance,
          boundaryDistance: blockedEnter.distance,
        });
      } else if (blockedEnter.resourceType === "intersection") {
        this._handleIntersectionAction(igv, blockedEnter, engine);
      }
      return {
        ok: false,
        blocked: true,
        blockedResourceType: blockedEnter.resourceType,
        blockedResourceId: blockedEnter.resourceId,
        stopTravelDistance:
          blockedEnter.stopTravelDistance ?? blockedEnter.triggerDistance,
      };
    }

    for (const action of enterActions) {
      if (action.resourceType === "road" || action.resourceType === "road_segment") {
        this._notifyRoadQueueManager(igv.id, "enter_road", {
          roadId: action.roadId ?? action.resourceId,
          segmentId:
            action.resourceType === "road_segment" ? action.resourceId : null,
          resourceId: action.resourceId,
          stopTravelDistance: action.stopTravelDistance ?? action.triggerDistance,
          triggerDistance: action.triggerDistance,
          boundaryDistance: action.distance,
        });
        continue;
      }
      if (action.resourceType === "intersection") {
        const granted = this._handleIntersectionAction(igv, action, engine);
        if (granted === false) {
          return {
            ok: false,
            blocked: true,
            blockedResourceType: action.resourceType,
            blockedResourceId: action.resourceId,
            stopTravelDistance:
              action.stopTravelDistance ?? action.triggerDistance,
          };
        }
      }
    }

    if (shouldFinish) {
      this._releaseIgvResources(igv.id, engine);
      this._notifyRoadQueueManager(this.id, "release_igv", { igvId: igv.id });
    }
    return { ok: true };
  }

  onMessage(envelope) {
    if (envelope.action === "sync_resource_actions") {
      const igv = this._boundEngine?.agents?.get(envelope.from);
      if (igv instanceof IGVAgent) {
        return this.processResourceActions(
          igv,
          envelope.payload?.actions,
          this._boundEngine,
        );
      }
      return { ok: false };
    }

    if (envelope.action === "query_intersection_state") {
      const intersectionId =
        typeof envelope.payload === "string"
          ? envelope.payload
          : envelope.payload?.intersectionId;
      return this.getIntersectionState(intersectionId);
    }

    if (envelope.action === "query_igv_resource_state") {
      const igvId = Number.isFinite(envelope.payload?.igvId)
        ? envelope.payload.igvId
        : envelope.from;
      return this.getIgvResourceState(igvId);
    }

    if (envelope.action === "release_igv") {
      const igvId = envelope.payload?.igvId ?? envelope.from;
      this._releaseIgvResources(igvId, this._boundEngine);
      return { ok: true };
    }

    return undefined;
  }

  onEvent(evt, engine) {
    this._boundEngine = engine;
    if (evt.type !== RESOURCE_EVENT_TYPE) {
      return;
    }

    const igv = engine.agents.get(evt.data?.igvId);
    if (!(igv instanceof IGVAgent)) {
      return;
    }

    const actions = igv.consumeScheduledCheckpoint(evt.data, engine);
    this.processResourceActions(igv, actions, engine);
  }
}

class FrameCollector extends Timer {
  constructor(id) {
    super(id, "frame-collector", "evt_frame", FRAME_INTERVAL);
    this._scheduleToken = 0;
  }

  _scheduleNext(engine, delay = 0) {
    this._scheduleToken += 1;
    engine.scheduleEvent(delay, this.eventType, {
      timerId: this.id,
      token: this._scheduleToken,
    });
  }

  start(engine) {
    this._scheduleNext(engine, 0);
  }

  onSimulationPause() {
    this._scheduleToken += 1;
  }

  onSimulationResume(engine) {
    this._scheduleNext(engine, 0);
  }

  onSimulationSpeedChange(engine, previousSpeed, nextSpeed) {
    const wasUnlimited = engine.isUnlimitedSpeed(previousSpeed);
    const isUnlimited = engine.isUnlimitedSpeed(nextSpeed);
    if (wasUnlimited === isUnlimited) {
      return;
    }
    if (wasUnlimited) {
      this._scheduleNext(engine, 0);
      return;
    }
    this._scheduleToken += 1;
  }

  onEvent(evt, engine) {
    if (
      evt.type !== this.eventType ||
      evt.data?.token !== this._scheduleToken
    ) {
      return;
    }
    if (engine._restoring) {
      this._scheduleNext(
        engine,
        engine.toSimDelay(engine.getRenderFrameInterval()),
      );
      return;
    }

    const nextDelay = engine.publishRenderFrame(performance.now());
    this._scheduleNext(engine, nextDelay);
  }
}

class TaskGenerator extends Timer {
  constructor(id, engine, settings = DEFAULT_WAREHOUSE_SETTINGS) {
    super(id, "task-generator", "evt_task_generate", 1);
    this._settings = normalizeWarehouseSettings(settings);
    this._rngExp = engine.rng.exponential(1 / this._settings.taskMean);
    this._rngUnit = engine.rng.uniform(0, 1);
  }

  start(engine) {
    engine.scheduleEvent(Math.max(1e-3, this._rngExp()), this.eventType, {
      timerId: this.id,
    });
  }

  onEvent(evt, engine) {
    if (evt.type !== this.eventType) {
      return;
    }
    this.onTick(engine);
    engine.scheduleEvent(Math.max(1e-3, this._rngExp()), this.eventType, {
      timerId: this.id,
    });
  }

  _chooseTaskPair(engine) {
    for (let attempt = 0; attempt < TASK_PAIR_SAMPLE_ATTEMPTS; attempt += 1) {
      const reverse = this._rngUnit() >= 0.5;
      const waterside = sampleTaskPoint(QC_TASK_LANES, this._rngUnit);
      const yardSide = sampleTaskPoint(YARD_TASK_LANES, this._rngUnit);
      if (!waterside || !yardSide) {
        continue;
      }

      const taskPair = reverse
        ? { start: yardSide, end: waterside }
        : { start: waterside, end: yardSide };

      if (taskLocationsOverlap(taskPair.start, taskPair.end)) {
        continue;
      }
      if (
        overlapsPendingTaskLocations(engine, taskPair.start) ||
        overlapsPendingTaskLocations(engine, taskPair.end)
      ) {
        continue;
      }
      return taskPair;
    }
    return null;
  }

  _pickAssignee(engine, taskPair) {
    const activeAgvs = getActiveIgvs(engine);
    if (activeAgvs.length === 0) {
      return null;
    }

    const candidates = activeAgvs.map((agv) => ({
      agv,
      taskCount: getIgvUserTaskCount(agv),
      distance: estimateLocationHeuristicDistance(agv.getCurrentLocation(), taskPair.start),
    })).sort((a, b) => {
      if (a.taskCount !== b.taskCount) return a.taskCount - b.taskCount;
      if (Math.abs(a.distance - b.distance) > EPSILON) return a.distance - b.distance;
      return a.agv.id - b.agv.id;
    });

    return candidates[0] ? candidates[0].agv : null;
  }

  onTick(engine) {
    const taskPair = this._chooseTaskPair(engine);
    if (!taskPair) {
      return;
    }

    const assignee = this._pickAssignee(engine, taskPair);
    if (!assignee) {
      return;
    }

    assignUserTaskPairToIgv(this, engine, assignee, taskPair);
    const taskToken = engine._ct2?.taskSequence;

    // Notify yard crane of pickup / dropoff
    if (taskPair.start.row !== "qc") {
      const crane = findCraneForWorkPoint(engine, taskPair.start);
      if (crane) {
        crane.assignPickup(taskToken, taskPair.start);
      }
    }
    if (taskPair.end.row !== "qc") {
      const crane = findCraneForWorkPoint(engine, taskPair.end);
      if (crane) {
        crane.assignDropoff(taskToken, taskPair.end);
      }
    }
  }
}
class YardInit extends Initiator {
  constructor(id, settings = DEFAULT_WAREHOUSE_SETTINGS) {
    super(id, "ct2-init");
    this._settings = normalizeWarehouseSettings(settings);
  }

  onInit(engine) {
    engine._ct2 = {
      activeIgvIds: new Set(),
      activeCraneIds: new Set(),
      cranesByRow: new Map(),
      spawnCounter: 0,
      pendingTaskPairs: new Map(),
      taskSequence: 0,
      roadQueueManagerId: null,
      intersectionManagerId: null,
    };

    // 1. Spawn Yard Cranes (30 cranes from DXF)
    for (const craneDef of YARD_CRANE_DEFINITIONS) {
      const craneId = 1000 + craneDef.index;
      const crane = new YardCraneAgent(craneId, craneDef);
      engine.addAgent(crane);
      engine._ct2.activeCraneIds.add(crane.id);

      const rowKey = craneDef.row;
      const list = engine._ct2.cranesByRow.get(rowKey) || [];
      list.push(crane);
      engine._ct2.cranesByRow.set(rowKey, list);
      engine._ct2.cranesByRow.set(`y${rowKey}`, list);
    }

    // 2. Randomly select 32 unique buffer slots for 32 IGVs
    const availableSlotIndices = Array.from({ length: BUFFER_SLOT_XS.length }, (_, i) => i);
    for (let i = availableSlotIndices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [availableSlotIndices[i], availableSlotIndices[j]] = [availableSlotIndices[j], availableSlotIndices[i]];
    }
    const chosenSlotIndices = availableSlotIndices.slice(0, Math.min(INITIAL_IGV_COUNT, BUFFER_SLOT_XS.length));

    const spawnedIgvs = [];
    for (let index = 0; index < chosenSlotIndices.length; index += 1) {
      const slotIdx = chosenSlotIndices[index];
      const slotX = BUFFER_SLOT_XS[slotIdx];
      const slotLaneId = `buf_${slotIdx + 1}`;
      const spawnLocation = Object.freeze({
        kind: "transit",
        laneId: slotLaneId,
        point: Object.freeze({ x: slotX, z: BUFFER_CENTER_Z }),
      });

      const igv = new IGVAgent(engine.nextId(), spawnLocation, this._settings);
      igv.setSpawnOrder(++engine._ct2.spawnCounter);
      engine.addAgent(igv);
      engine._ct2.activeIgvIds.add(igv.id);
      spawnedIgvs.push({ igv, location: spawnLocation });
    }

    // 3. Traffic Planners
    const trafficState = createTrafficManagerState();
    const roadQueueManager = new RoadQueueManager(engine.nextId(), trafficState);
    engine.addPlanner(roadQueueManager);
    roadQueueManager.bindEngine(engine);
    engine._ct2.roadQueueManagerId = roadQueueManager.id;

    for (const { igv, location } of spawnedIgvs) {
      roadQueueManager.registerInitialIgvOccupancy(igv, location);
    }

    const intersectionManager = new IntersectionManager(
      engine.nextId(),
      roadQueueManager.id,
      this._settings,
      trafficState,
    );
    engine.addPlanner(intersectionManager);
    intersectionManager.bindEngine(engine);
    engine._ct2.intersectionManagerId = intersectionManager.id;

    for (const igvId of engine._ct2.activeIgvIds) {
      engine.agents.get(igvId)?.bindEngine?.(engine);
    }

    // 4. Frame Collector & Task Generator
    const frameCollector = new FrameCollector(engine.nextId());
    engine.addPlanner(frameCollector);
    frameCollector.start(engine);

    const taskGenerator = new TaskGenerator(engine.nextId(), engine, this._settings);
    engine.addPlanner(taskGenerator);
    taskGenerator.start(engine);
  }
}
export async function setupWarehouseScene(
  engine,
  settings = DEFAULT_WAREHOUSE_SETTINGS,
) {
  const sceneSettings = normalizeWarehouseSettings(settings);
  const init = new YardInit(engine.nextId(), sceneSettings);
  engine.addPlanner(init);
}

export function getSceneRenderConfig() {
  return SCENE_RENDER_CONFIG;
}

export const __testUtils = Object.freeze({
  buildRoute,
  buildLockZone,
  createTaskPoint(laneId, x) {
    const laneDefinition = TASK_WORK_LANES.find(
      (lane) => lane.laneId === laneId,
    );
    return laneDefinition ? createTaskPoint(laneDefinition, x) : null;
  },
  getSpawnLocations() {
    return SPAWN_LOCATIONS;
  },
  createTaskMarker(id, color) {
    return new TaskMarkerAgent(id, color);
  },
  chooseGeneratedTaskPair(engine, settings, id = 1) {
    return new TaskGenerator(id, engine, settings)._chooseTaskPair(engine);
  },
  createTestIgv(location, settings, id = 1) {
    return new IGVAgent(id, location, settings);
  },
  getLocation(location) {
    return normalizeLocationSpec(location);
  },
  getPendingTaskPairCount(engine) {
    return getPendingTaskPairStore(engine)?.size ?? 0;
  },
  getVisibleTaskMarkerCount(engine) {
    let count = 0;
    for (const taskPair of getPendingTaskPairStore(engine)?.values() || []) {
      if (
        engine.agents.get(taskPair.startMarkerId)?.getAuxiliaryRenderData?.()
      ) {
        count += 1;
      }
      if (engine.agents.get(taskPair.endMarkerId)?.getAuxiliaryRenderData?.()) {
        count += 1;
      }
    }
    return count;
  },
  registerPendingTaskPair(engine, taskToken, taskPair) {
    registerPendingTaskPair(engine, taskToken, taskPair);
  },
  releasePendingTaskStartMarker(engine, taskToken) {
    releasePendingTaskStartMarker(engine, taskToken);
  },
  releasePendingTaskPair(engine, taskToken) {
    releasePendingTaskPair(engine, taskToken);
  },
  estimateRouteTravelTime,
  routePoseAtDistance,
  taskLocationsOverlap,
  buildRouteResourcePlan,
  ROAD_SECTIONS,
  INTERSECTION_RESOURCES,
  createTrafficManagerState,
  createRoadQueueManager(id = 1, trafficState = createTrafficManagerState()) {
    return new RoadQueueManager(id, trafficState);
  },
  createIntersectionManager(
    id = 2,
    roadQueueManagerId = 1,
    settings,
    trafficState = createTrafficManagerState(),
  ) {
    return new IntersectionManager(
      id,
      roadQueueManagerId,
      settings,
      trafficState,
    );
  },
  estimateTravelTimeToRouteDistance,
  getLane(laneId) {
    return LANE_BY_ID.get(laneId) || null;
  },
  getResourceEventType() {
    return RESOURCE_EVENT_TYPE;
  },
});

export { setupWarehouseScene as setupScene };
