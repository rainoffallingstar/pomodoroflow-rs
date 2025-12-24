import { useEffect, useRef } from "react";
import "../styles/ParticleSystem.css";

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  opacity: number;
  color: string;
  settled: boolean;
}

interface ParticleSystemProps {
  phase: string;
  isRunning: boolean;
}

/**
 * 粒子系统组件
 * - 从圆环向外喷洒粒子
 * - 粒子受重力影响下落
 * - 粒子在底部堆积
 */
export function ParticleSystem({ phase, isRunning }: ParticleSystemProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const particlesRef = useRef<Particle[]>([]);
  const animationRef = useRef<number>();
  const spawnIntervalRef = useRef<number>();

  // 获取 Phase 颜色
  const getPhaseColor = (phase: string): string => {
    switch (phase) {
      case "work":
        return "#ff9500";
      case "short_break":
        return "#34c759";
      case "long_break":
        return "#007aff";
      default:
        return "#ff9500";
    }
  };

  // 生成粒子
  const spawnParticle = (canvasWidth: number, canvasHeight: number) => {
    const angle = Math.random() * Math.PI * 2;
    const speed = 2 + Math.random() * 3;

    particlesRef.current.push({
      x: canvasWidth / 2,
      y: canvasHeight / 2,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 2,
      radius: 2 + Math.random() * 2,
      opacity: 1,
      color: getPhaseColor(phase),
      settled: false,
    });
  };

  // 检测碰撞
  const isColliding = (p1: Particle, p2: Particle): boolean => {
    const dx = p1.x - p2.x;
    const dy = p1.y - p2.y;
    const distance = Math.sqrt(dx * dx + dy * dy);
    return distance < p1.radius + p2.radius;
  };

  // 更新粒子
  const updateParticles = (canvasWidth: number, canvasHeight: number) => {
    const gravity = 0.15;
    const friction = 0.99;

    particlesRef.current.forEach((particle) => {
      if (particle.settled) return;

      // 应用重力
      particle.vy += gravity;
      particle.vx *= friction;

      // 更新位置
      particle.x += particle.vx;
      particle.y += particle.vy;

      // 边界检测
      if (particle.x - particle.radius < 0) {
        particle.x = particle.radius;
        particle.vx *= -0.5;
      }
      if (particle.x + particle.radius > canvasWidth) {
        particle.x = canvasWidth - particle.radius;
        particle.vx *= -0.5;
      }

      // 底部检测
      if (particle.y + particle.radius >= canvasHeight) {
        particle.y = canvasHeight - particle.radius;
        particle.settled = true;
        particle.vx = 0;
        particle.vy = 0;
      }

      // 堆积检测 - 与已沉淀的粒子碰撞
      for (const other of particlesRef.current) {
        if (other === particle || !other.settled) continue;
        
        if (isColliding(particle, other)) {
          particle.settled = true;
          particle.y = other.y - particle.radius * 2;
          particle.vx = 0;
          particle.vy = 0;
          break;
        }
      }
    });

    // 移除过多粒子（限制数量）
    if (particlesRef.current.length > 150) {
      particlesRef.current = particlesRef.current.slice(-150);
    }
  };

  // 渲染粒子
  const renderParticles = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    particlesRef.current.forEach((particle) => {
      ctx.beginPath();
      ctx.arc(particle.x, particle.y, particle.radius, 0, Math.PI * 2);
      ctx.fillStyle = particle.color;
      ctx.globalAlpha = particle.opacity;
      ctx.fill();
      ctx.globalAlpha = 1;
    });
  };

  // 动画循环
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const animate = () => {
      updateParticles(canvas.width, canvas.height);
      renderParticles();
      animationRef.current = requestAnimationFrame(animate);
    };

    if (isRunning) {
      animate();

      // 定期生成粒子
      spawnIntervalRef.current = window.setInterval(() => {
        // 每次生成 3-5 个粒子
        const count = 3 + Math.floor(Math.random() * 3);
        for (let i = 0; i < count; i++) {
          spawnParticle(canvas.width, canvas.height);
        }
      }, 200);
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
      if (spawnIntervalRef.current) {
        clearInterval(spawnIntervalRef.current);
      }
    };
  }, [isRunning, phase]);

  // 设置 Canvas 尺寸
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resizeCanvas = () => {
      const parent = canvas.parentElement;
      if (parent) {
        canvas.width = parent.clientWidth;
        canvas.height = parent.clientHeight;
      }
    };

    resizeCanvas();
    window.addEventListener("resize", resizeCanvas);

    return () => {
      window.removeEventListener("resize", resizeCanvas);
    };
  }, []);

  return <canvas ref={canvasRef} className="particles-canvas" />;
}
