import { createMemo, createSignal } from "solid-js";
import {
  useElementSize,
  useMouse,
  useMousePressed,
  useRafFn,
  useWindowSize,
} from "solidjs-use";
import styles from "./HubButton.module.scss";

const acc = 1.2;
const friction = 0.08;
const minDist = 0.3;

type Vec2 = readonly [x: number, y: number];
type Quad = readonly [
  distance: number,
  appliedAcc: Vec2,
  base: Vec2,
  name: string
];

export default function HubButton() {
  const { width: screenWidth, height: screenHeight } = useWindowSize();
  const [el, setEl] = createSignal<HTMLButtonElement>();
  const { width: buttonWidth, height: buttonHeight } = useElementSize(el);
  const { x: mouseX, y: mouseY } = useMouse();
  const { pressed } = useMousePressed();

  const [isDragging, setIsDragging] = createSignal(false);

  const minX = createMemo(() => buttonWidth() / 2);
  const minY = createMemo(() => buttonHeight() / 2);
  const maxX = createMemo(() => screenWidth() - buttonWidth() / 2);
  const maxY = createMemo(() => screenHeight() - buttonHeight() / 2);

  const [x, setX] = createSignal(screenWidth() / 2);
  const [y, setY] = createSignal(screenHeight());

  let vel: Vec2 = [0, 0];
  let current: Vec2 = [x(), y()];

  useRafFn(({ delta }) => {
    const prev = current;

    if (isDragging()) {
      if (!pressed()) {
        setIsDragging(false);
        return;
      }

      current = [mouseX(), mouseY()];

      const [px, py] = prev;
      const [cx, cy] = current;

      vel = [cx - px, cy - py];
      setX(cx);
      setY(cy);
      return;
    }

    const currX = x();
    const currY = y();

    // Define the quadrants and the acceleration.
    const quads: Quad[] = [
      [currX - minX(), [-acc, 0], [0, 1], "left"],
      [maxX() - currX, [acc, 0], [0, 1], "right"],
      [currY - minY(), [0, -acc], [1, 0], "top"],
      [maxY() - currY, [0, acc], [1, 0], "bottom"],
    ];

    ({ vel } = quads.reduce(
      (prev, [dist, [accX, accY], [baseX, baseY]]) => {
        if (dist > prev.dist) return prev;

        // Apply the sign since the acceleration is always positive.
        const [velX, velY] = vel;
        const absDist = Math.abs(dist);
        const applyAbsDist = (v: number, base: number): number =>
          absDist < minDist ? v * base : v;

        const s = Math.sign(dist);
        return {
          vel: [
            applyAbsDist(velX + accX * s, baseX),
            applyAbsDist(velY + accY * s, baseY),
          ],
          dist,
        } as const;
      },
      {
        vel,
        dist: Infinity,
      }
    ));

    // Apply friction.
    vel = vel.map((v) => v * (1 - friction)) as unknown as Vec2;

    // Apply velocity.
    const [vx, vy] = vel;
    setX(currX + vx * delta * 0.1);
    setY(currY + vy * delta * 0.1);
    current = [currX, currY];
  });

  return (
    <div class={styles.hub}>
      <button
        ref={setEl}
        style={{ left: `${x()}px`, top: `${y()}px` }}
        onMouseDown={() => setIsDragging(true)}
      >
        💠
      </button>
    </div>
  );
}
