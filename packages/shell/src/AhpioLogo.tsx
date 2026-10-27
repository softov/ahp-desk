/**
 * ahpio's logo: four dots, green at the top, blue in the middle, yellow and
 * red at the bottom corners.
 */
export function AhpioLogo({ size }: { size: number }) {
  return (
    <svg className="logo" width={size} height={size} viewBox="0 0 87 85" aria-hidden="true">
      <circle fill="#00A859" cx="43" cy="11" r="11" />
      <circle fill="#5B9BFF" cx="43" cy="49" r="11" />
      <circle fill="#FFC82A" cx="11" cy="74" r="11" />
      <circle fill="#ED3237" cx="76" cy="74" r="11" />
    </svg>
  );
}
