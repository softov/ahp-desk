/**
 * ahpio's logo for now: ahpapp's mark, from ahpapp's `ahc-logo.svg`. An A whose
 * counter is a four-point star, with three status dots under it.
 */
export function AhpioLogo({ size }: { size: number }) {
  return (
    <svg className="logo" width={size} height={size} viewBox="0 0 895 939" fillRule="evenodd" aria-hidden="true">
      <path fill="#5B9BFF" d="M143 938c19,-48 52,-115 76,-162 76,-148 148,-325 229,-480l303 642 144 -1 -443 -937c-15,9 -4,-2 -14,14l-104 218c-30,67 -328,681 -333,706l143 0z" />
      <path fill="#5B9BFF" fillRule="nonzero" d="M447 541c26,79 53,106 132,132 -79,26 -106,53 -132,132 -26,-79 -53,-106 -132,-132 79,-26 106,-53 132,-132z" />
      <circle fill="#00A859" cx="303" cy="893" r="23" />
      <circle fill="#FFC82A" cx="446" cy="893" r="23" />
      <circle fill="#ED3237" cx="594" cy="893" r="23" />
    </svg>
  );
}
