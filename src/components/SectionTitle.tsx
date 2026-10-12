export default function SectionTitle({
  children,
  id,
}: {
  /** Display name of the section */
  children: string;
  /** Section anchor ID, also used for the unique SVG mask */
  id: string;
}) {
  const maskId = `${id}-mask`;
  return (
    <a href={`#${id}`} className="group">
      <div className="section-hd">
        <div
          className="section-striped"
          style={{ "--section-mask": `url(#${maskId})` } as React.CSSProperties}
        >
          <svg
            className="pointer-events-none absolute inset-0"
            width="100%"
            height="100%"
            aria-hidden="true"
          >
            <defs>
              <mask id={maskId} maskUnits="userSpaceOnUse">
                <rect width="100%" height="100%" fill="white" />
                <text
                  x="50%"
                  y="50%"
                  textAnchor="middle"
                  dominantBaseline="central"
                  fill="black"
                  className="section-title"
                >
                  {children}
                </text>
              </mask>
            </defs>
          </svg>
          <h2 className="section-title">{children}</h2>
          <div
            aria-hidden="true"
            className="absolute inset-0 -z-10 mask-(--section-mask) [-webkit-mask-image:var(--section-mask)]"
          >
            <h2 className="section-title-shadow-contrast">{children}</h2>
            <h2 className="section-title-shadow">{children}</h2>
          </div>
        </div>
      </div>
    </a>
  );
}
