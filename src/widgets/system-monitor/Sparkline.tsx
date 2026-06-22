interface SparklineProps {
  values: number[];
  max: number;
  className?: string;
  width?: number;
  height?: number;
  color?: string;
}

export function Sparkline({
  values,
  max,
  className,
  width = 80,
  height = 24,
  color = "currentColor",
}: SparklineProps) {
  if (values.length < 2) {
    return (
      <svg
        className={className}
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
      />
    );
  }

  const clamp = (n: number) => Math.max(0, Math.min(max, n));
  const step = width / (values.length - 1);
  const toY = (v: number) => height - (clamp(v) / max) * (height - 2) - 1;

  const points = values
    .map((v, i) => `${(i * step).toFixed(1)},${toY(v).toFixed(1)}`)
    .join(" ");

  // Fill area under the line
  const firstX = "0";
  const lastX = ((values.length - 1) * step).toFixed(1);
  const fillPath = `M${firstX},${height} L${points
    .split(" ")
    .map((p, i) => (i === 0 ? `L${p}` : p))
    .join(" ")} L${lastX},${height} Z`;

  return (
    <svg
      className={className}
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <path d={fillPath} fill={color} fillOpacity={0.15} />
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  );
}
