import type { ILoadParams } from "@tsparticles/engine";

export const confettiConfig: ILoadParams = {
  id: "tsparticles",

  options: {
    emitters: {
      position: { x: 50, y: 100 },
      rate: { quantity: 100, delay: 0.05 },
      life: { duration: 1, count: 1 }
    },
    particles: {
      move: {
        direction: "top",
        enable: true,
        outModes: "destroy",
        speed: { min: 10, max: 20 },
        gravity: { enable: true, acceleration: 9.8 }
      },
      number: { value: 0 },
      opacity: { value: 1 },
      shape: { type: ["square", "circle"] },
      size: { value: { min: 3, max: 8 } },
      color: { value: ["#FF6B6B", "#4ECDC4", "#FFD93D", "#6BCF7F", "#C77DFF"] },
      life: { duration: { value: 3 } },
      rotate: {
        value: { min: 0, max: 360 },
        direction: "random",
        animation: { enable: true, speed: 30 }
      }
    }
  }
};
