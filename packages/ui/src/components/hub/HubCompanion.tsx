import { type Accessor, createSignal, onMount } from "solid-js";
import {
  clamp,
  useElementSize,
  useMouse,
  useMousePressed,
  useRafFn,
  useWindowScroll,
  useWindowSize,
} from "solidjs-use";
import HubButton from "./HubButton";
import styles from "./HubCompanion.module.scss";
import { useHubPosition } from "~/contexts";
import { useTrans } from "~/i18n";

const acc = 1.2;
const friction = 0.08;
const minDist = 0.3;
const minVel = 0.001;
const absoluteMax = 50_000;

function minDrag() {
  return matchMedia("(pointer: coarse)").matches ? 30 : 5;
}

function adjustAcc(acc: number, absDist: number): number {
  return absDist <= minDist * 10
    ? acc * Math.min((Math.log(absDist + 0.2) + 2) / 3, 1)
    : acc;
}

function fixNumber(n: number, name: string): number {
  return isNaN(n) ? (console.warn(name, "is NaN"), 0) : n;
}

type Vec2 = readonly [x: number, y: number];
type Quad = readonly [
  distance: number,
  appliedAcc: Vec2,
  base: Vec2,
  name: string,
];

interface ThrowableInit {
  readonly onClick?: (() => void) | undefined;
}

function useRelativeMousePos(): { x: Accessor<number>; y: Accessor<number> } {
  const { x: rawX, y: rawY } = useMouse();
  const { x: scrollX, y: scrollY } = useWindowScroll();
  return {
    x: () => rawX() - scrollX(),
    y: () => rawY() - scrollY(),
  };
}

function motion({ onClick }: ThrowableInit) {
  const { width: screenWidth, height: screenHeight } = useWindowSize();
  const [el, setEl] = createSignal<HTMLButtonElement>();
  const { width: buttonWidth, height: buttonHeight } = useElementSize(el);
  const { x: mouseX, y: mouseY } = useRelativeMousePos();
  const { pressed } = useMousePressed();
  const [containerDisplayed, setContainerDisplayed] =
    createSignal<boolean>(false);
  const { x, setX, y, setY } = useHubPosition();

  const [mouseDrag, setMouseDrag] = createSignal<{
    dragging: boolean;
    startX: number;
    startY: number;
  }>();

  const startDragging = () => {
    setMouseDrag({
      dragging: false,
      startX: mouseX(),
      startY: mouseY(),
    });
  };

  const minX = () => buttonWidth() / 2;
  const minY = () => buttonHeight() / 2;
  const maxX = () => screenWidth() - buttonWidth() / 2;
  const maxY = () => screenHeight() - buttonHeight() / 2;

  let vel: Vec2 = [0, 0];
  let current: Vec2 = [x(), y()];

  const { pause, resume } = useRafFn(
    ({ delta }) => {
      const prev = current;

      const drag = mouseDrag();
      if (drag) {
        const { dragging, startX, startY } = drag;

        if (!pressed()) {
          setMouseDrag(undefined);
          if (!dragging) onClick?.();
          return;
        }

        const [px, py] = prev;
        const [cx, cy] = (current = [mouseX(), mouseY()]);

        if (!dragging) {
          const dist = Math.sqrt(
            Math.pow(cx - startX, 2) + Math.pow(cy - startY, 2),
          );

          if (dist < minDrag()) return;
          setMouseDrag({ dragging: true, startX, startY });
        }

        vel = [cx - px, cy - py];
        setX(cx);
        setY(cy);
        return;
      }

      const currX = fixNumber(x(), "x");
      const currY = fixNumber(y(), "y");

      // Define the quadrants and the acceleration.
      // This is essentially the gravity for each quadrant.
      const quads: Quad[] = [
        [currX - minX(), [-acc, 0], [0, 1], "left"],
        [maxX() - currX, [acc, 0], [0, 1], "right"],
        [currY - minY(), [0, -acc], [1, 0], "top"],
        [maxY() - currY, [0, acc], [1, 0], "bottom"],
      ];

      ({ vel } = quads.reduce(
        (prev, [dist, [accX, accY], [baseX, baseY]]) => {
          if (dist > prev.dist) return prev;

          const [velX, velY] = vel;
          // Zero the velocity if we're close enough to the boundary.
          const absDist = Math.abs(dist);
          const applyMinimums = (v: number, base: number): number =>
            Math.abs(v) < minVel ? 0 : absDist < minDist ? v * base : v;

          // Apply the sign since the acceleration should point towards the boundary.
          const s = Math.sign(dist);
          return {
            vel: [
              applyMinimums(velX + adjustAcc(accX, absDist) * s, baseX),
              applyMinimums(velY + adjustAcc(accY, absDist) * s, baseY),
            ],
            dist,
          } as const;
        },
        {
          vel,
          dist: Infinity,
        },
      ));

      // Apply friction.
      vel = vel.map((v) => v * (1 - friction)) as unknown as Vec2;

      // Apply velocity.
      const [vx, vy] = vel;
      current = [
        setX(
          clamp(
            currX + vx * delta * 0.1,
            -absoluteMax,
            absoluteMax + screenWidth(),
          ),
        ),
        setY(
          clamp(
            currY + vy * delta * 0.1,
            -absoluteMax,
            absoluteMax + screenHeight(),
          ),
        ),
      ];

      if (!containerDisplayed()) {
        setContainerDisplayed(true);
      }
    },
    { immediate: false },
  );

  return { setEl, startDragging, containerDisplayed, pause, resume };
}

export interface HubCompanionProps extends ThrowableInit {
  readonly hidden: Accessor<boolean>;
}

export default function HubCompanion({ hidden, ...props }: HubCompanionProps) {
  const { setEl, startDragging, containerDisplayed, resume } = motion(props);
  const [t] = useTrans();

  onMount(resume);

  return (
    <div
      class={styles.hub}
      classList={{ [styles.visible]: containerDisplayed() }}
    >
      <HubButton
        ref={setEl}
        title={t().core.nav.openHub()}
        classList={{ [styles.hidden]: hidden() }}
        onMouseDown={startDragging}
        onTouchStart={startDragging}
      />
    </div>
  );
}
